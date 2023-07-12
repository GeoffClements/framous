//! These are the structs that you instantiate in your application.
//! 
//! In order to used framed packets you can use:
//! * `FramedRead` to just decode packets into your own message type
//! * `FramedWrite` if you just want to send encoded messages
//! * `Framed` to do both
//! 
//! The `Framed*` structs take ownership the underlying `Read` or `Write`
//! object. This means that they will work in multi-threaded applications
//! but this presents a problem in the very likely case when the `Read`
//! and `Write` object is the same `TcpStream`. In this case use the
//! `try_clone` method on the stream to get a thread-safe handle.
//! 
//! A `Framed` type is useful in single-threaded applications where you
//! just want a single object to encode and decode. For multi-threaded
//! applications you can use `FramedRead` in one thread and `FramedWrite`
//! in another.
//! 

use std::io::{self, Read, Write};

use bytes::BytesMut;

use crate::{Decoder, Encoder};

const INITIAL_CAPACITY: usize = 8 * 1024;

pub struct FramedRead<R, D> {
    inner: R,
    decoder: D,
    buf: BytesMut,
}

impl<R, D> FramedRead<R, D> {
    pub fn new(inner: R, decoder: D) -> Self {
        Self {
            inner,
            decoder,
            buf: BytesMut::with_capacity(INITIAL_CAPACITY),
        }
    }
}

pub struct FramedWrite<W, E> {
    inner: W,
    encoder: E,
}

impl<W, E> FramedWrite<W, E> {
    pub fn new(inner: W, encoder: E) -> Self {
        Self { inner, encoder }
    }
}

pub struct Framed<R, W, D, E> {
    reader: FramedRead<R, D>,
    writer: FramedWrite<W, E>,
}

impl<R, W, D, E, I> Framed<R, W, D, E>
where
    R: Read,
    W: Write,
    D: Decoder<Item = I, Error = io::Error>,
    E: Encoder<I>,
{
    pub fn new(reader: R, writer: W, decoder: D, encoder: E) -> Framed<R, W, D, E> {
        Framed {
            reader: FramedRead::new(reader, decoder),
            writer: FramedWrite::new(writer, encoder),
        }
    }

    pub fn split(self) -> (FramedRead<R, D>, FramedWrite<W, E>) {
        (self.reader, self.writer)
    }
}

/// Trait for reading frames
pub trait FramedReader<I> {
    fn framed_read(&mut self) -> io::Result<I>;
}

/// Trait for writing frames
pub trait FramedWriter<I> {
    fn framed_write(&mut self, item: I) -> io::Result<()>;
}

