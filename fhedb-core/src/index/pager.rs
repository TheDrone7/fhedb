//! # Index Pager
//!
//! Manages page-level file I/O for B+ tree indices.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

/// The size of a page in bytes.
/// TODO: Make this configurable with server config.
pub const PAGE_SIZE: usize = 4096;

/// A fixed-size block of data representing a page on the disk.
pub type Page = [u8; PAGE_SIZE];

/// The structure responsible for managing page-level file I/O.
#[derive(Debug)]
pub struct Pager {
    /// The file handle for the page file.
    file: File,
    /// The total number of pages in the file.
    total_pages: u32,
}

impl Pager {
    /// Creates a new pager for the specified file path.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the page file. If the file does not exist, it will be created.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Pager`]) if successful,
    /// or [`Err`]\([`io::Error`]) if the file could not be opened or is corrupted.
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let len = file.metadata()?.len();

        if len % PAGE_SIZE as u64 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Page file data is not a multiple of page size.",
            ));
        }

        let total_pages = (len / PAGE_SIZE as u64) as u32;

        Ok(Self { file, total_pages })
    }

    /// Reads a specific page from the file.
    ///
    /// ## Arguments
    ///
    /// * `page_num` - The page number to read (0-based index).
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Page`]) if successful,
    /// or [`Err`]\([`io::Error`]) if the page number is out of bounds or the read failed.
    pub fn read_page(&mut self, page_num: u32) -> io::Result<Page> {
        if page_num >= self.total_pages {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Page number '{}' out of bounds (total: {})",
                    page_num, self.total_pages
                ),
            ));
        }

        let mut page = [0u8; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))?;
        self.file.read_exact(&mut page)?;

        Ok(page)
    }

    /// Writes a page to the file at the specified page number.
    ///
    /// ## Arguments
    ///
    /// * `page_num` - The page number to write to (0-based index).
    /// * `page` - The page data to write.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the page number is out of bounds or the write failed.
    pub fn write_page(&mut self, page_num: u32, page: &Page) -> io::Result<()> {
        if page_num >= self.total_pages {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Page number {} out of bounds (total: {})",
                    page_num, self.total_pages
                ),
            ));
        }

        self.file
            .seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))?;
        self.file.write_all(page)?;

        Ok(())
    }

    /// Allocates a new page at the end of the file.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`u32`]) with the new page number,
    /// or [`Err`]\([`io::Error`]) if the allocation failed.
    pub fn allocate_page(&mut self) -> io::Result<u32> {
        let page_num = self.total_pages;
        let empty_page = [0u8; PAGE_SIZE];

        self.file
            .seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))?;
        self.file.write_all(&empty_page)?;
        self.total_pages += 1;

        Ok(page_num)
    }

    /// Returns the total number of pages in the file.
    pub fn page_count(&self) -> u32 {
        self.total_pages
    }
}
