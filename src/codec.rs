//! Traits used by the user-defined decoders and encoders.
//! These are used with [`FramedRead`] and [`FramedWrite`]

use bytes::BytesMut;
use std::io;

/// The `Decoder` trait.
/// Objects that implement this trait take a `BytesMut` and return
/// an item whose type is defined as the associated type `Item`.
/// 
/// It is important that implementers deplete bytes from `src` when
/// they are used to form the returned `item`. This can be done
/// by using the `split*` methods and the `get*` and `put*` methods
/// which become available when you import the [`Buf`] and
/// [`BufMut`] traits.
/// 
/// There are three possibilities to return from this method:
/// - `Ok(Some(Item))` - in which case decoding is complete and the
/// user defined `item` is returned.
/// - `Ok(None)`- there is not enough data in `src` to decode the
/// message. When this is returned `read` is called again on the
/// underlying `Read` object.
/// - `Err()` - An error has occurred. This will close the underlying
/// `Read` object. If you want to indicate a protocol error it is better
/// to use the user defined `Item` to do this.
/// 
/// [`Buf`]: https://docs.rs/bytes/1.4.0/bytes/trait.Buf.html
/// [`BufMut`]: https://docs.rs/bytes/1.4.0/bytes/trait.BufMut.html
/// 
pub trait Decoder {
    type Item;
    type Error: From<io::Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;
}

 /// The `Encoder` trait.
 /// Objects that implement this trait take a user-defined `Item` and
 /// insert into the provided `BytesMut`.
pub trait Encoder<I> {
    type Error: From<io::Error>;

    fn encode(&mut self, item: I, dst: &mut BytesMut) -> Result<(), Self::Error>;
}
