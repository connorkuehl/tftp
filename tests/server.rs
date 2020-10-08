use std::net::UdpSocket;

use tftp::Server;

#[test]
fn test_serve_when_request_is_not_read_or_write() {
    let serve_dir = tempfile::tempdir().unwrap();
    let serve_addr = "127.0.0.1";
    let (port, server) = Server::random_port(&serve_addr, serve_dir.path()).unwrap();

    let socket = UdpSocket::bind("0.0.0.0:12345").unwrap();

    let op = vec![0, 5];
    let mut code = vec![0, 1];
    let mut message = b"file not found\0".to_vec();
    let mut bytes = op;
    bytes.append(&mut code);
    bytes.append(&mut message);

    let _ = socket
        .send_to(&bytes[..], &format!("{}:{}", serve_addr, port))
        .unwrap();

    assert!(server.serve().is_err());
}
