use crate::bencode::Bencode;
use std::{collections::BTreeMap, io::Write};

pub type Result<T> = std::io::Result<T>;

pub struct Encoder<W: Write> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, decoded: &Bencode) -> Result<()> {
        match decoded {
            Bencode::Integer(number) => self.encode_number(number),
            Bencode::String(bytes) => self.encode_string(bytes),
            Bencode::List(list) => self.encode_list(list),
            Bencode::Dictionary(dictionary) => self.encode_dictionary(dictionary),
        }
    }

    fn encode_number(&mut self, number: &i64) -> Result<()> {
        write!(&mut self.writer, "i{}e", number)
    }

    fn encode_string(&mut self, bytes: &[u8]) -> Result<()> {
        write!(&mut self.writer, "{}:", bytes.len()).and(self.writer.write_all(bytes))
    }

    fn encode_list(&mut self, list: &[Bencode]) -> Result<()> {
        self.writer
            .write_all(b"l")
            .and(list.iter().try_for_each(|decoded| self.encode(decoded)))
            .and(self.writer.write_all(b"e"))
    }

    fn encode_dictionary(&mut self, dictionary: &BTreeMap<String, Bencode>) -> Result<()> {
        self.writer
            .write_all(b"d")
            .and(dictionary.into_iter().try_for_each(|(key, value)| {
                self.encode_string(key.as_bytes()).and(self.encode(value))
            }))
            .and(self.writer.write_all(b"e"))
    }
}

#[cfg(test)]
mod tests {
    use super::Encoder;

    fn create_encoder() -> Encoder<Vec<u8>> {
        Encoder::new(vec![])
    }
}
