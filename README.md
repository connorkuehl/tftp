# tftp

The `tftp` crate provides implementations for the following components of
the Trivial File Transfer Protocol (RFC 1350):

* The protocol (types that represent TFTP packets as well as types that
  can participate in the TFTP flow for reading or writing files with
  TFTP.
* A client
* A server

For more information, please see [THE TFTP PROTOCOL (REVISION 2)](
https://tools.ietf.org/html/rfc1350).

### Try it out

In one terminal window, start up the server:

```console
$ cargo run --example server 0.0.0.0:6655 ./artifacts
Serving Trivial File Transfer Protocol (TFTP) @ 0.0.0.0:6655
```

Then in another window:

```console
$ cargo run --example client 0.0.0.0:6655 get alice-in-wonderland.txt
[..]
The Project Gutenberg EBook of Alice’s Adventures in Wonderland, by Lewis
Carroll This eBook is for the use of anyone anywhere at no cost and with
almost no restrictions whatsoever.  You may copy it, give it away or
re-use it under the terms of the Project Gutenberg License included
with this eBook or online at www.gutenberg.org


Title: Alice’s Adventures in Wonderland
[..]
```

Alternatively, you may connect to your server from another host.

License: Apache-2.0
