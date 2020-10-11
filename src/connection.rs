use std::io::{Read, Result, Write};
use std::net::UdpSocket;

use crate::bytes::IntoBytes;
use crate::packet::expect::ExpectPacket;
use crate::packet::*;

/*
 * TODO: Probably add support for timeouts and retransmissions */

pub const MIN_PORT_NUMBER: u16 = 1024;

pub struct Connection {
    socket: UdpSocket,
}

impl Connection {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }

    pub fn get<W: Write>(self, mut writer: W) -> Result<W> {
        loop {
            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = self.socket.recv(&mut buf)?;

            let data: Packet<Data> = self.socket.expect_packet(&buf[..bytes_recvd])?;

            if let Err(err) = writer.write_all(&data.body.data[..]) {
                let _ = self
                    .socket
                    .send(&Packet::error(err.kind().into(), format!("{}", err)).into_bytes()[..]);
                return Err(err);
            }

            let payload_size = data.body.data.len();
            let ack = Packet::<Ack>::from(data);
            let _ = self.socket.send(&ack.into_bytes()[..])?;

            if payload_size < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(writer)
    }

    pub fn put<R: Read>(self, mut reader: R) -> Result<()> {
        let mut current_block = 1;

        loop {
            let mut buf = [0; MAX_PAYLOAD_SIZE];

            let bytes_read = match reader.read(&mut buf) {
                Ok(bytes_read) => bytes_read,
                Err(err) => {
                    let _ = self.socket.send(
                        &Packet::error(err.kind().into(), format!("{}", err)).into_bytes()[..],
                    );
                    return Err(err);
                }
            };

            let data = Packet::data(Block::new(current_block), buf[..bytes_read].to_vec());

            let _ = self.socket.send(&data.into_bytes()[..])?;

            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = self.socket.recv(&mut buf)?;

            let ack: Packet<Ack> = self.socket.expect_packet(&buf[..bytes_recvd])?;

            assert_eq!(Block::new(current_block), ack.body.block);
            current_block += 1;

            if bytes_read < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use rand::Rng;

    use super::*;
    use crate::bytes::FromBytes;
    use crate::packet::{Code, Error};

    fn test_blank_sends_invalid_packet_error<T, F>(f: F)
    where
        T: std::fmt::Debug,
        F: FnOnce(Connection) -> Result<T>,
    {
        const INVALID_PACKET: &[u8] = b"this is an invalid packet. hopefully.";

        // Create our server socket
        let server_port: u16 = rand::thread_rng().gen_range(MIN_PORT_NUMBER, u16::MAX);
        let server_sock = UdpSocket::bind(("localhost", server_port)).unwrap();

        // Create our client socket
        let client_port: u16 = rand::thread_rng().gen_range(MIN_PORT_NUMBER, u16::MAX);
        let client_sock = UdpSocket::bind(("localhost", client_port)).unwrap();

        // Connect them together
        client_sock.connect(("localhost", server_port)).unwrap();

        // Create a connection struct for our client
        let client_conn = Connection::new(client_sock);

        // Send an (hopefully) invalid packet
        server_sock
            .send_to(INVALID_PACKET, ("localhost", client_port))
            .unwrap();

        // Assert that, when trying to <blank>, we get an invalid packet error
        // let err = client_conn.get(&mut Vec::new()).unwrap_err();
        let actual = f(client_conn).unwrap_err();
        let expected: io::Error =
            Packet::error(Code::IllegalOperation, Code::IllegalOperation.as_str()).into();
        assert_eq!(actual.kind(), expected.kind());

        // Find the first error packet, assuring we skip over the data packet that gets sent in the put test
        let mut buf = [0; MAX_PACKET_SIZE];
        let rcvd = loop {
            let (rcvd, _) = server_sock.recv_from(&mut buf).unwrap();
            if let Ok(_) = Packet::<Data>::from_bytes(&buf[..rcvd]) {
                continue;
            }
            break rcvd;
        };

        // Assert that we get an "invalid packet" error packet
        let actual: io::Error = Packet::<Error>::from_bytes(&buf[..rcvd]).unwrap().into();
        assert_eq!(actual.kind(), expected.kind());
    }

    #[test]
    fn test_get_sends_invalid_packet_error() {
        test_blank_sends_invalid_packet_error(|conn| conn.get(Vec::new()))
    }

    #[test]
    fn test_put_sends_invalid_packet_error() {
        test_blank_sends_invalid_packet_error(|conn| conn.put(&b"wowzers"[..]))
    }
}
