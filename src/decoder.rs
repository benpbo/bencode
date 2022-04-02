use crate::bencode::Bencode;
use std::io::Read;

pub enum DecoderError {
    EOF,
    IO,
}

pub type DecoderResult<T> = Result<T, DecoderError>;

pub struct Decoder<R: Read> {
    reader: R,
    buffer: [u8; 1],
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: [0u8],
        }
    }

    pub fn decode(&mut self) -> DecoderResult<Bencode> {
        let current = self.advance()?;
        match current {
            b'i' => self.decode_integer(),
            d @ b'0'..=b'9' => self.decode_string(),
            b'l' => self.decode_list(),
            b'd' => self.decode_dictionary(),
            c => todo!(),
        }
    }

    fn decode_integer(&mut self) -> Result<Bencode, DecoderError> {
        todo!()
    }

    fn decode_string(&self) -> Result<Bencode, DecoderError> {
        todo!()
    }

    fn decode_list(&self) -> Result<Bencode, DecoderError> {
        todo!()
    }

    fn decode_dictionary(&self) -> Result<Bencode, DecoderError> {
        todo!()
    }

    fn advance(&mut self) -> Result<u8, DecoderError> {
        let amount_read = self
            .reader
            .read(&mut self.buffer)
            .map_err(|_| DecoderError::IO)?;

        if amount_read == 0 {
            Err(DecoderError::EOF)
        } else {
            Ok(self.buffer[0])
        }
    }
}
