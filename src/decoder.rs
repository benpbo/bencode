use crate::bencode::Bencode;
use std::io::Read;

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
    EOF,
    IO,
    NAN,
    EmptyNumber,
    IntegerOverflow,
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
        debug_assert_eq!(self.current(), b'i');

        // Empty integer: "ie"
        if self.advance()? == b'e' {
            return Err(DecoderError::EmptyNumber);
        }

        let sign = self.decode_integer_sign()?;
        let number = self.decode_number(sign)?;

        if self.current() == b'e' {
            Ok(Bencode::Integer(number))
        } else {
            Err(DecoderError::NAN)
        }
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

    fn decode_integer_sign(&mut self) -> DecoderResult<i64> {
        if self.current() == b'-' {
            self.advance()?;
            return Ok(-1);
        }

        Ok(1)
    }

    fn decode_number(&mut self, sign: i64) -> DecoderResult<i64> {
        let mut number: i64 = 0;
        while let Some(digit) = self.decode_digit() {
            number = number
                .checked_mul(10)
                .ok_or(DecoderError::IntegerOverflow)?;
            number = number
                .checked_add(digit * sign)
                .ok_or(DecoderError::IntegerOverflow)?;

            self.advance()?;
        }

        Ok(number)
    }

    fn decode_digit(&mut self) -> Option<i64> {
        (self.current() as char)
            .to_digit(10)
            .map(|digit| digit as i64)
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

    fn current(&mut self) -> u8 {
        self.buffer[0]
    }
}

#[cfg(test)]
mod tests {
    use super::Decoder;
    use crate::{bencode::Bencode, decoder::DecoderError};
    use std::io::Cursor;

    fn create_decoder(encoded: &[u8]) -> Decoder<Cursor<&[u8]>> {
        Decoder {
            reader: Cursor::new(&encoded[1..]),
            buffer: [encoded[0]],
        }
    }

    #[test]
    fn test_decode_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i123e");

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(123)));
    }

    #[test]
    fn test_decode_negative_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i-123e");

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(-123)));
    }

    #[test]
    fn test_decode_zero() {
        // Arrange
        let mut decoder = create_decoder(b"i0e");

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(0)));
    }

    #[test]
    fn test_decode_empty_integer() {
        // Arrange
        let mut decoder = create_decoder(b"ie");

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Err(DecoderError::EmptyNumber));
    }

    #[test]
    fn test_decode_nan_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i123a123e");

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Err(DecoderError::NAN));
    }

    #[test]
    fn test_decode_overflow_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i9223372036854775808e"); // i64::MAX + 1

        // Act
        let result = decoder.decode_integer();

        // Assert
        assert_eq!(result, Err(DecoderError::IntegerOverflow));
    }
}
