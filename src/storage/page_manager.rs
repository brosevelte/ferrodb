use super::page::Page;
use crate::storage::page_io::{PageError, PageIO};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::Path;

pub struct PageManager {
    page_io: PageIO,
    cache: LruCache<u64, Page>,
    page_size: usize,
}

impl PageManager {
    pub fn new(
        db_path: impl AsRef<Path>,
        page_size: u64,
        cache_size: usize,
    ) -> Result<Self, PageError> {
        Ok(Self {
            page_io: PageIO::new(db_path, page_size)?,
            cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            page_size: page_size as usize,
        })
    }

    pub fn get_page(&mut self, page_id: u64) -> Result<&Page, PageError> {
        if !self.cache.contains(&page_id) {
            let page = self.page_io.read_page(page_id)?;
            self.cache.put(page_id, page);
        }
        Ok(self.cache.get(&page_id).unwrap())
    }

    pub fn write_page(&mut self, page_id: u64, page: Page) -> Result<(), PageError> {
        self.page_io.write_page(page_id, &page)?;
        self.cache.put(page_id, page);
        Ok(())
    }

    pub fn invalidate(&mut self, page_id: u64) {
        self.cache.pop(&page_id);
    }

    pub fn flush(&mut self) -> Result<(), PageError> {
        for (&page_id, page) in self.cache.iter() {
            self.page_io.write_page(page_id, page)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_manager() -> (NamedTempFile, PageManager) {
        let temp_file = NamedTempFile::new().unwrap();
        let page_size = 128; // Smaller page size for testing
        let cache_size = 10; // Small cache for testing
        let manager = PageManager::new(temp_file.path(), page_size, cache_size).unwrap();
        (temp_file, manager)
    }

    #[test]
    fn test_cache_hit() {
        let (_temp, mut manager) = setup_test_manager();
        let data = vec![42u8; manager.page_size];
        manager.write_page(0, Page::new(data.clone())).unwrap();

        let page = manager.get_page(0).unwrap();
        assert_eq!(page.as_bytes(), &data);
    }

    #[test]
    fn test_cache_eviction() {
        let temp_file = NamedTempFile::new().unwrap();
        let page_size = 128;
        let cache_size = 1; // Very small cache for testing eviction
        let mut manager = PageManager::new(temp_file.path(), page_size, cache_size).unwrap();

        // Write two pages with cache size 1
        let data1 = vec![1u8; page_size as usize];
        let data2 = vec![2u8; page_size as usize];

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
        let mut new_manager = PageManager::new(_temp.path(), manager.page_size as u64, 10).unwrap();
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
