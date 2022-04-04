use crate::bencode::Bencode;
use std::io::{ErrorKind, Read};

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
    EOF,
    IO,
    NAN,
    EmptyNumber,
    IntegerOverflow,
}

impl From<std::io::Error> for DecoderError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            ErrorKind::UnexpectedEof => DecoderError::EOF,
            _ => DecoderError::IO,
        }
    }
}

pub type DecoderResult<T> = Result<T, DecoderError>;

pub struct Decoder<R: Read> {
    reader: R,
    current: u8,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, current: 0 }
    }

    pub fn decode(&mut self) -> DecoderResult<Bencode> {
        self.advance()?;
        self.decode_current()
    }

    fn decode_current(&mut self) -> DecoderResult<Bencode> {
        match self.current {
            b'i' => self.decode_integer(),
            b'0'..=b'9' => self.decode_string(),
            b'l' => self.decode_list(),
            b'd' => self.decode_dictionary(),
            _c => todo!(),
        }
    }

    fn decode_integer(&mut self) -> DecoderResult<Bencode> {
        debug_assert_eq!(self.current, b'i');

        // Empty integer: "ie"
        if self.advance()? == b'e' {
            return Err(DecoderError::EmptyNumber);
        }

        let sign = self.decode_integer_sign()?;
        let number = sign * self.decode_number()?;
        self.expect(b'e', DecoderError::NAN)?;

        Ok(Bencode::Integer(number))
    }

    fn decode_string(&mut self) -> DecoderResult<Bencode> {
        debug_assert!(self.current.is_ascii_digit());

        let length = self.decode_number()? as usize;
        self.expect(b':', DecoderError::NAN)?;
        let bytes = self.read_bytes(length)?;

        Ok(Bencode::String(bytes))
    }

    fn decode_list(&mut self) -> DecoderResult<Bencode> {
        debug_assert_eq!(self.current, b'l');

        let mut items = vec![];
        while self.advance()? != b'e' {
            items.push(self.decode_current()?);
        }

        Ok(Bencode::List(items))
    }

    fn decode_dictionary(&self) -> DecoderResult<Bencode> {
        todo!()
    }

    fn decode_integer_sign(&mut self) -> DecoderResult<i64> {
        if self.current == b'-' {
            self.advance()?;
            return Ok(-1);
        }

        Ok(1)
    }

    fn decode_number(&mut self) -> DecoderResult<i64> {
        let mut number: i64 = 0;
        while let Some(digit) = self.decode_digit() {
            number = number
                .checked_mul(10)
                .and_then(|number| number.checked_add(digit))
                .ok_or(DecoderError::IntegerOverflow)?;

            self.advance()?;
        }

        Ok(number)
    }

    fn decode_digit(&self) -> Option<i64> {
        (self.current as char)
            .to_digit(10)
            .map(|digit| digit as i64)
    }

    fn read_bytes(&mut self, amount: usize) -> DecoderResult<Vec<u8>> {
        let mut bytes = vec![0; amount];
        self.reader
            .read_exact(&mut bytes)
            .map_err(DecoderError::from)?;

        Ok(bytes)
    }

    fn expect(&self, expected: u8, error: DecoderError) -> DecoderResult<()> {
        if self.current != expected {
            return Err(error);
        }

        Ok(())
    }

    fn advance(&mut self) -> DecoderResult<u8> {
        let mut buffer = [0u8];
        let amount_read = self.reader.read(&mut buffer).map_err(DecoderError::from)?;

        if amount_read == 0 {
            return Err(DecoderError::EOF);
        }

        self.current = buffer[0];
        Ok(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::Decoder;
    use crate::{bencode::Bencode, decoder::DecoderError};
    use std::io::Cursor;

    fn create_decoder(encoded: &[u8]) -> Decoder<Cursor<&[u8]>> {
        Decoder::new(Cursor::new(encoded))
    }

    #[test]
    fn test_decode_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i123e");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(123)));
    }

    #[test]
    fn test_decode_negative_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i-123e");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(-123)));
    }

    #[test]
    fn test_decode_zero() {
        // Arrange
        let mut decoder = create_decoder(b"i0e");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::Integer(0)));
    }

    #[test]
    fn test_decode_empty_integer() {
        // Arrange
        let mut decoder = create_decoder(b"ie");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Err(DecoderError::EmptyNumber));
    }

    #[test]
    fn test_decode_nan_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i123a123e");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Err(DecoderError::NAN));
    }

    #[test]
    fn test_decode_overflow_integer() {
        // Arrange
        let mut decoder = create_decoder(b"i9223372036854775808e"); // i64::MAX + 1

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Err(DecoderError::IntegerOverflow));
    }

    #[test]
    fn test_decode_ascii_string() {
        // Arrange
        let mut decoder = create_decoder(b"4:spam");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::String(b"spam".to_vec())));
    }

    #[test]
    fn test_decode_raw_byte_string() {
        // Arrange
        let mut decoder = create_decoder(b"4:\x00\x01\x02\x03");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::String(vec![0x00, 0x01, 0x02, 0x03])));
    }

    #[test]
    fn test_decode_too_long_ascii_string() {
        // Arrange
        let mut decoder = create_decoder(b"4:spam+");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::String(b"spam".to_vec())));
    }

    #[test]
    fn test_decode_too_short_ascii_string() {
        // Arrange
        let mut decoder = create_decoder(b"4:spa");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Err(DecoderError::EOF));
    }

    #[test]
    fn test_decode_list() {
        // Arrange
        let mut decoder = create_decoder(b"l4:spam4:eggse");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(
            result,
            Ok(Bencode::List(vec![
                Bencode::String(b"spam".to_vec()),
                Bencode::String(b"eggs".to_vec())
            ]))
        );
    }

    #[test]
    fn test_decode_empty_list() {
        // Arrange
        let mut decoder = create_decoder(b"le");

        // Act
        let result = decoder.decode();

        // Assert
        assert_eq!(result, Ok(Bencode::List(vec![])));
    }
}
