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

/// Configures if and how we should retransmit packets if we don't get a response
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RetransmissionConfig {
    /// Do not retransmit packets. Just error out.
    NoRetransmission,

    /// Retransmit packets indefinitely
    ForeverAfter {
        /// How long should we wait before retransmitting?
        timeout: std::time::Duration,
    },

    /// Retransmit packets a limited amount of times
    NTimesAfter {
        /// How long should we wait before retransmitting?
        timeout: std::time::Duration,

        /// How many times should we retransmit?
        limit: std::num::NonZeroUsize,
    },
}

impl Default for RetransmissionConfig {
    fn default() -> Self {
        Self::NoRetransmission
    }
}

// Adapts the new, enum-based, representation to what `UdpSocket::set_read_timeout` and `Connection::new` want
impl RetransmissionConfig {
    fn timeout(&self) -> Option<&std::time::Duration> {
        match self {
            Self::NoRetransmission => None,
            Self::ForeverAfter { timeout } | Self::NTimesAfter { timeout, .. } => Some(timeout),
        }
    }

    fn max_retransmissions(&self) -> Option<usize> {
        match self {
            Self::NoRetransmission => Some(0),
            Self::ForeverAfter { .. } => None,
            Self::NTimesAfter { limit, .. } => Some(limit.get()),
        }
    }
}

mod bytes;
pub mod client;
mod connection;
pub mod packet;
mod server;

pub use client::{Client, ConnectTo};
pub use server::{Handler, Server};
