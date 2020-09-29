use std::thread;

use tftp::client;
use tftp::packet::Mode;
use tftp::Server;

#[test]
fn test_put() {
    let server_addr = "127.0.0.1:6655";
    let serve_dir = tempfile::tempdir().unwrap();

    let server = Server::new(server_addr, serve_dir.path()).unwrap();
    let server_thread = thread::spawn(move || {
        let handler = server.serve().unwrap();
        handler.handle().unwrap();
    });

    let data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/artifacts/alice-in-wonderland.txt"
    ));

    let client = client::Builder::new()
        .unwrap()
        .connect_to(server_addr)
        .unwrap()
        .build();

    client
        .put("alice-in-wonderland.txt", Mode::NetAscii, &data[..])
        .unwrap();

    let file_path = serve_dir.path().join("alice-in-wonderland.txt");

    let actual = std::fs::read_to_string(&file_path).unwrap();
    let bytes = actual.into_bytes();
    std::fs::remove_file(file_path).unwrap();

    assert_eq!(&bytes[..], &data[..]);

    server_thread.join().unwrap();
}
