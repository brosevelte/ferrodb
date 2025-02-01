use super::page::{Page, PageDecodeError};
use crate::storage::page_io::{PageIO, PageIOError};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PageManagerError {
    #[error("Invalid cache size: {0}")]
    InvalidCacheSize(String),

    #[error("Page error: {0}")]
    PageDecodeError(#[from] PageDecodeError),

    #[error("Page IO error: {0}")]
    PageIOError(#[from] PageIOError),
}

pub struct PageManager {
    page_io: PageIO,
    cache: LruCache<u64, Page>,
    page_size: usize,
}

impl PageManager {
    fn new(
        db_path: impl AsRef<Path>,
        page_size: usize,
        cache_size: usize,
    ) -> Result<Self, PageManagerError> {
        let cache = NonZeroUsize::new(cache_size).ok_or(PageManagerError::InvalidCacheSize(
            "Cache size must be greater than 0.".into(),
        ))?;
        Ok(Self {
            page_io: PageIO::new(db_path)?,
            cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            page_size: page_size,
        })
    }

    pub fn get_page(&mut self, page_id: u64) -> Result<&Page, PageManagerError> {
        if !self.cache.contains(&page_id) {
            let page = self.page_io.read_page(page_id, self.page_size)?;
            self.cache.put(page_id, page);
        }
        Ok(self.cache.get(&page_id).unwrap())
    }

    pub fn write_page(&mut self, page_id: u64, page: Page) -> Result<(), PageManagerError> {
        self.page_io.write_page(page_id, self.page_size, &page)?;
        self.cache.put(page_id, page);
        Ok(())
    }

    pub fn invalidate(&mut self, page_id: u64) {
        self.cache.pop(&page_id);
    }

    pub fn flush(&mut self) -> Result<(), PageManagerError> {
        for (&page_id, page) in self.cache.iter() {
            self.page_io.write_page(page_id, self.page_size, page)?;
        }
        Ok(())
    }
}

pub struct PageManagerBuilder {
    db_path: PathBuf,
    page_size: usize,
    cache_size: usize,
}

impl PageManagerBuilder {
    pub fn new(db_path: impl AsRef<Path>) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            page_size: 4096,  // Default page size
            cache_size: 1000, // Default cache size
        }
    }

    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    pub fn cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    pub fn build(self) -> Result<PageManager, PageManagerError> {
        if self.page_size == 0 {
            return Err(PageManagerError::PageDecodeError(
                PageDecodeError::InvalidPageSize("Page size cannot be 0".into()),
            ));
        }

        PageManager::new(self.db_path, self.page_size, self.cache_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_manager() -> (NamedTempFile, PageManager) {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = PageManagerBuilder::new(temp_file.path())
            .page_size(128) // Smaller page size for testing
            .cache_size(10) // Small cache for testing
            .build()
            .unwrap();
        (temp_file, manager)
    }

    #[test]
    fn test_builder_configuration() {
        let temp_file = NamedTempFile::new().unwrap();

        // Test default configuration
        let default_manager = PageManagerBuilder::new(temp_file.path()).build().unwrap();
        assert_eq!(default_manager.page_size, 4096);

        // Test custom configuration
        let custom_manager = PageManagerBuilder::new(temp_file.path())
            .page_size(8192)
            .cache_size(500)
            .build()
            .unwrap();
        assert_eq!(custom_manager.page_size, 8192);
    }

    #[test]
    fn test_invalid_page_size() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = PageManagerBuilder::new(temp_file.path())
            .page_size(0)
            .build();
        assert!(matches!(
            result,
            Err(PageManagerError::PageDecodeError(
                PageDecodeError::InvalidBytes(_)
            ))
        ));
    }

    #[test]
    fn test_cache_hit() {
        let (_temp, mut manager) = setup_test_manager();
        let page_size = manager.page_size;
        let page = Page::full(42, page_size);
        manager.write_page(0, page).unwrap();

        let page = manager.get_page(0).unwrap();
        assert_eq!(page.as_bytes(), &vec![42u8; page_size]);
    }

    #[test]
    fn test_cache_eviction() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = PageManagerBuilder::new(temp_file.path())
            .page_size(128)
            .cache_size(1) // Very small cache for testing eviction
            .build()
            .unwrap();

        // Write two pages with cache size 1
        let data1 = vec![1u8; manager.page_size];
        let data2 = vec![2u8; manager.page_size];

        manager.write_page(0, Page::new(data1.clone())).unwrap();
        manager.write_page(1, Page::new(data2.clone())).unwrap();

        // First page should be evicted and require disk read
        let page1 = manager.get_page(0).unwrap();
        assert_eq!(page1.as_bytes(), &data1);
    }

    #[test]
    fn test_flush() {
        let (_temp, mut manager) = setup_test_manager();
        let data = vec![42u8; manager.page_size];

        manager.write_page(0, Page::new(data.clone())).unwrap();
        manager.flush().unwrap();

        // Create new manager to verify data was written to disk
        let mut new_manager = PageManager::new(_temp.path(), manager.page_size, 10).unwrap();
        let page = new_manager.get_page(0).unwrap();
        assert_eq!(page.as_bytes(), &data);
    }

    #[test]
    fn test_buffer_reuse() {
        let (_temp, mut manager) = setup_test_manager();

        // Write different data to multiple pages
        for i in 0..5 {
            let data = vec![(i as u8); manager.page_size];
            manager.write_page(i, Page::new(data)).unwrap();
        }

        // Read them back, this should cycle through the buffers
        for i in 0..5 {
            let expected = vec![(i as u8); manager.page_size];
            let page = manager.get_page(i).unwrap();
            assert_eq!(*page, Page::new(expected));
        }
    }
}
