/*
# Framous

This package is inspired by the `codec` module from [`tokio::util`] but, unlike tokio,
is designed to work with non-async code.

The intended use case for this crate is when you need to send and receive frames of
data via some add-hoc byte-orientated protocol, usually but not necessarily, over TCP.

- It supports the sending of user-defined message structures by encoding them to a
byte-orientated frame through a user-defined `Encoder`.

- Conversely, it supports the receiving of a byte-oriented frames and decoding then through
a user-defined `Decoder` into messages as understood by the application.


[`tokio::util`]: https://docs.rs/tokio-util/latest/tokio_util/
*/

pub mod codec;
pub mod framed;

pub use codec::{Decoder, Encoder};
pub use framed::{Framed, FramedRead, FramedReader, FramedWrite, FramedWriter};
