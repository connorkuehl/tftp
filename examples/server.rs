use std::convert::{TryFrom, TryInto};
use std::env;
use std::fs::{File, OpenOptions};
use std::net::UdpSocket;

use rand::Rng;

use tftp::*;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage ./server address:port directory");
    }

    let addr = args[0].clone();
    let wd = args[1].clone();

    env::set_current_dir(&wd).expect("couldn't change working directory");

    let ingress = UdpSocket::bind(&addr).expect("couldn't bind to address");
    let mut rng = rand::thread_rng();

    loop {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (n_bytes, src_addr) = ingress.recv_from(&mut buf).expect("didn't receive data");

        let filled_buf = &mut buf[..n_bytes];
        let mut op = [0; 2];
        op.copy_from_slice(&filled_buf[..2]);
        let op = u16::from_le_bytes(op);
        let op = op.try_into().unwrap();

        let port: u16 = rng.gen_range(1001, 65535);
        let addr = format!("127.0.0.1:{}", port);
        let socket = UdpSocket::bind(&addr).expect("couldn't bind to address");
        socket.connect(src_addr).unwrap();

        match op {
            Opcode::Rrq => {
                let req: Packet<Rq> = Packet::try_from(filled_buf.to_vec()).unwrap();
                let filename = req.body.filename;
                let file = OpenOptions::new()
                    .read(true)
                    .open(filename).unwrap();
                let conn = Connection::<Put<File>>::new(socket, file);
                conn.put().unwrap();
            },
            Opcode::Wrq => {
                let req: Packet<Rq> = Packet::try_from(filled_buf.to_vec()).unwrap();
                let filename = req.body.filename;
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(filename).unwrap();
                
                let ack: Vec<u8> = Packet::<Ack>::new(0).into();
                socket.send(&ack[..]).unwrap();

                let conn = Connection::<Get<File>>::new(socket, file);
                conn.get().unwrap();
            },
            _ => continue,
        }
    }
}
