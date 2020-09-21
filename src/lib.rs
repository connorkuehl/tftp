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

mod bytes;
mod client;
mod connection;
pub mod packet;
mod server;

pub use client::{Client, ConnectTo};
pub use server::{Handler, Server};
