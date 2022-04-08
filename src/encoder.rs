use crate::bencode::Bencode;
use std::io::Write;

pub struct Encoder<W: Write> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, _decoded: Bencode) {
        todo!()
    }
}
