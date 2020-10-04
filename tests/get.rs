use std::{io, thread};

use tftp::client;
use tftp::packet::Mode;
use tftp::Server;

#[test]
fn test_get() {
    let exemplar = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/artifacts/alice-in-wonderland.txt"
    ));

    let serve_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts");
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

#[derive(Debug)]
struct ErroneousWriter;

impl io::Write for ErroneousWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, format!("Fake error")))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, format!("Fake error")))
    }
}

// NB: this test just hangs instead of failing before the fix is applied. we
// could add a timeout here if we wanted it to fail instead.
#[test]
fn test_get_sends_error() {
    // Create our server
    let serve_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/artifacts");
    let (port, server) = Server::random_port("127.0.0.1", serve_dir).unwrap();
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

    // When trying to write to a broken writer, the client should error out
    let error = client
        .get("alice-in-wonderland.txt", Mode::NetAscii, ErroneousWriter)
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(format!("{}", error.into_inner().unwrap()), "Fake error");

    // When receiving an error packet (due to the broken writer), the server should error out as well
    server_thread.join().unwrap().unwrap_err();
}
