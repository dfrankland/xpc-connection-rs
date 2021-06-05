# xpc-connection-rs

[![shield]][crate]
[![sys shield]][sys crate]

[crate]: https://crates.io/crates/xpc-connection
[shield]: https://img.shields.io/crates/v/xpc-connection?label=xpc-connection
[sys crate]: https://crates.io/crates/xpc-connection-sys
[sys shield]: https://img.shields.io/crates/v/xpc-connection-sys?label=xpc-connection-sys

XPC connection bindings for Rust.

## What is XPC?

A low-level (libSystem) interprocess communication mechanism that is based on
serialized property lists for Mac OS. Read more at the
[Apple Developer website][apple developer].

[apple developer]: https://developer.apple.com/documentation/xpc

## Supported Data Types

*   `array`: `Vec<Message>`
*   `data`: `Vec<u8>`
*   `dictionary`: `HashMap<String, Message>`
*   `error`: `MessageError`
*   `fd`: `RawFd`
*   `int64`: `i64`
*   `string`: `String`
*   `uint64`: `u64`
*   `uuid`: `Vec<u8>`

## Yet to Be Supported Data Types

*   `activity`
*   `bool`
*   `date`
*   `double`
*   `endpoint`
*   `null`
*   `shmem`
