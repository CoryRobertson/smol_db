use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use smol_db_common::this;

fn main() {
    println!("Hello, world!");
    this();

    let listener = TcpListener::bind("0.0.0.0:8222").unwrap();

    let mut thread_vec: Vec<JoinHandle<()>> = vec![];

    for income in listener.incoming() {
        for i in 0..thread_vec.len() {
            match thread_vec.get(i) {
                None => {}
                Some(t) => {
                    if t.is_finished() {
                        thread_vec.remove(i);
                    }
                }

            }
        }

        let handle = thread::spawn(move || {
            let stream = income.expect("failed to recieve tcp stream");
            loop {
                // client loop
            }
        });

        thread_vec.push(handle);
        println!("connection handled. number of connections: {}", thread_vec.len());

    }

}

fn handle_client(stream: &TcpStream) -> bool {
    true
}