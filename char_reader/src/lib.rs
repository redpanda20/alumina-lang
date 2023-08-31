

use std::io;
use std::str;
#[derive(Debug)]
pub enum CharReaderError {
    IOError(std::io::Error),
    ReachedEOF,

}
impl From<std::io::Error> for CharReaderError {
    fn from(err: std::io::Error) -> Self {
        CharReaderError::IOError(err)
    }
}
impl From<str::Utf8Error> for CharReaderError {
    fn from(_: str::Utf8Error) -> Self {
        CharReaderError::IOError(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF8"))
    }
}

pub struct CharReader<R: io::Read> {
	inner: R,
	buf: Box<[u8]>,
    pos: usize, // Left side
    filled: usize // Right side
}

const DEFAULT_BUF_SIZE: usize = 5_000;

impl <R: io::Read> CharReader<R> {
	pub fn new(inner: R) -> Self {
		CharReader::with_capacity(DEFAULT_BUF_SIZE, inner)
	}

	pub fn with_capacity(capacity: usize, inner: R) -> Self {
        let buf = vec![0; capacity].into_boxed_slice();
		Self { inner, buf, pos: 0, filled: 0 }
	}


    pub fn next_char(&mut self) -> Result<char, CharReaderError> {
        // Enough bytes in the current buffer
        if self.pos + 4 >= self.filled {

            // Shift buffer to the start
            self.buf.copy_within(self.pos .. self.filled, 0);
            self.filled -= self.pos;

            self.filled += self.inner.read(&mut self.buf)?;
            if self.filled == 0 {
                return Err(CharReaderError::ReachedEOF);
            }
            self.pos = 0;
        }
        
        let char = str::from_utf8(&self.buf[self.pos..self.filled])?
            .chars().next().expect("&str must be at least length 1");

        self.pos += char.len_utf8();
        Ok(char)
    }

}

impl <R: io::Read> Iterator for CharReader<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_char().ok()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn buildable() {
        let bytes: &[u8] = &[0, 0, 0, 0, 0];
        let char_reader = CharReader::new(bytes);

        assert_eq!(char_reader.buf.len(), 5000);
    }

    #[test]
    fn file_read() {
        let file = File::open("../test.alo")
            .expect("Unable to open file");

        let mut reader = CharReader::new(file);
        let mut string = String::new();

        loop {
            match reader.next_char() {
                Ok(ch) => string.push(ch),
                Err(err) if matches!(err, CharReaderError::ReachedEOF) => break,
                Err(err) => assert!(false, "Reader failed unexpectedly: {:?}", err)
            }
        }

        println!("Output:\n{}", string)
    }
}
