//! # Index Pager
//!
//! Manages page-level file I/O for B+ tree indices.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

/// The size of a page in bytes.
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
    /// The page number of the first (root) page in the file.
    root_page_num: u32,
    /// The page number of the first free page in the file.
    free_page_num: u32,
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

        let pager = if total_pages == 0 {
            let mut temp = Self {
                file,
                total_pages: 1,
                root_page_num: 0,
                free_page_num: 0,
            };
            temp.save_metadata()?;
            temp
        } else {
            let mut temp = Self {
                file,
                total_pages,
                root_page_num: 0,
                free_page_num: 0,
            };
            temp.load_metadata()?;
            temp
        };

        Ok(pager)
    }

    /// Creates a new empty page filled with zeroes.
    pub fn new_page(&self) -> Page {
        [0u8; PAGE_SIZE]
    }

    /// Returns the page number of the root node.
    pub fn root_page_num(&self) -> u32 {
        self.root_page_num
    }

    /// Returns the page number of the first free page.
    pub fn free_page_num(&self) -> u32 {
        self.free_page_num
    }

    /// Sets the root page number and persists the metadata.
    ///
    /// ## Arguments
    ///
    /// * `page_num` - The page number to set as the root.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the metadata could not be written.
    pub fn set_root(&mut self, page_num: u32) -> io::Result<()> {
        self.root_page_num = page_num;
        self.save_metadata()
    }

    /// Loads the pager metadata (root and free page numbers) from page 0.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\((`root_page_num`, `free_page_num`)) if successful,
    /// or [`Err`]\([`io::Error`]) if the metadata page could not be read.
    pub fn load_metadata(&mut self) -> io::Result<(u32, u32)> {
        let metadata_page = self.read_page(0)?;
        let root_page_num = u32::from_le_bytes(metadata_page[0..4].try_into().unwrap());
        let free_page_num = u32::from_le_bytes(metadata_page[4..8].try_into().unwrap());
        self.root_page_num = root_page_num;
        self.free_page_num = free_page_num;

        Ok((root_page_num, free_page_num))
    }

    /// Writes the current pager metadata (root and free page numbers) to page 0.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the metadata page could not be written.
    pub fn save_metadata(&mut self) -> io::Result<()> {
        let mut metadata_page = self.new_page();
        metadata_page[0..4].copy_from_slice(&self.root_page_num.to_le_bytes());
        metadata_page[4..8].copy_from_slice(&self.free_page_num.to_le_bytes());

        self.write_page(0, &metadata_page)
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

        let mut page = self.new_page();
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
        if self.free_page_num != 0 {
            let reused_page_num = self.free_page_num;
            let page = self.read_page(reused_page_num)?;
            self.free_page_num = u32::from_le_bytes(page[0..4].try_into().unwrap());
            self.save_metadata()?;

            self.write_page(reused_page_num, &self.new_page())?;
            return Ok(reused_page_num);
        }

        let page_num = self.total_pages;
        let empty_page = self.new_page();

        self.file
            .seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))?;
        self.file.write_all(&empty_page)?;
        self.total_pages += 1;

        Ok(page_num)
    }

    /// Frees a page by adding it to the free page list.
    ///
    /// ## Arguments
    ///
    /// * `page_num` - The page number to free.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the page number is invalid or the write failed.
    pub fn free_page(&mut self, page_num: u32) -> io::Result<()> {
        if page_num == 0 || page_num >= self.total_pages {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid page number to free.",
            ));
        }

        let mut page = self.new_page();
        page[0..4].copy_from_slice(&self.free_page_num.to_le_bytes());
        self.write_page(page_num, &page)?;
        self.free_page_num = page_num;
        self.save_metadata()
    }

    /// Returns the total number of pages in the file.
    pub fn page_count(&self) -> u32 {
        self.total_pages
    }
}
