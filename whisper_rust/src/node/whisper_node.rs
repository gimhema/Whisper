use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

type Handler = Box<dyn Fn(String) + Send + Sync>;

struct Node {
    conn: Arc<Mutex<TcpStream>>,
    id: String,
    subscribers: Arc<Mutex<HashMap<String, Handler>>>,
}

impl Node {
    fn connect_to_broker(address: &str) -> io::Result<Self> {
        let conn = TcpStream::connect(address)?;
        let id = conn.local_addr()?.to_string();
        Ok(Node {
            conn: Arc::new(Mutex::new(conn)),
            id,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn subscribe(&self, topic: &str) -> io::Result<()> {
        let payload = format!("SUB {}\n", topic);
        self.conn.lock().unwrap().write_all(payload.as_bytes())
    }

    fn register_handler<F>(&self, topic: &str, handler: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.subscribers
            .lock()
            .unwrap()
            .insert(topic.to_string(), Box::new(handler));
    }

    fn publish(&self, topic: &str, message: &str) -> io::Result<()> {
        let payload = format!("PUB {} {}\n", topic, message);
        self.conn.lock().unwrap().write_all(payload.as_bytes())
    }

    fn handle_user_input(&self) {
        let node = self.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut input = String::new();

            println!("Type commands: (e.g., 'SUB topic', 'PUB topic message')");
            loop {
                input.clear();
                if stdin.read_line(&mut input).unwrap() == 0 {
                    break;
                }
                let parts: Vec<&str> = input.trim().splitn(3, ' ').collect();
                if parts.len() < 2 {
                    println!("Invalid command. Usage: SUB <topic> | PUB <topic> <message>");
                    continue;
                }
                let command = parts[0].to_uppercase();
                let topic = parts[1];

                match command.as_str() {
                    "SUB" => {
                        if let Err(e) = node.subscribe(topic) {
                            println!("Subscribe error: {}", e);
                        } else {
                            println!("Subscribed to topic: {}", topic);
                        }
                    }
                    "PUB" => {
                        if parts.len() < 3 {
                            println!("Publish needs a topic and a message. Usage: PUB <topic> <message>");
                            continue;
                        }
                        let message = parts[2];
                        if let Err(e) = node.publish(topic, message) {
                            println!("Publish error: {}", e);
                        }
                    }
                    _ => println!("Unknown command: {}", command),
                }
            }
        });
    }

    fn listen(&self) {
        let conn = self.conn.lock().unwrap().try_clone().unwrap();
        let subscribers = Arc::clone(&self.subscribers);

        thread::spawn(move || {
            let reader = BufReader::new(conn);
            for line in reader.lines() {
                match line {
                    Ok(msg) => {
                        let parts: Vec<&str> = msg.trim().splitn(3, ' ').collect();
                        if parts.len() < 3 {
                            println!("Malformed message: {}", msg);
                            continue;
                        }
                        let command = parts[0];
                        let topic = parts[1];
                        let body = parts[2];

                        println!("Raw Message : {}", body);

                        if command == "MSG" {
                            if let Some(handler) = subscribers.lock().unwrap().get(topic) {
                                handler(body.to_string());
                            } else {
                                println!("No handler registered for topic: {}", topic);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Read error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            conn: Arc::clone(&self.conn),
            id: self.id.clone(),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

fn main() -> io::Result<()> {
    let node = Node::connect_to_broker("127.0.0.1:8080")?;

    // 예시: 토픽별 핸들러 등록
    node.register_handler("chat", |msg| {
        println!("[Chat] Received: {}", msg);
    });

    // 브로커에 구독 요청
    node.subscribe("chat")?;

    // 서버 수신 대기
    node.listen();

    // 사용자 입력 처리
    node.handle_user_input();

    // 프로그램 종료 방지
    loop {
        thread::park();
    }
}
