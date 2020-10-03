use std::thread;

use tftp::client;
use tftp::packet::Mode;
use tftp::Server;

const REPO_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn test_get() {
    let exemplar = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/artifacts/alice-in-wonderland.txt"
    ));

    let mut serve_dir = REPO_ROOT.to_string();
    serve_dir.push_str("/artifacts");
    let (port, server) = Server::random_port("127.0.0.1", serve_dir).unwrap();
    let server_addr = format!("127.0.0.1:{}", port);

    let server_thread = thread::spawn(move || {
        let handler = server.serve().unwrap();
        handler.handle().unwrap();
    });

    let client = client::Builder::new()
        .unwrap()
        .connect_to(server_addr)
        .unwrap()
        .build();

    let actual = Vec::with_capacity(exemplar.len());
    let actual = client
        .get("alice-in-wonderland.txt", Mode::NetAscii, actual)
        .unwrap();
    assert_eq!(&actual[..], &exemplar[..]);

    server_thread.join().unwrap();
}
