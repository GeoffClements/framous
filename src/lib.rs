//! # Framous
//! This package is inspired by the `codec` module from [`tokio::util`] but, unlike tokio,
//! is designed to work with non-async code.
//! 
//! The intended use case for this crate is when you need to send and receive frames of
//! data via some add-hoc byte-orientated protocol, usually but not necessarily, over TCP.
//! 
//! - It supports the sending of user-defined message structures by encoding them to a
//! byte-orientated frame through a user-defined `Encoder`.
//! 
//! - Conversely, it supports the receiving of a byte-oriented frames and decoding then through
//! a user-defined `Decoder` into messages as understood by the application.
//! 
//! [`tokio::util`]: https://docs.rs/tokio-util/latest/tokio_util/
//! 
//! Typical usage:
//! ```no_run
//! 
//! enum MyMessage {
//!     msg1,
//!     msg2,
//! }
//! 
//! struct MyCodec;
//! 
//! impl Decoder for MyCodec {
//!     type Item = MyMessage;
//!     type Error = io::Error;
//! 
//!     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//!         // decode code
//!         // see codec::Decoder
//!     }
//! }
//! 
//! impl Encoder<MyMessage> for TestCodec {
//!     type Error = io::Error;
//! 
//!     fn encode(&mut self, item: MyMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
//!         // encode code
//!         // see codec::Encoder
//!     }
//! }
//! 
//! let cx = TcpStream::connect("127.0.0.1:35642").unwrap();
//! let mut rx = FramedRead(cx.try_clone()?, MyCodec);
//! let mut tx = FramedWrite(cx, MyCodec);
//! 
//! // Send a message
//! tx.framed_write(MyMessage::msg1).unwrap();
//! 
//! // Block on waiting for a message
//! let msg = rx.framed_read().unwrap();

pub mod codec;
pub mod framed;

pub use codec::{Decoder, Encoder};
pub use framed::{Framed, FramedRead, FramedReader, FramedWrite, FramedWriter};
