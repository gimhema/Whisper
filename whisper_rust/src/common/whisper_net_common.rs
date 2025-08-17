use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type MessageHandler = dyn Fn(TcpStream, Vec<u8>) + Send + Sync + 'static;

struct TCPServer {
    address: String,
    message_handler: Arc<Mutex<Option<Box<MessageHandler>>>>,
}

impl TCPServer {
    fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            message_handler: Arc::new(Mutex::new(None)),
        }
    }

    fn set_message_handler<F>(&mut self, handler: F)
    where
        F: Fn(TcpStream, Vec<u8>) + Send + Sync + 'static,
    {
        *self.message_handler.lock().unwrap() = Some(Box::new(handler));
    }

    fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;
        println!("Server is listening on {}", self.address);

        for stream in listener.incoming() {
            match stream {
                Ok(conn) => {
                    let handler = Arc::clone(&self.message_handler);
                    thread::spawn(move || {
                        TCPServer::handle_connection(conn, handler);
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_connection(conn: TcpStream, handler: Arc<Mutex<Option<Box<MessageHandler>>>>) {
        let addr = conn.peer_addr().map(|a| a.to_string()).unwrap_or_default();
        println!("Start handle connection, {}", addr);

        let reader = BufReader::new(conn.try_clone().unwrap());
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let trimmed = l.trim();
                    if !trimmed.is_empty() {
                        if let Some(ref h) = *handler.lock().unwrap() {
                            h(conn.try_clone().unwrap(), trimmed.as_bytes().to_vec());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Read error from {}: {}", addr, e);
                    break;
                }
            }
        }
    }
}

// fn main() -> std::io::Result<()> {
//     let mut server = TCPServer::new("127.0.0.1:8080");

//     server.set_message_handler(|mut conn, data| {
//         println!(
//             "Received from {}: {:?}",
//             conn.peer_addr().unwrap(),
//             String::from_utf8_lossy(&data)
//         );
//         // Echo back
//         use std::io::Write;
//         if let Err(e) = writeln!(conn, "{}", String::from_utf8_lossy(&data)) {
//             eprintln!("Write error: {}", e);
//         }
//     });

//     server.run()
// }
