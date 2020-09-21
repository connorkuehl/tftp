use std::env;

use tftp::Server;

fn main() {
    let mut args = env::args().skip(1);
    let addr = args.next().unwrap();
    let wd = args.next().unwrap();

    let server = Server::new(addr, wd).unwrap();

    while let Ok(h) = server.serve() {
        let _ = h.handle();
    }
}
