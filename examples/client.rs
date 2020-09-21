use std::env;
use std::net::UdpSocket;
use std::process;

use rand::Rng;

use tftp::*;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: ./client [get|put] filename");
        process::exit(1);
    }

    let verb = args[0].clone();
    let filename = args[1].clone();

    let mut rng = rand::thread_rng();
    let port: u16 = rng.gen_range(1001, 65535);
    let addr = format!("127.0.0.1:{}", port);
    let socket = UdpSocket::bind(&addr).expect("couldn't bind to address");

    match verb.as_str() {
        "get" => {
            let req: Packet<Rrq> = Packet::new(filename, Mode::NetAscii);
        },
        "put" => {
        }
        _ => panic!("invalid verb");
    }
}
