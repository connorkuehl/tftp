use std::io::ErrorKind;
use std::io::Result;
use std::thread;
use tempfile::TempDir;
use tftp::packet::Mode;
use tftp::{client, Server};

fn run(server: Server, num_queries: usize) {
    for _ in 0..num_queries {
        if let Ok(h) = server.serve() {
            match h.handle() {
                Ok(()) => println!("OK"),
                Err(e) => println!("FAIL: {:?}", e),
            };
        }
    }
}

pub fn clone_client_builder(
    client_builder: &client::Builder<client::ConnectTo>,
) -> Result<client::Builder<client::ConnectTo>> {
    match client_builder.try_clone() {
        Err(err) if err.kind() == ErrorKind::AddrInUse => clone_client_builder(client_builder),
        ok_or_error => ok_or_error,
    }
}

pub fn main() {
    let temp_dir = TempDir::new().unwrap();
    let server_ip = "127.0.0.1";
    let num_queries = 1000;
    let (port, server) = Server::random_port(server_ip, temp_dir.path()).unwrap();
    let server_thread = thread::spawn(move || run(server, num_queries));
    let client_builder = client::Builder::new()
        .unwrap()
        .connect_to(format!("{}:{}", server_ip, port))
        .unwrap();

    for idx in 0..num_queries / 2 {
        let client = clone_client_builder(&client_builder).unwrap().build();
        let filename = format!("file{}.txt", idx);
        let content = format!("file{} content", idx);
        client
            .put(filename, Mode::NetAscii, content.as_bytes())
            .unwrap();
    }

    for idx in 0..num_queries / 2 {
        let client = clone_client_builder(&client_builder).unwrap().build();
        let filename = format!("file{}.txt", idx);
        client
            .get(filename, Mode::NetAscii, std::io::stdout())
            .unwrap();
    }
    server_thread.join().unwrap();
}
