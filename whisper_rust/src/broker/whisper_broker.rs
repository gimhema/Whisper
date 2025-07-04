// Cargo.toml 파일에 다음 의존성을 추가해야 합니다:
// [dependencies]
// tokio = { version = "1", features = ["full"] }

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex; // 비동기 컨텍스트를 위한 tokio의 Mutex 사용
use std::fmt;
use std::error::Error;

// 브로커 작업에서 발생할 수 있는 오류를 위한 사용자 정의 열거형
// MutexLockError는 tokio::sync::Mutex가 패닉 시 복구할 수 없기 때문에
// 이론적으로는 발생하지 않거나, 발생 시 복구 불가능한 에러로 간주됩니다.
// 하지만 다른 I/O 에러 등은 여전히 발생할 수 있으므로 Error 타입을 유지합니다.
#[derive(Debug)]
enum BrokerError {
    Io(std::io::Error),
    InvalidMessageFormat,
    MissingMessage,
    UnknownCommand,
    // tokio::sync::Mutex는 락 획득 실패 시 패닉을 일으키므로 이 에러는 직접적으로 발생하지 않습니다.
    // 하지만 다른 잠금 관련 로직(예: RwLock 등)에서 발생할 수 있으므로 남겨둘 수도 있습니다.
    // 현재 코드에서는 사실상 사용되지 않습니다.
    MutexLockError,
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrokerError::Io(e) => write!(f, "I/O error: {}", e),
            BrokerError::InvalidMessageFormat => write!(f, "Invalid message format"),
            BrokerError::MissingMessage => write!(f, "Missing message content"),
            BrokerError::UnknownCommand => write!(f, "Unknown command"),
            BrokerError::MutexLockError => write!(f, "Failed to acquire mutex lock (this should not happen with tokio::sync::Mutex directly)"),
        }
    }
}

impl Error for BrokerError {}

// std::io::Error를 BrokerError로 쉽게 변환하기 위한 From 트레이트 구현
impl From<std::io::Error> for BrokerError {
    fn from(err: std::io::Error) -> Self {
        BrokerError::Io(err)
    }
}

// `tokio::sync::AcquireError`에 대한 `From` 구현은 더 이상 필요하지 않습니다.
// 왜냐하면 `tokio::sync::Mutex::lock().await`가 `Result`를 반환하지 않기 때문입니다.
// 만약 `std::sync::Mutex`를 사용한다면 `std::sync::PoisonError`에 대한 From 구현이 필요합니다.


// 구독자(Subscriber)를 나타내는 구조체
// The writer half of the TcpStream is protected by a Mutex to allow safe concurrent writes
// if multiple publishers or other parts of the system need to write to the same subscriber.

struct Subscriber {
    id: String,
    writer: Mutex<tokio::net::tcp::OwnedWriteHalf>,
}

// 메시지 브로커를 나타내는 구조체
pub struct Broker {
    listener: TcpListener,
    // 구독 정보와 모든 구독자 목록은 Arc<Mutex<HashMap<...>>>를 통해 공유 가능한 상태로 관리됩니다.
    subscriptions: Arc<Mutex<HashMap<String, HashMap<String, Arc<Subscriber>>>>>, // 토픽 -> (구독자 ID -> 구독자)
    subscribers: Arc<Mutex<HashMap<String, Arc<Subscriber>>>>, // 모든 활성 구독자 (ID -> 구독자)
}

