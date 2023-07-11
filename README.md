# Framous

This package is inspired by the `codec` module from [`tokio::util`] but, unlike tokio,
is designed to work with non-async code.

The intended use case for this crate is when you need to send and receive frames of
data via some add-hoc byte-orientated protocol, usually but not necessarily, over TCP.

- It supports the sending of user-defined message structures by encoding them to a
byte-orientated frame through a user-defined `Encoder`.

- Conversely, it supports the receiving of byte-oriented frames and decoding them through
a user-defined `Decoder` into messages as understood by the application.

This is a low-dependency, light-weight crate.

[![MIT licensed][mit-badge]][mit-url]
[![Crate](https://img.shields.io/crates/v/framous.svg)](https://crates.io/crates/framous)

[`tokio::util`]: https://docs.rs/tokio-util/latest/tokio_util/
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/tokio-rs/tokio/blob/master/LICENSE