impl<I, R, D> FramedReader<I> for FramedRead<R, D>
where
    R: Read,
    D: Decoder<Item = I, Error = io::Error>,
{
    fn framed_read(&mut self) -> io::Result<I> {
        let mut src = [0u8; INITIAL_CAPACITY];
        loop {
            let bytes_read = self.inner.read(&mut src)?;
            self.buf.extend_from_slice(&src[..bytes_read]);
            match self.decoder.decode(&mut self.buf) {
                Ok(Some(item)) => return Ok(item),
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

impl<I, W, E> FramedWriter<I> for FramedWrite<W, E>
where
    W: Write,
    E: Encoder<I, Error = io::Error>,
{
    fn framed_write(&mut self, item: I) -> io::Result<()> {
        let mut dst = BytesMut::with_capacity(INITIAL_CAPACITY);
        self.encoder.encode(item, &mut dst)?;
        self.inner.write(&dst[..])?;
        self.inner.flush()
    }
}

impl<R, W, D, E, I> FramedReader<I> for Framed<R, W, D, E>
where
    R: Read,
    D: Decoder<Item = I, Error = io::Error>,
{
    fn framed_read(&mut self) -> io::Result<I> {
        self.reader.framed_read()
    }
}

impl<R, W, D, E, I> FramedWriter<I> for Framed<R, W, D, E>
where
    W: Write,
    E: Encoder<I, Error = io::Error>,
{
    fn framed_write(&mut self, item: I) -> io::Result<()> {
        self.writer.framed_write(item)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BufMut};
    use socket_server_mocker::{
        server_mocker::ServerMocker,
        server_mocker_instruction::{ServerMockerInstruction, ServerMockerInstructionsList},
        tcp_server_mocker::TcpServerMocker,
    };

    use std::net::TcpStream;

    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestMsg {
        U8(u8),
        U16(u16),
        Unrecognised,
    }

    struct TestCodec;

    impl Decoder for TestCodec {
        type Item = TestMsg;
        type Error = io::Error;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            if src.len() < 1 {
                return Ok(None);
            }

            let msg_len = src[0];
            if src.len() < msg_len as usize + 1 {
                return Ok(None);
            }

            src.advance(1);
            let mut msg = src.split_to(msg_len as usize);

            match msg_len {
                1 => {
                    let val = msg.get_u8();
                    Ok(Some(TestMsg::U8(val)))
                }
                2 => {
                    let val = msg.get_u16();
                    Ok(Some(TestMsg::U16(val)))
                }
                _ => Ok(Some(TestMsg::Unrecognised)),
            }
        }
    }

    impl Encoder<TestMsg> for TestCodec {
        type Error = io::Error;

        fn encode(&mut self, item: TestMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
            match item {
                TestMsg::U8(val) => {
                    dst.put_u8(1);
                    dst.put_u8(val);
                }
                TestMsg::U16(val) => {
                    dst.put_u8(2);
                    dst.put_u16(val);
                }
                TestMsg::Unrecognised => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, ""))
                }
            }
            Ok(())
        }
    }

    #[test]
    fn reader_valid_u8() {
        let r = vec![1u8, 128];
        let mut framed = FramedRead::new(&r[..], TestCodec);
        let data = framed.framed_read().unwrap();
        assert_eq!(data, TestMsg::U8(128));
    }

    #[test]
    fn reader_valid_u16() {
        let r = vec![2u8, 1, 128];
        let mut framed = FramedRead::new(&r[..], TestCodec);
        let data = framed.framed_read().unwrap();
        assert_eq!(data, TestMsg::U16(2u16.pow(8) + 128));
    }

    #[test]
    fn read_unrecognised() {
        let r = vec![3u8, 1, 128, 0];
        let mut framed = FramedRead::new(&r[..], TestCodec);
        let data = framed.framed_read().unwrap();
        assert_eq!(data, TestMsg::Unrecognised);
    }

    #[test]
    fn write_valid_u8() {
        let mut buf = vec![];
        let mut framed = FramedWrite::new(&mut buf, TestCodec);
        framed.framed_write(TestMsg::U8(12)).ok();
        assert_eq!(buf, vec![1, 12]);
    }

    #[test]
    fn write_valid_u16() {
        let mut buf = vec![];
        let mut framed = FramedWrite::new(&mut buf, TestCodec);
        framed.framed_write(TestMsg::U16(1234)).ok();
        assert_eq!(buf, vec![2, 4, 210]);
    }

    #[test]
    fn invalid_write() {
        let mut buf = vec![];
        let mut framed = FramedWrite::new(&mut buf, TestCodec);
        let response = framed.framed_write(TestMsg::Unrecognised);
        assert!(response.is_err());
    }

    #[test]
    fn framed() {
        let reader = vec![2u8, 4, 210];
        let mut writer = vec![];
        let mut framed = Framed::new(&reader[..], &mut writer, TestCodec, TestCodec);
        framed.framed_write(TestMsg::U16(1234)).ok();
        let msg = framed.framed_read().unwrap();
        assert_eq!(writer, vec![2u8, 4, 210]);
        assert_eq!(msg, TestMsg::U16(1234));
    }

    #[test]
    fn framed_over_tcp() {
        let test_buf = vec![2u8, 25, 143];

        let tcp_server_mocker = TcpServerMocker::new(35642).unwrap();
        let rx = TcpStream::connect("127.0.0.1:35642").unwrap();
        let tx = rx.try_clone().unwrap();

        let mut framed = Framed::new(rx, tx, TestCodec, TestCodec);

        tcp_server_mocker
            .add_mock_instructions_list(ServerMockerInstructionsList::new_with_instructions(
                [
                    ServerMockerInstruction::ReceiveMessage,
                    ServerMockerInstruction::SendMessage(Vec::from(test_buf.clone())),
                ]
                .as_slice(),
            ))
            .unwrap();

        framed.framed_write(TestMsg::U16(6543)).ok();
        let response = framed.framed_read().unwrap();

        assert_eq!(response, TestMsg::U16(6543));
        assert_eq!(tcp_server_mocker.pop_received_message().unwrap(), test_buf);
    }
}
