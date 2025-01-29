use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor};

#[derive(Debug, PartialEq)]
pub struct Page {
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum PageDecodeError {
    IoError(io::Error),
    InvalidFormat,
}

impl From<io::Error> for PageDecodeError {
    fn from(error: io::Error) -> Self {
        PageDecodeError::IoError(error)
    }
}

impl Page {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, PageDecodeError> {
        if offset + 4 > self.data.len() {
            return Err(PageDecodeError::InvalidFormat);
        }
        let mut cursor = Cursor::new(&self.data[offset..]);
        Ok(cursor.read_u32::<BigEndian>()?)
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) -> Result<(), PageDecodeError> {
        if offset + 4 > self.data.len() {
            return Err(PageDecodeError::InvalidFormat);
        }
        let mut cursor = Cursor::new(&mut self.data[offset..]);
        cursor.write_u32::<BigEndian>(value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_u32() {
        let mut page = Page::new(vec![0; 8]);
        page.write_u32(0, 42).unwrap();
        assert_eq!(page.read_u32(0).unwrap(), 42);
    }

    #[test]
    fn test_invalid_offset() {
        let page = Page::new(vec![0; 4]);
        assert!(matches!(
            page.read_u32(2),
            Err(PageDecodeError::InvalidFormat)
        ));
    }
}
