use super::page::{Page, PageDecodeError};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PageIOError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Decode error: {0}")]
    DecodeError(#[from] PageDecodeError),

    #[error("Page {0} not found")]
    PageNotFound(u64),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub struct PageIO {
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl PageIO {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, PageIOError> {
        let reader_file = File::open(&db_path)?;
        let writer_file = OpenOptions::new().write(true).create(true).open(&db_path)?;

        let reader = BufReader::new(reader_file.try_clone()?);
        let writer = BufWriter::new(writer_file.try_clone()?);

        Ok(Self { reader, writer })
    }

    pub fn read_page(&mut self, page_id: u64, page_size: usize) -> Result<Page, PageIOError> {
        let mut buffer = vec![0; page_size];
        let offset = page_id as u64 * page_size as u64;

        // Seek to position
        self.reader.seek(SeekFrom::Start(offset))?;

        // Try to read the exact amount
        match self.reader.read_exact(&mut buffer) {
            Ok(_) => Ok(Page::new(buffer)),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                Err(PageIOError::PageNotFound(page_id))
            }
            Err(e) => Err(PageIOError::IoError(e)),
        }
    }

    pub fn write_page(
        &mut self,
        page_id: u64,
        page_size: usize,
        page: &Page,
    ) -> Result<(), PageIOError> {
        let offset = page_id as u64 * page_size as u64;
        self.writer.seek(SeekFrom::Start(offset))?;
        self.writer.write_all(page.as_bytes())?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), PageIOError> {
        self.writer.flush()?; // Add this line to flush the buffer
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_page_io() -> (NamedTempFile, usize, PageIO) {
        let temp_file = NamedTempFile::new().unwrap();
        let page_size: usize = 128; // Smaller page size for testing
        let page_io = PageIO::new(temp_file.path()).unwrap();
        (temp_file, page_size, page_io)
    }

    #[test]
    fn test_write_and_read_page() {
        let (_temp, page_size, mut page_io) = setup_test_page_io();
        let write_data = vec![42u8; page_size as usize];
        page_io
            .write_page(0, page_size, &Page::new(write_data.clone()))
            .unwrap();
        page_io.flush().unwrap();
        let read_page = page_io.read_page(0, page_size).unwrap();
        assert_eq!(write_data, read_page.as_bytes());
    }

    #[test]
    fn test_read_nonexistent_page() {
        let (_temp, page_size, mut page_io) = setup_test_page_io();
        let result = page_io.read_page(999, page_size);
        assert!(matches!(result, Err(PageIOError::PageNotFound(999))));
    }

    #[test]
    fn test_invalid_buffer_size() {
        let (_temp, page_size, mut page_io) = setup_test_page_io();
        let result = page_io.read_page(0, page_size);
        assert!(matches!(result, Err(PageIOError::PageNotFound(0))));
    }
}
