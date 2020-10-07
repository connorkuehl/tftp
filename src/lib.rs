//! The `tftp` crate provides implementations for the following components of
//! the Trivial File Transfer Protocol (RFC 1350):
//!
//! * The protocol (types that represent TFTP packets as well as types that
//!   can participate in the TFTP flow for reading or writing files with
//!   TFTP.
//! * A client
//! * A server
//!
//! For more information, please see [THE TFTP PROTOCOL (REVISION 2)](
//! https://tools.ietf.org/html/rfc1350).
//!
//! ## Try it out
//!
//! In one terminal window, start up the server:
//!
//! ```console
//! $ cargo run --example server 0.0.0.0:6655 ./artifacts
//! Serving Trivial File Transfer Protocol (TFTP) @ 0.0.0.0:6655
//! ```
//!
//! Then in another window:
//!
//! ```console
//! $ cargo run --example client 0.0.0.0:6655 get alice-in-wonderland.txt
//! [..]
//! The Project Gutenberg EBook of Alice’s Adventures in Wonderland, by Lewis
//! Carroll This eBook is for the use of anyone anywhere at no cost and with
//! almost no restrictions whatsoever.  You may copy it, give it away or
//! re-use it under the terms of the Project Gutenberg License included
//! with this eBook or online at www.gutenberg.org
//!
//!
//! Title: Alice’s Adventures in Wonderland
//! [..]
//! ```
//!
//! Alternatively, you may connect to your server from another host.

#![deny(missing_docs)]

/// POD struct representing the configuration of the retransmission of packets
// NB: this is a struct so that you can only specify max_retransmissions if you specify a time :>
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RetransmissionConfig {
    /// How long should we wait for a reply before retransmitting the last packet?
    timeout: std::time::Duration,

    /// How many times should we retransmit the last packet?
    ///
    /// Note that this is the number of *retransmissions*, not transmissions, so
    /// this means that setting this to `Some(0)` means that the packet will still be
    /// sent once.
    ///
    /// If this is set to `None`, the packet will be retransmitted indefinitely.
    max_retransmissions: Option<usize>,
}

mod bytes;
pub mod client;
mod connection;
pub mod packet;
mod server;

pub use client::{Client, ConnectTo};
pub use server::{Handler, Server};
