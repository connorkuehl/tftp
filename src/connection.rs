use std::io::{self, Read, Result, Write};
use std::net::UdpSocket;

use crate::bytes::IntoBytes;
use crate::packet::expect::ExpectPacket;
use crate::packet::*;

/*
 * TODO: Probably add support for timeouts and retransmissions */

pub const MIN_PORT_NUMBER: u16 = 1001;

pub struct Connection {
    socket: UdpSocket,
}

impl Connection {
    /// Create a new Connection
    ///
    /// It is assumed that the socket is already connected and already has a read/write timeout set
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }

    pub fn get<W: Write>(self, mut writer: W) -> Result<W> {
        let mut last_block = None;

        loop {
            // Try to get a packet
            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = loop {
                match self.socket.recv(&mut buf) {
                    Ok(bytes_recvd) => break bytes_recvd,
                    Err(error) => match error.kind() {
                        // If we time out, let's assume our last packet got dropped
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                            // If we just sent an ACK, let's resend it
                            if let Some(last_block) = last_block {
                                let ack = Packet::ack(last_block);
                                self.socket.send(&ack.into_bytes()[..])?;
                            } else {
                                // FIXME: resend the read request?
                                return Err(error);
                            }
                        }

                        _ => return Err(error),
                    },
                }
            };

            // Parse it as a data packet or bail
            let data: Packet<Data> = self.socket.expect_packet(&buf[..bytes_recvd])?;

            // FIXME: validate that this isn't a duplicate data packet

            // Write the received data to the writer, and send an error packet if writing failed
            if let Err(err) = writer.write_all(&data.body.data[..]) {
                let _ = self
                    .socket
                    .send(&Packet::error(err.kind().into(), format!("{}", err)).into_bytes()[..]);
                return Err(err);
            }

            // Send an acknowledgement packet
            let ack = Packet::ack(data.body.block);
            self.socket.send(&ack.into_bytes()[..])?;
            last_block = Some(data.body.block);

            // If the data payload length is less than the maximum, then this is the last block
            if data.body.data.len() < MAX_PAYLOAD_SIZE {
                // FIXME: we should "dally" a bit and see if we get the last
                // data packet again, which would mean that the other end of the
                // connection did not receive our last ACK and we should
                // therefore repeat it (see RFC1350 §6 Normal Termination)
                break;
            }
        }

        Ok(writer)
    }

    pub fn put<R: Read>(self, mut reader: R) -> Result<()> {
        let mut current_block = 1;

        loop {
            // Read a block from our reader
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

            // Create a DATA packet for it
            let data = Packet::data(Block::new(current_block), buf[..bytes_read].to_vec());
            let data_bytes = data.into_bytes();

            let ack: Packet<Ack> = loop {
                // Send the latest DATA packet
                self.socket.send(&data_bytes[..])?;

                // Try to receive an ACK packet
                let mut buf = [0; MAX_PACKET_SIZE];
                match self.socket.recv(&mut buf) {
                    Ok(bytes_recvd) => break self.socket.expect_packet(&buf[..bytes_recvd])?,

                    Err(error) => match error.kind() {
                        // If we time out, let's assume our last packet got dropped
                        // and retransmit it by running through the loop again
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                            continue;
                        }

                        _ => return Err(error),
                    },
                }
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
    use std::io;
    use std::time::Duration;

    use rand::Rng;

    use super::*;
    use crate::bytes::FromBytes;
    use crate::packet::{Code, Error};

    const TIMEOUT: Duration = Duration::from_secs(3);

    fn create_server_client() -> (UdpSocket, Connection) {
        // Create our server socket
        let server_port: u16 = rand::thread_rng().gen_range(MIN_PORT_NUMBER, u16::MAX);
        let server_sock = UdpSocket::bind(("localhost", server_port)).unwrap();

        // Create our client socket
        let client_port: u16 = rand::thread_rng().gen_range(MIN_PORT_NUMBER, u16::MAX);
        let client_sock = UdpSocket::bind(("localhost", client_port)).unwrap();

        // Connect them together
        client_sock.connect(("localhost", server_port)).unwrap();
        server_sock.connect(("localhost", client_port)).unwrap();

        // Create a connection struct for our client
        let client_conn = Connection::new(client_sock);

        (server_sock, client_conn)
    }

    fn test_blank_sends_invalid_packet_error<T, F>(f: F)
    where
        T: std::fmt::Debug,
        F: FnOnce(Connection) -> Result<T>,
    {
        const INVALID_PACKET: &[u8] = b"this is an invalid packet. hopefully.";

        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client();

        // Send an (hopefully) invalid packet to the client
        server_sock.send(INVALID_PACKET).unwrap();

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

    #[test]
    fn test_get_retransmits_ack() {
        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client();
        client_conn.socket.set_read_timeout(Some(TIMEOUT)).unwrap();

        // Send the client off into its own little space
        let client_thread = std::thread::spawn(move || client_conn.get(Vec::new()));

        // Pretend the client's sent a read request

        // Let's send our first data packet, making sure it's not a terminator
        server_sock
            .send(&Packet::data(Block::new(1), &[b'h'; MAX_PAYLOAD_SIZE][..]).into_bytes()[..])
            .unwrap();

        // Let's now sleep for.. double the timeout duration. That'll work!
        std::thread::sleep(TIMEOUT * 2);

        // Now, we'll expect to have gotten two ACKs
        let mut prev: Option<Packet<Ack>> = None;
        for _ in 0..2 {
            let mut buf = [0; MAX_PACKET_SIZE];
            let recvd = server_sock.recv(&mut buf).unwrap();
            let packet: Packet<Ack> = server_sock.expect_packet(&buf[..recvd]).unwrap();

            match prev {
                Some(ref prev) => assert_eq!(prev.body.block, packet.body.block),
                None => prev = Some(packet),
            }
        }

        // Now we'll send one last one DATA packet to terminate our client thread
        server_sock
            .send(&Packet::data(Block::new(1), &[b'i'; 1][..]).into_bytes()[..])
            .unwrap();

        // Then make sure our buffer contains the expected data and nothing weird happened
        let buf = client_thread.join().unwrap().unwrap();
        let mut expected = b"h".repeat(MAX_PAYLOAD_SIZE);
        expected.push(b'i');
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_put_retransmits_data() {
        const BOGUS_DATA: &[u8] = b"hey, look, listen";

        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client();
        client_conn.socket.set_read_timeout(Some(TIMEOUT)).unwrap();

        // Send the client off into its own little space to execute their fictitous write request
        let client_thread = std::thread::spawn(move || client_conn.put(BOGUS_DATA));

        // Let's now sleep for.. double the timeout duration. That'll work!
        // Our client's already sent a DATA packet, and we won't send an ACK packet for a while.
        std::thread::sleep(TIMEOUT * 2);

        // Now, we'll expect to have gotten two DATAs
        for _ in 0..2 {
            let mut buf = [0; MAX_PACKET_SIZE];
            let recvd = server_sock.recv(&mut buf).unwrap();
            let packet: Packet<Data> = server_sock.expect_packet(&buf[..recvd]).unwrap();

            assert_eq!(packet.body.block, Block::new(1));
            assert_eq!(packet.body.data, BOGUS_DATA);
        }

        // Now we'll send the ACK packet to terminate our client thread
        server_sock
            .send(&Packet::ack(Block::new(1)).into_bytes()[..])
            .unwrap();

        // Then make sure our client exited successfully
        client_thread.join().unwrap().unwrap();
    }
}
