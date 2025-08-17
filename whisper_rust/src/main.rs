// src/main.rs

use std::env;
use std::process;

mod broker;
mod common;
mod node;

use crate::broker::whisper_broker::Broker; // re-export 덕분에 짧게
use crate::node::whisper_node::Node; // re-export 덕분에 짧게
use crate::common::whisper_type_common::WhisperMode;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage:\n  {} broker <addr>\n  {} node <addr>\n\nExamples:\n  {} broker 127.0.0.1:8080\n  {} node 127.0.0.1:8080",
            args[0], args[0], args[0], args[0]
        );
        process::exit(1);
    }

    let subcmd = args[1].as_str();
    let addr = args.get(2).map(String::as_str).unwrap_or_default();

    let mut mode = WhisperMode::DEFAULT;

    match subcmd {
        "broker" => {
            if addr.is_empty() {
                eprintln!("Broker mode requires address argument (e.g., 127.0.0.1:8080)");
                process::exit(1);
            }
            mode = WhisperMode::BROKER;
            println!("Broker mode (addr = {})", addr);

            match Broker::create_broker(addr).await {
                Ok(broker) => {
                    broker.run().await; // loop, 반환 안 됨
                }
                Err(e) => {
                    eprintln!("Failed to start broker: {}", e);
                    process::exit(1);
                }
            }
        }

        "node" => {
            if addr.is_empty() {
                eprintln!("Node mode requires address argument (e.g., 127.0.0.1:8080)");
                process::exit(1);
            }
            mode = WhisperMode::NODE;
            println!("Node mode (broker = {})", addr);

            let node = match Node::connect_to_broker(addr) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to connect to broker: {}", e);
                    process::exit(1);
                }
            };

            // 필요시 기본 핸들러/구독
            // node.register_handler("chat", |msg| println!("[chat] {}", msg));
            // let _ = node.subscribe("chat");

            node.listen();
            node.handle_user_input();

            // Tokio 런타임에서 그냥 슬립하며 생존
            use tokio::time::{sleep, Duration};
            loop {
                sleep(Duration::from_secs(3600)).await;
            }
        }

        _ => {
            eprintln!("Unsupported mode: {}", subcmd);
            process::exit(1);
        }
    }

    // 보통 여기 도달하지 않음
    println!("Selected mode: {:?}", mode as i32);
}
