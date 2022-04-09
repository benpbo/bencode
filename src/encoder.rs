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
    use crate::bencode::Bencode;

    use super::Encoder;

    fn create_encoder() -> Encoder<Vec<u8>> {
        Encoder::new(vec![])
    }

    #[test]
    fn test_encode_positive_integer() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::Integer(123));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"i123e");
    }

    #[test]
    fn test_encode_negative_integer() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::Integer(-123));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"i-123e");
    }

    #[test]
    fn test_encode_zero() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::Integer(0));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"i0e");
    }

    #[test]
    fn test_encode_ascii_string() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::String(b"spam".to_vec()));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"4:spam");
    }

    #[test]
    fn test_encode_raw_byte_string() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::String(b"\x00\x01\x02\x03".to_vec()));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"4:\x00\x01\x02\x03");
    }

    #[test]
    fn test_encode_empty_string() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::String(vec![]));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"0:");
    }

    #[test]
    fn test_encode_list() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::List(vec![
            Bencode::String(b"spam".to_vec()),
            Bencode::String(b"eggs".to_vec()),
        ]));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"l4:spam4:eggse");
    }

    #[test]
    fn test_encode_empty_list() {
        // Arrange
        let mut encoder = create_encoder();

        // Act
        let result = encoder.encode(&Bencode::List(vec![]));

        // Assert
        assert!(result.is_ok());
        assert_eq!(encoder.writer, b"le");
    }
}
