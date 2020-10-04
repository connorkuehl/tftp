use std::io::{self, Read, Result, Write};
use std::net::UdpSocket;

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::*;

/*
 * TODO: Probably add support for timeouts and retransmissions */

pub const MIN_PORT_NUMBER: u16 = 1001;

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

            let data = match Packet::<Data>::from_bytes(&buf[..bytes_recvd]) {
                Ok(d) => d,

                Err(_) => {
                    let error = match Packet::<Error>::from_bytes(&buf[..bytes_recvd]) {
                        Ok(err) => err,
                        Err(_) => {
                            let _ = self.socket.send(
                                &Packet::error(Code::NotDefined, "invalid packet").into_bytes()[..],
                            );
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "invalid packet",
                            ));
                        }
                    };

                    return Err(io::Error::from(error));
                }
            };

            let _ = writer.write(&data.body.data[..])?;

            let ack = Packet::ack(data.body.block);
            let _ = self.socket.send(&ack.into_bytes()[..])?;

            if data.body.data.len() < MAX_PAYLOAD_SIZE {
                break;
            }
        }

        Ok(writer)
    }

    pub fn put<R: Read>(self, mut reader: R) -> Result<()> {
        let mut current_block = 1;

        loop {
            let mut buf = [0; MAX_PAYLOAD_SIZE];

            let bytes_read = reader.read(&mut buf)?;
            let data = Packet::data(Block::new(current_block), buf[..bytes_read].to_vec());

            let _ = self.socket.send(&data.into_bytes()[..])?;

            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = self.socket.recv(&mut buf)?;

            let ack = match Packet::<Ack>::from_bytes(&buf[..bytes_recvd]) {
                Err(_) => {
                    let error = match Packet::<Error>::from_bytes(&buf[..bytes_recvd]) {
                        Ok(err) => err,
                        Err(_) => {
                            let _ = self.socket.send(
                                &Packet::error(Code::NotDefined, "invalid packet").into_bytes()[..],
                            );
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "invalid packet",
                            ));
                        }
                    };

                    return Err(io::Error::from(error));
                }
                Ok(a) => a,
            };

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
    use rand::Rng;

    use super::*;

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
        let err = f(client_conn).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.into_inner().unwrap().to_string(), "invalid packet");

        // Find the first error packet, assuring we skip over the data packet that gets sent in the put test
        let mut buf = [0; MAX_PACKET_SIZE];
        let rcvd = loop {
            let (rcvd, _) = server_sock.recv_from(&mut buf).unwrap();
            if let Ok(d) = Packet::<Data>::from_bytes(&buf[..rcvd]) {
                continue;
            }
            break rcvd;
        };

        // Assert that we get an "invalid packet" error packet
        let packet_err: io::Error = Packet::<Error>::from_bytes(&buf[..rcvd]).unwrap().into();
        assert_eq!(packet_err.kind(), io::ErrorKind::Other);
        assert_eq!(
            packet_err.into_inner().unwrap().to_string(),
            "invalid packet"
        );
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
