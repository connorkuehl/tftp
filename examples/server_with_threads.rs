use std::env;
use std::thread;

use tftp::Server;

fn main() {
    let mut args = env::args().skip(1);
    let addr = args.next().unwrap();
    let wd = args.next().unwrap();

    let server = Server::new(addr.clone(), wd, None).unwrap();
    println!("Serving Trivial File Transfer Protocol (TFTP) @ {}", addr);

    while let Ok(h) = server.serve() {
        print!("Handling request...");

        thread::spawn(|| match h.handle() {
            Ok(()) => println!("OK"),
            Err(e) => println!("FAIL: {:?}", e),
        });
    }
}
