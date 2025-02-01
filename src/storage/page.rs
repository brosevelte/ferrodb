use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Page {
    data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum PageDecodeError {
    #[error("Invalid format: attempted to read beyond page bounds")]
    UnexpectedEof,

    #[error("Invalid page size: {0}")]
    InvalidPageSize(String),

    #[error("Unable to parse bytes into expected type")]
    InvalidBytes(#[from] io::Error),
}

impl Page {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn zeros(page_size: usize) -> Self {
        Self::new(vec![0; page_size])
    }

    pub fn full(value: u8, page_size: usize) -> Self {
        Self::new(vec![value; page_size])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, PageDecodeError> {
        if offset + 4 > self.data.len() {
            return Err(PageDecodeError::UnexpectedEof);
        }
        let mut cursor = Cursor::new(&self.data[offset..]);
        Ok(cursor.read_u32::<BigEndian>()?)
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) -> Result<(), PageDecodeError> {
        if offset + 4 > self.data.len() {
            return Err(PageDecodeError::UnexpectedEof);
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
            Err(PageDecodeError::UnexpectedEof)
        ));
    }

    #[test]
    fn test_from_size() {
        let page: Page = Page::zeros(8192);
        assert_eq!(page.as_bytes(), &vec![0; 8192]);
    }
}
