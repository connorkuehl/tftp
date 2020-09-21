use std::env;
use std::fs::File;
use std::io::{Result, Write};
use std::net::ToSocketAddrs;
use std::path::Path;

use tftp::packet::Mode;
use tftp::{Client, ConnectTo};

fn put<T: AsRef<Path>, A: ToSocketAddrs>(src: T, client: Client<ConnectTo<A>>) {
    let target = src
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let source = File::open(src).unwrap();

    client.put(target, Mode::NetAscii, source).unwrap();
}

fn get<T: AsRef<str>, W: Write, A: ToSocketAddrs>(
    file: T,
    client: Client<ConnectTo<A>>,
    write: W,
) -> Result<W> {
    client.get(file, Mode::NetAscii, write)
}

fn main() {
    let mut args = env::args().skip(1);
    let server = args.next().unwrap();
    let verb = args.next().unwrap();
    let file = args.next().unwrap();

    let client = Client::new()
        .map(|c| c.connect_to(server).unwrap())
        .unwrap();

    match verb.as_str() {
        "get" => {
            let _ = get(file, client, std::io::stdout()).unwrap();
        }
        "put" => put(file, client),
        _ => panic!("unknown verb"),
    }
}
