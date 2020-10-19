# Capturing TFTP packets with Wireshark

To inspect packets exchanged between a TFTP server and client, it is recommended
to use [Wireshark](https://www.wireshark.org/). Wireshark is a very powerful
general purpose packet capture tool, used as standard across the IT industry.

Below is a quick-start guide which demonstrates how to capture some TFTP traffic
between a test client and server. It should be possible to capture TFTP
packets sent in a non-test environment using a similar approach with a few 
small changes, such as the interface or the port numbers. The guide is written
with Ubuntu desktop in mind but should apply to Windows and macOS with minor
differences.

## Instructions

1. Clone this repository and ensure you can run `cargo test` on it
2. Open Wireshark ([download here](https://www.wireshark.org/#download))
3. You should be presented with a list of interfaces - choose `loopback`
    - If using Windows and you cannot capture packets on `loopback`
      then [check this guide](https://wiki.wireshark.org/CaptureSetup/Loopback)
4. Click the blue shark fin ('Start' or 'Start a new live capture')
5. Return to the command line in this project and run `cargo test`
    - A series of packets should appear in the Wireshark window
6. You can choose to stop the capture after `cargo test` has finished 
   (click the red square in Wireshark). If you do not, any new packets
   transmitted on `loopback` will be added to the list in Wireshark.
   
## Packets

You should have captured a series of UDP packets. The tests run TFTP servers 
and clients on ports which are randomly selected each time `cargo test` is run.
The ports will thus change each time you repeat this capture.

As for the structure of the packets, the full TFTP specification is 
[available here](https://tools.ietf.org/html/rfc1350). In brief each packet
will contain some non-TFTP-specific header information (such as the 
IP header, UDP header, etc...) followed by the TFTP data.
You can extract the TFTP specific data by

1. Selecting the packet of interest from the list of captured packets
2. Right clicking the 'Data' section of the packet information
    - Left clicking will highlight the corresponding bytes in the pane below
3. Choosing Copy > Bytes > Offset Hex Text
4. Pasting this into an editor program like gedit or notepad++

For example, here is some TFTP error data sent in one of the tests in hex
followed by ASCII where appropriate.

```
0000   00 05 00 04 49 6c 6c 65 67 61 6c 20 54 46 54 50  ....Illegal TFTP
0010   20 6f 70 65 72 61 74 69 6f 6e 00                  operation.
```

- The first two bytes are the OPCODE, `00 05` is an error
- The following data is OPCODE specific
    - For an error packet the following two bytes are the `ErrorCode`, 
      `00 04` is an `Illegal TFTP operation.` error
    - After the error code is a human readable error message, ASCII encoded
- A null byte `00` terminates the frame
    
Here is a read request packet sent during the tests, in a similar format.

```
0000   00 01 61 6c 69 63 65 2d 69 6e 2d 77 6f 6e 64 65  ..alice-in-wonde
0010   72 6c 61 6e 64 2e 74 78 74 00 6e 65 74 61 73 63  rland.txt.netasc
0020   69 69 00                                         ii.
```

- `00 01` is the OPCODE for a read request (RRQ)
- The filename in ASCII `alice-in-wonderland.txt` immediately follows
- The filename is followed by a null byte `00` (second line, 10th pair)
- The _mode_ in ASCII `netascii` immediately follows
- A null byte `00` terminates the frame