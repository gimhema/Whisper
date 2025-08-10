use std::env;
use std::process;
use std::thread;

mod common;
mod broker;
mod node;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut mode = common::Mode::Default;

    if args.len() > 1 {
        let val = &args[1];
        println!("Arguments: {}", val);

        if val == "broker" {
            if args.len() < 3 {
                eprintln!("Broker mode requires address argument (e.g., 127.0.0.1:8080)");
                process::exit(1);
            }
            let conn_info = &args[2];
            mode = common::Mode::Broker;
            println!("Broker mode");

            let broker = broker::create_broker(conn_info);
            broker.run();

        } else if val == "node" {
            if args.len() < 3 {
                eprintln!("Node mode requires address argument (e.g., 127.0.0.1:8080)");
                process::exit(1);
            }
            let conn_info = &args[2];
            mode = common::Mode::Node;
            println!("Node mode");

            let node = match node::Node::connect_to_broker(conn_info) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to connect to broker: {}", e);
                    process::exit(1);
                }
            };

            node.setup_handler();

            let node_clone = node.clone();
            thread::spawn(move || {
                node_clone.listen();
            });

            node.handle_user_input();

            // 프로그램 종료 방지
            loop {
                thread::park();
            }
        } else {
            println!("Unsupported mode");
        }
    } else {
        println!("No arguments passed.");
    }

    println!("Selected mode: {:?}", mode);
}
