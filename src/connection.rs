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
    max_retransmissions: Option<usize>,
}

impl Connection {
    /// Create a new Connection
    ///
    /// It is assumed that the socket is already connected and already has a read/write timeout set
    pub fn new(socket: UdpSocket, max_retransmissions: Option<usize>) -> Self {
        Self {
            socket,
            max_retransmissions,
        }
    }

    fn check_retransmission(
        &self,
        error: io::Error,
        current_retransmissions: &mut usize,
    ) -> Result<()> {
        // Check that this is a timeout error
        if !matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ) {
            return Err(error);
        }

        // Check that we're under the max amount of retransmissions
        *current_retransmissions += 1;
        if let Some(max_retransmissions) = self.max_retransmissions {
            if *current_retransmissions > max_retransmissions {
                let _ =
                    self.socket.send(
                        &Packet::error(Code::NotDefined, "exceeded max retransmissions")
                            .into_bytes()[..],
                    );

                return Err(error);
            }
        }

        Ok(())
    }

    pub fn get<W: Write>(self, mut writer: W) -> Result<W> {
        let mut last_block = None;
        let mut current_retransmissions = 0;

        loop {
            // Try to get a packet
            let mut buf = [0; MAX_PACKET_SIZE];
            let bytes_recvd = loop {
                match self.socket.recv(&mut buf) {
                    Ok(bytes_recvd) => break bytes_recvd,

                    // If we get an error, we either need to retransmit the last packet or bail
                    Err(error) => {
                        // If we've sent an ACK before, then we can move forward
                        // with the retransmission check
                        if let Some(last_block) = last_block {
                            // Check if we should retransmit the current packet
                            self.check_retransmission(error, &mut current_retransmissions)?;

                            // If so, do so and continue through the loop
                            let ack = Packet::ack(last_block);
                            self.socket.send(&ack.into_bytes()[..])?;
                        } else {
                            // Otherwise, we've nothing to retransmit and shall
                            // just return the error. We could either wait here
                            // or retransmit the read request, but that's a FIXME.
                            return Err(error);
                        }
                    }
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
            current_retransmissions = 0;

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
        let mut current_retransmissions = 0;

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

                    // If we get an error, we either need to retransmit the last packet or bail
                    Err(error) => {
                        // Check if we should retransmit the current packet
                        self.check_retransmission(error, &mut current_retransmissions)?;
                        // If so, do so by running through the loop again
                    }
                }
            };

            if Block::new(current_block) != ack.body.block {
                let error = Packet::error(
                    Code::IllegalOperation,
                    format!(
                        "expected ACK for {:?} but got ACK for {:?}",
                        current_block, ack.body.block
                    ),
                );
                self.socket.send(&error.clone().into_bytes()[..])?;
                return Err(io::Error::from(error));
            }
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
    use std::convert::TryFrom;
    use std::io;
    use std::time::Duration;

    use rand::Rng;

    use super::*;
    use crate::bytes::FromBytes;
    use crate::packet::{Code, Error};

    const TIMEOUT: Duration = Duration::from_secs(3);
    const MAX_RETRANSMISSIONS: usize = 3;

    fn create_server_client(max_retransmissions: Option<usize>) -> (UdpSocket, Connection) {
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
        let client_conn = Connection::new(client_sock, max_retransmissions);

        (server_sock, client_conn)
    }

    fn test_blank_sends_invalid_packet_error<T, F>(f: F)
    where
        T: std::fmt::Debug,
        F: FnOnce(Connection) -> Result<T>,
    {
        const INVALID_PACKET: &[u8] = b"this is an invalid packet. hopefully.";

        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client(None);

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
        let (server_sock, client_conn) = create_server_client(None);
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
        let (server_sock, client_conn) = create_server_client(None);
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

    #[test]
    fn test_get_gives_up_after_n_retransmissions() {
        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client(Some(MAX_RETRANSMISSIONS));
        client_conn.socket.set_read_timeout(Some(TIMEOUT)).unwrap();

        // Send the client off into its own little space
        let client_thread = std::thread::spawn(move || client_conn.get(Vec::new()));

        // Pretend the client's sent a read request

        // Let's send our first data packet, making sure it's not a terminator
        server_sock
            .send(&Packet::data(Block::new(1), &[b'h'; MAX_PAYLOAD_SIZE][..]).into_bytes()[..])
            .unwrap();

        // Let's now sleep for long enough that the client'll surely retransmit more than its maximum
        std::thread::sleep(TIMEOUT * u32::try_from(MAX_RETRANSMISSIONS + 1).unwrap());

        // Now, we'll expect to have gotten the one original ACK packet and the n other retransmissions
        let mut prev: Option<Packet<Ack>> = None;
        for _ in 0..(MAX_RETRANSMISSIONS + 1) {
            let mut buf = [0; MAX_PACKET_SIZE];
            let recvd = server_sock.recv(&mut buf).unwrap();
            let packet: Packet<Ack> = server_sock.expect_packet(&buf[..recvd]).unwrap();

            match prev {
                Some(ref prev) => assert_eq!(prev.body.block, packet.body.block),
                None => prev = Some(packet),
            }
        }

        // Then we're expecting an error
        let mut buf = [0; MAX_PACKET_SIZE];
        let recvd = server_sock.recv(&mut buf).unwrap();
        let _packet: Packet<Error> = server_sock.expect_packet(&buf[..recvd]).unwrap();

        // We'll also expect to not have any other datagrams
        server_sock.set_nonblocking(true).unwrap();
        server_sock.recv(&mut [0; MAX_PACKET_SIZE]).unwrap_err();

        client_thread.join().unwrap().unwrap_err();
    }

    #[test]
    fn test_put_gives_up_after_n_retransmissions() {
        const BOGUS_DATA: &[u8] = b"hey, look, listen";

        // Create our server/client pair
        let (server_sock, client_conn) = create_server_client(Some(MAX_RETRANSMISSIONS));
        client_conn.socket.set_read_timeout(Some(TIMEOUT)).unwrap();

        // Send the client off into its own little space to execute their fictitous write request
        let client_thread = std::thread::spawn(move || client_conn.put(BOGUS_DATA));

        // Send the client off into its own little space to execute their fictitous write request
        std::thread::sleep(TIMEOUT * u32::try_from(MAX_RETRANSMISSIONS + 1).unwrap());

        // Now, we'll expect to have gotten the one original DATA packet and the n other retransmissions
        for _ in 0..(MAX_RETRANSMISSIONS + 1) {
            let mut buf = [0; MAX_PACKET_SIZE];
            let recvd = server_sock.recv(&mut buf).unwrap();
            let packet: Packet<Data> = server_sock.expect_packet(&buf[..recvd]).unwrap();

            assert_eq!(packet.body.block, Block::new(1));
            assert_eq!(packet.body.data, BOGUS_DATA);
        }

        // Then we're expecting an error
        let mut buf = [0; MAX_PACKET_SIZE];
        let recvd = server_sock.recv(&mut buf).unwrap();
        let _packet: Packet<Error> = server_sock.expect_packet(&buf[..recvd]).unwrap();

        // We'll also expect to not have any other datagrams
        server_sock.set_nonblocking(true).unwrap();
        server_sock.recv(&mut [0; MAX_PACKET_SIZE]).unwrap_err();

        // We'll expect our client to exit unsuccesfully
        client_thread.join().unwrap().unwrap_err();
    }
}