impl Broker {
    /// 주어진 주소로 브로커를 생성하고 TCP 리스너를 바인딩합니다.
    pub async fn create_broker(conn_info: &str) -> Result<Broker, BrokerError> {
        let listener = TcpListener::bind(conn_info).await.map_err(BrokerError::Io)?;
        println!("Broker listening on {}", conn_info);
        Ok(Broker {
            listener,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 브로커를 실행하고 들어오는 연결을 처리합니다.
    pub async fn run(&self) {
        println!("Running Message Broker");

        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New client connected: {}", addr);
                    // 공유 상태를 새로운 태스크로 클론하여 전달합니다.
                    let subscriptions_clone = Arc::clone(&self.subscriptions);
                    let subscribers_clone = Arc::clone(&self.subscribers);

                    // 각 클라이언트 연결을 별도의 비동기 태스크에서 처리합니다.
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, subscriptions_clone, subscribers_clone).await {
                            eprintln!("Error handling client {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    // 실제 애플리케이션에서는 여기에서 더 정교한 오류 처리를 할 수 있습니다.
                }
            }
        }
    }

    /// 단일 클라이언트 연결을 처리하는 함수입니다.
    async fn handle_client(
        stream: TcpStream,
        subscriptions: Arc<Mutex<HashMap<String, HashMap<String, Arc<Subscriber>>>>>,
        all_subscribers: Arc<Mutex<HashMap<String, Arc<Subscriber>>>>,
    ) -> Result<(), BrokerError> {
        let client_addr = stream.peer_addr().map_err(BrokerError::Io)?.to_string();
        println!("Handling client: {}", client_addr);

        // TcpStream을 읽기 및 쓰기 부분으로 분할합니다.
        let (reader_half, writer_half) = stream.into_split();
        let mut reader = BufReader::new(reader_half);
        let mut buffer = Vec::new();

        // 이 클라이언트를 위한 Subscriber 인스턴스를 한 번 생성합니다.
        let subscriber_arc = Arc::new(Subscriber {
            id: client_addr.clone(),
            writer: Mutex::new(writer_half),
        });

        // 이 구독자를 모든 구독자 목록에 추가합니다.
        { // Scoped lock for all_subscribers
            let mut all_subs_guard = all_subscribers.lock().await; // `?` 제거
            all_subs_guard.insert(client_addr.clone(), Arc::clone(&subscriber_arc));
        }

        loop {
            buffer.clear();
            // 메시지를 줄바꿈 문자로 구분하여 읽습니다.
            let bytes_read = reader.read_until(b'\n', &mut buffer).await.map_err(BrokerError::Io)?;

            if bytes_read == 0 {
                // 클라이언트가 연결을 닫았습니다.
                println!("Client {} disconnected.", client_addr);
                // 구독자를 전역 목록 및 토픽 구독에서 정리합니다.
                Self::cleanup_subscriber(client_addr, subscriptions, all_subscribers).await?;
                break;
            }

            let msg = String::from_utf8_lossy(&buffer).trim_end_matches('\n').to_string();
            println!("Received Raw Message from {}: {}", client_addr, msg);

            let parts: Vec<&str> = msg.splitn(3, ' ').collect();

            if parts.len() < 2 {
                let mut writer_guard = subscriber_arc.writer.lock().await; // `?` 제거
                writer_guard.write_all(b"ERR invalid message format\n").await.map_err(BrokerError::Io)?;
                continue;
            }

            let cmd = parts[0];
            let topic = parts[1].to_string(); // 소유권이 있는 String으로 변환

            match cmd {
                "SUB" => {
                    // 구독을 수행하고 클라이언트에게 확인 메시지를 보냅니다.
                    Self::perform_subscription(
                        Arc::clone(&subscriber_arc), // 이 클라이언트를 위한 Arc<Subscriber> 전달
                        topic.clone(),
                        subscriptions.clone(),
                    ).await?;
                    let mut writer_guard = subscriber_arc.writer.lock().await; // `?` 제거
                    writer_guard.write_all(format!("SUBSCRIBED to {}\n", topic).as_bytes()).await.map_err(BrokerError::Io)?;
                },
                "PUB" => {
                    if parts.len() < 3 {
                        let mut writer_guard = subscriber_arc.writer.lock().await; // `?` 제거
                        writer_guard.write_all(b"ERR missing message\n").await.map_err(BrokerError::Io)?;
                        continue;
                    }
                    let message = parts[2].to_string(); // 소유권이 있는 String으로 변환
                    Self::publish(topic, message, subscriptions.clone()).await?;
                },
                _ => {
                    let mut writer_guard = subscriber_arc.writer.lock().await; // `?` 제거
                    writer_guard.write_all(b"ERR unknown command\n").await.map_err(BrokerError::Io)?;
                }
            }
        }
        Ok(())
    }

    /// 실제 구독 로직을 수행하는 헬퍼 함수입니다.
    async fn perform_subscription(
        subscriber_arc: Arc<Subscriber>,
        topic: String,
        subscriptions: Arc<Mutex<HashMap<String, HashMap<String, Arc<Subscriber>>>>>,
    ) -> Result<(), BrokerError> {
        let mut subs_guard = subscriptions.lock().await;
        let topic_subs = subs_guard.entry(topic.clone()).or_insert_with(HashMap::new);
        
        // 문제의 라인: subscriber_arc의 클론을 insert 메서드에 전달합니다.
        topic_subs.insert(subscriber_arc.id.clone(), Arc::clone(&subscriber_arc)); 
        // 또는 더 간결하게: topic_subs.insert(subscriber_arc.id.clone(), subscriber_arc.clone());

        println!("Subscriber {} SUBSCRIBED to {}", subscriber_arc.id, topic); // 이제 subscriber_arc를 사용할 수 있습니다.
        Ok(())
    }

    /// 주어진 토픽에 메시지를 발행합니다.
    async fn publish(
        topic: String,
        message: String,
        subscriptions: Arc<Mutex<HashMap<String, HashMap<String, Arc<Subscriber>>>>>,
    ) -> Result<(), BrokerError> {
        println!("Publishing to topic: {}", topic);

        let mut subs_guard = subscriptions.lock().await; // `?` 제거
        let subs_for_topic = subs_guard.get_mut(&topic);

        if subs_for_topic.is_none() || subs_for_topic.as_ref().unwrap().is_empty() {
            println!("No subscribers for topic: {}", topic);
            return Ok(());
        }

        let subs_map = subs_for_topic.unwrap();
        let mut ids_to_remove = Vec::new();
        let full_message = format!("MSG {} {}\n", topic, message);
        let message_bytes = full_message.as_bytes();

        // 토픽의 모든 구독자에게 메시지를 보냅니다.
        for (id, sub_arc) in subs_map.iter() {
            let mut writer_guard = sub_arc.writer.lock().await; // `?` 제거
            if let Err(e) = writer_guard.write_all(message_bytes).await {
                eprintln!("Failed to send to subscriber: {} error: {}", id, e);
                ids_to_remove.push(id.clone());
            }
        }

        // 반복 후 연결이 끊어진 구독자를 제거합니다.
        for id in ids_to_remove {
            subs_map.remove(&id);
            println!("Removed disconnected subscriber: {} from topic {}", id, topic);
            // Go 코드와 마찬가지로, publish 실패 시에는 해당 토픽의 맵에서만 제거합니다.
            // 전역 `subscribers` 맵에서의 제거는 `cleanup_subscriber`가 처리합니다.
        }

        Ok(())
    }

    /// 연결이 끊어진 구독자를 정리하는 헬퍼 함수입니다.
    async fn cleanup_subscriber(
        client_id: String,
        subscriptions: Arc<Mutex<HashMap<String, HashMap<String, Arc<Subscriber>>>>>,
        all_subscribers: Arc<Mutex<HashMap<String, Arc<Subscriber>>>>,
    ) -> Result<(), BrokerError> {
        // 전역 목록에서 제거
        let mut all_subs_guard = all_subscribers.lock().await; // `?` 제거
        if all_subs_guard.remove(&client_id).is_some() {
            println!("Removed subscriber {} from global list.", client_id);
        }

        // 특정 토픽 구독에서 제거
        let mut subs_guard = subscriptions.lock().await; // `?` 제거
        for (_topic, topic_subs) in subs_guard.iter_mut() {
            topic_subs.remove(&client_id);
        }
        Ok(())
    }
}

// 브로커를 실행하기 위한 메인 함수
#[tokio::main] // tokio 런타임을 사용하여 비동기 메인 함수를 실행합니다.
async fn main() -> Result<(), BrokerError> {
    let broker = Broker::create_broker("127.0.0.1:8080").await?;
    broker.run().await;
    Ok(())
}