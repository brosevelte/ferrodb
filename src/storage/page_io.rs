use super::page::{Page, PageDecodeError};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct PageIO {
    file: File,
    page_size: u64,
}

#[derive(Debug)]
pub enum PageError {
    IoError(io::Error),
    DecodeError(PageDecodeError),
    InvalidPageSize,
    PageNotFound,
}

impl From<io::Error> for PageError {
    fn from(error: io::Error) -> Self {
        PageError::IoError(error)
    }
}

impl From<PageDecodeError> for PageError {
    fn from(error: PageDecodeError) -> Self {
        PageError::DecodeError(error)
    }
}

impl PageIO {
    pub fn new(db_path: impl AsRef<Path>, page_size: u64) -> Result<Self, PageError> {
        // Create parent directories if they don't exist
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(db_path)?;

        Ok(Self { file, page_size })
    }

    pub fn read_page(&mut self, page_id: u64) -> Result<Page, PageError> {
        let mut buffer = vec![0; self.page_size as usize];
        self.file.seek(SeekFrom::Start(self.page_offset(page_id)))?;
        match self.file.read_exact(&mut buffer) {
            Ok(_) => Ok(Page::new(buffer)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Err(PageError::PageNotFound),
            Err(e) => Err(PageError::IoError(e)),
        }
    }

    pub fn write_page(&mut self, page_id: u64, page: &Page) -> Result<(), PageError> {
        let data = page.as_bytes();
        if data.len() != self.page_size as usize {
            return Err(PageError::InvalidPageSize);
        }
        self.file.seek(SeekFrom::Start(self.page_offset(page_id)))?;
        self.file.write_all(data)?;
        self.file.flush()?;
        Ok(())
    }

    fn page_offset(&self, page_id: u64) -> u64 {
        page_id * self.page_size
    }

    pub fn blank_page(&self) -> Vec<u8> {
        vec![0; self.page_size as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_page_io() -> (NamedTempFile, PageIO) {
        let temp_file = NamedTempFile::new().unwrap();
        let page_size = 128; // Smaller page size for testing
        let page_io = PageIO::new(temp_file.path(), page_size).unwrap();
        (temp_file, page_io)
    }

    #[test]
    fn test_write_and_read_page() {
        let (_temp, mut page_io) = setup_test_page_io();
        let write_data = vec![42u8; page_io.page_size as usize];
        page_io
            .write_page(0, &Page::new(write_data.clone()))
            .unwrap();

        let read_page = page_io.read_page(0).unwrap();
        assert_eq!(write_data, read_page.as_bytes());
    }

    #[test]
    fn test_read_nonexistent_page() {
        let (_temp, mut page_io) = setup_test_page_io();
        let result = page_io.read_page(999);
        assert!(matches!(result, Err(PageError::PageNotFound)));
    }

    #[test]
    fn test_invalid_buffer_size() {
        let (_temp, mut page_io) = setup_test_page_io();
        let result = page_io.read_page(0);
        assert!(matches!(result, Err(PageError::PageNotFound)));
    }

    #[test]
    fn test_blank_page() {
        let (_temp, page_io) = setup_test_page_io();
        let blank = page_io.blank_page();
        assert_eq!(blank.len(), page_io.page_size as usize);
        assert!(blank.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn test_read_with_blank_page() {
        let (_temp, mut page_io) = setup_test_page_io();
        let write_data = vec![42u8; page_io.page_size as usize];
        page_io
            .write_page(0, &Page::new(write_data.clone()))
            .unwrap();

        let read_page = page_io.read_page(0).unwrap();
        assert_eq!(write_data, read_page.as_bytes());
    }
}
