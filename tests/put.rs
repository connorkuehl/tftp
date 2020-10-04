use std::{io, thread};

use tftp::client;
use tftp::packet::Mode;
use tftp::Server;

#[test]
fn test_put() {
    let serve_dir = tempfile::tempdir().unwrap();
    let (port, server) = Server::random_port("127.0.0.1", serve_dir.path()).unwrap();
    let server_addr = format!("127.0.0.1:{}", port);

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

struct ErroneousReader;

impl io::Read for ErroneousReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, format!("Fake error")))
    }
}

#[test]
fn test_put_sends_error() {
    // Create our server
    let serve_dir = tempfile::tempdir().unwrap();
    let (port, server) = Server::random_port("127.0.0.1", serve_dir.path()).unwrap();
    let server_addr = format!("127.0.0.1:{}", port);

    // Start a thread running its mainloop
    let server_thread = thread::spawn(move || {
        let handler = server.serve().unwrap();
        handler.handle()
    });

    // Create our client
    let client = client::Builder::new()
        .unwrap()
        .connect_to(server_addr)
        .unwrap()
        .build();

    // When trying to create a file with a broken reader, client.put should error out
    client
        .put("broken-file.txt", Mode::NetAscii, ErroneousReader)
        .unwrap_err();

    // When receiving an error packet (due to the broken reader), the server should error out as well
    server_thread.join().unwrap().unwrap_err();
}
