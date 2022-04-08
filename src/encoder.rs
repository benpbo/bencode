use crate::bencode::Bencode;
use std::io::Write;

pub type Result<T> = std::io::Result<T>;

pub struct Encoder<W: Write> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, decoded: Bencode) -> Result<()> {
        match decoded {
            Bencode::Integer(number) => self.encode_number(number),
            Bencode::String(bytes) => self.encode_string(&bytes),
            Bencode::List(_list) => todo!(),
            Bencode::Dictionary(_dictionary) => todo!(),
        }
    }

    fn encode_number(&mut self, number: i64) -> Result<()> {
        write!(&mut self.writer, "i{}e", number)
    }

    fn encode_string(&mut self, bytes: &[u8]) -> Result<()> {
        write!(&mut self.writer, "{}:", bytes.len()).and(self.writer.write_all(bytes))
    }
}
