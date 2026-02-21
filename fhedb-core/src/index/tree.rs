//! # B+ Tree
//!
//! Provides the [`BPlusTree`] structure for managing a disk-backed B+ tree index.

use crate::index::{
    node::{InternalCell, LeafCell, Node, NodeHeader, NodeType, SLOT_SIZE},
    pager::{PAGE_SIZE, Page, Pager},
};
use std::io;

/// A disk-backed B+ tree index structure.
pub struct BPlusTree {
    /// The pager responsible for page-level file I/O.
    pager: Pager,
}

impl BPlusTree {
    /// Opens a B+ tree backed by the given [`Pager`].
    /// Initializes a new root leaf node if the tree is empty.
    ///
    /// ## Arguments
    ///
    /// * `pager` - The [`Pager`] to use for page I/O.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`BPlusTree`]) if successful,
    /// or [`Err`]\([`io::Error`]) if initialization failed.
    pub fn open(pager: Pager) -> io::Result<Self> {
        let mut tree = Self { pager };

        if tree.pager.page_count() == 1 && tree.pager.root_page_num() == 0 {
            tree.initialize()?;
        }

        Ok(tree)
    }

    /// Initializes the tree by allocating a root leaf page.
    fn initialize(&mut self) -> io::Result<()> {
        let root_page_num = self.pager.allocate_page()?;
        self.pager.set_root(root_page_num)?;

        let mut root_page = self.pager.read_page(root_page_num)?;
        let mut root_node = Node::new(&mut root_page);
        root_node.init(NodeType::Leaf, 0);
        self.pager.write_page(root_page_num, &root_page)?;

        Ok(())
    }

    /// Traverses the tree from the root to find the leaf page that would contain the given key.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to search for.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`u32`]) with the leaf page number,
    /// or [`Err`]\([`io::Error`]) if a page could not be read.
    pub fn find_leaf(&mut self, key: &[u8]) -> io::Result<u32> {
        let mut current_page_num = self.pager.root_page_num();

        loop {
            let mut page = self.pager.read_page(current_page_num)?;
            let node = Node::new(&mut page);
            let header = node.get_header();

            if header.node_type == NodeType::Leaf {
                return Ok(current_page_num);
            }

            let (index, found) = node.binary_search(key);

            if found {
                let cell_data = node.get_cell_data(index);
                current_page_num = InternalCell::from_bytes(cell_data).child_page;
            } else if index == 0 {
                current_page_num = header.first_child;
            } else {
                let cell_data = node.get_cell_data(index - 1);
                current_page_num = InternalCell::from_bytes(cell_data).child_page;
            }
        }
    }

    /// Inserts a key-value pair into the tree.
    /// Splits leaf nodes if needed to accommodate the new entry.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to insert.
    /// * `value` - The 16-byte value to associate with the key.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the key already exists or the payload is too large.
    pub fn insert(&mut self, key: &[u8], value: &[u8; 16]) -> io::Result<()> {
        let cell = LeafCell { key, value };
        let cell_bytes = cell.to_bytes();

        if cell_bytes.len() > PAGE_SIZE - NodeHeader::SIZE - SLOT_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Key payload too large for a single page",
            ));
        }

        let mut current_page_num = self.find_leaf(key)?;
        let mut page = self.pager.read_page(current_page_num)?;

        loop {
            let (idx, found, parent_page_num) = {
                let node = Node::new(&mut page);
                let (idx, found) = node.binary_search(key);
                (idx, found, node.get_header().parent_page)
            };

            if found {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Duplicate key: The key already exists in the index.",
                ));
            }

            let insert_result = {
                let mut node = Node::new(&mut page);
                node.insert_cell(idx, &cell_bytes)
            };

            if insert_result.is_ok() {
                self.pager.write_page(current_page_num, &page)?;
                return Ok(());
            } else {
                let new_page_num = self.pager.allocate_page()?;
                let mut new_page = [0u8; PAGE_SIZE];

                let separator = self.split_leaf(&mut page, &mut new_page, new_page_num)?;
                self.pager.write_page(current_page_num, &page)?;
                self.pager.write_page(new_page_num, &new_page)?;

                self.insert_internal(parent_page_num, current_page_num, new_page_num, &separator)?;

                if key >= separator.as_slice() {
                    current_page_num = new_page_num;
                }
                page = self.pager.read_page(current_page_num)?;
            }
        }
    }

    /// Splits a leaf node into two halves.
    /// Cells are divided at the midpoint, with the left half staying in `left_page`
    /// and the right half moved to `right_page`.
    ///
    /// ## Arguments
    ///
    /// * `left_page` - The original leaf page to split (modified in place).
    /// * `right_page` - The new page that receives the upper half of cells.
    /// * `right_page_num` - The page number assigned to `right_page`.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<u8>`]) with the separator key (first key of the right node),
    /// or [`Err`]\([`io::Error`]) on failure.
    pub fn split_leaf(
        &mut self,
        left_page: &mut Page,
        right_page: &mut Page,
        right_page_num: u32,
    ) -> io::Result<Vec<u8>> {
        let left_node = Node::new(left_page);
        let left_header = left_node.get_header();

        let mut right_node = Node::new(right_page);
        right_node.init(NodeType::Leaf, left_header.parent_page);
        let mut right_header = right_node.get_header();
        right_header.next_page = left_header.next_page;
        right_node.set_header(right_header);

        let mut temp_left_page = [0u8; PAGE_SIZE];
        let mut temp_left_node = Node::new(&mut temp_left_page);
        temp_left_node.init(NodeType::Leaf, left_header.parent_page);
        let mut temp_header = temp_left_node.get_header();
        temp_header.next_page = right_page_num;
        temp_left_node.set_header(temp_header);

        let mid_index = left_header.keys_count / 2;
        for i in 0..mid_index {
            let cell_data = left_node.get_cell_data(i);
            temp_left_node.insert_cell(i, cell_data).unwrap();
        }
        for i in mid_index..left_header.keys_count {
            let cell_data = left_node.get_cell_data(i);
            right_node.insert_cell(i - mid_index, cell_data).unwrap();
        }

        let separator_key = right_node.get_key_at(0).to_vec();
        left_page.copy_from_slice(&temp_left_page);

        Ok(separator_key)
    }

    /// Inserts a separator key into an internal node after a child split.
    /// Creates a new root if `parent_page_num` is 0. Recursively splits
    /// internal nodes as needed.
    ///
    /// ## Arguments
    ///
    /// * `parent_page_num` - The page number of the parent internal node, or 0 to create a new root.
    /// * `left_page_num` - The page number of the left child.
    /// * `right_page_num` - The page number of the new right child.
    /// * `separator_key` - The key that separates the two children.
    pub fn insert_internal(
        &mut self,
        parent_page_num: u32,
        left_page_num: u32,
        right_page_num: u32,
        separator_key: &[u8],
    ) -> io::Result<()> {
        if parent_page_num == 0 {
            let new_root_page_num = self.pager.allocate_page()?;
            let mut new_root_page = [0u8; PAGE_SIZE];
            let mut new_root_node = Node::new(&mut new_root_page);
            new_root_node.init(NodeType::Internal, 0);

            let mut header = new_root_node.get_header();
            header.first_child = left_page_num;
            new_root_node.set_header(header);

            let cell = InternalCell {
                key: separator_key,
                child_page: right_page_num,
            };
            new_root_node.insert_cell(0, &cell.to_bytes()).unwrap();
            self.pager.write_page(new_root_page_num, &new_root_page)?;

            self.pager.set_root(new_root_page_num)?;
            self.update_parent(left_page_num, new_root_page_num)?;
            self.update_parent(right_page_num, new_root_page_num)?;

            return Ok(());
        }

        let mut current_page_num = parent_page_num;
        let mut current_page = self.pager.read_page(current_page_num)?;

        loop {
            let grand_parent_page_num = {
                let node = Node::new(&mut current_page);
                node.get_header().parent_page
            };

            let cell = InternalCell {
                key: separator_key,
                child_page: right_page_num,
            };
            let cell_bytes = cell.to_bytes();

            let insert_result = {
                let mut node = Node::new(&mut current_page);
                let (idx, _) = node.binary_search(separator_key);
                node.insert_cell(idx, &cell_bytes)
            };

            if insert_result.is_ok() {
                self.pager.write_page(current_page_num, &current_page)?;
                return Ok(());
            } else {
                let new_page_num = self.pager.allocate_page()?;
                let mut new_page = [0u8; PAGE_SIZE];

                let (new_separator, adopted_children) =
                    self.split_internal(&mut current_page, &mut new_page, new_page_num)?;

                for child in adopted_children {
                    self.update_parent(child, new_page_num)?;
                }

                self.pager.write_page(current_page_num, &current_page)?;
                self.pager.write_page(new_page_num, &new_page)?;

                self.insert_internal(
                    grand_parent_page_num,
                    current_page_num,
                    new_page_num,
                    &new_separator,
                )?;

                if separator_key >= new_separator.as_slice() {
                    current_page_num = new_page_num;
                }
                current_page = self.pager.read_page(current_page_num)?;
            }
        }
    }

    /// Splits an internal node into two halves.
    /// The median key is promoted as the separator, and child pointers are
    /// redistributed between the two pages.
    ///
    /// ## Arguments
    ///
    /// * `left_page` - The original internal page to split (modified in place).
    /// * `right_page` - The new page that receives the upper half of cells.
    /// * `right_page_num` - The page number assigned to `right_page`.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\((`separator_key`, `adopted_children`)) where `separator_key` is the
    /// promoted key and `adopted_children` are the child page numbers moved to `right_page`,
    /// or [`Err`]\([`io::Error`]) on failure.
    pub fn split_internal(
        &mut self,
        left_page: &mut Page,
        right_page: &mut Page,
        right_page_num: u32,
    ) -> io::Result<(Vec<u8>, Vec<u32>)> {
        let left_node = Node::new(left_page);
        let left_header = left_node.get_header();

        let mut right_node = Node::new(right_page);
        right_node.init(NodeType::Internal, left_header.parent_page);
        let mut right_header = right_node.get_header();
        right_header.next_page = left_header.next_page;
        right_node.set_header(right_header);

        let mut temp_left_page = [0u8; PAGE_SIZE];
        let mut temp_left_node = Node::new(&mut temp_left_page);
        temp_left_node.init(NodeType::Internal, left_header.parent_page);
        let mut temp_header = temp_left_node.get_header();
        temp_header.first_child = left_header.first_child;
        temp_header.next_page = right_page_num;
        temp_left_node.set_header(temp_header);

        let mid_index = left_header.keys_count / 2;
        for i in 0..mid_index {
            let cell_data = left_node.get_cell_data(i);
            temp_left_node.insert_cell(i, cell_data).unwrap();
        }

        let mid_cell_data = left_node.get_cell_data(mid_index);
        let mid_cell = InternalCell::from_bytes(mid_cell_data);
        let separator_key = mid_cell.key.to_vec();

        let mut right_header = right_node.get_header();
        right_header.first_child = mid_cell.child_page;
        right_node.set_header(right_header);

        let mut adopted_children = Vec::new();
        adopted_children.push(mid_cell.child_page);

        for i in mid_index + 1..left_header.keys_count {
            let cell_data = left_node.get_cell_data(i);
            right_node
                .insert_cell(i - mid_index - 1, cell_data)
                .unwrap();

            let child_cell = InternalCell::from_bytes(cell_data);
            adopted_children.push(child_cell.child_page);
        }

        left_page.copy_from_slice(&temp_left_page);
        Ok((separator_key, adopted_children))
    }

    /// Updates the parent page pointer of a child node.
    ///
    /// ## Arguments
    ///
    /// * `child_page_num` - The page number of the child node to update.
    /// * `new_parent_page_num` - The new parent page number to set.
    pub fn update_parent(
        &mut self,
        child_page_num: u32,
        new_parent_page_num: u32,
    ) -> io::Result<()> {
        let mut page = self.pager.read_page(child_page_num)?;
        let mut node = Node::new(&mut page);
        let mut header = node.get_header();
        header.parent_page = new_parent_page_num;
        node.set_header(header);
        self.pager.write_page(child_page_num, &page)
    }

    /// Updates the value associated with an existing key.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to look up.
    /// * `new_value` - The new 16-byte value to store.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`io::Error`]) if the key was not found.
    pub fn update(&mut self, key: &[u8], new_value: &[u8; 16]) -> io::Result<()> {
        let page_num = self.find_leaf(key)?;
        let mut page = self.pager.read_page(page_num)?;

        let (idx, found) = {
            let node = Node::new(&mut page);
            node.binary_search(key)
        };

        if !found {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Key not found"));
        }

        let mut node = Node::new(&mut page);
        node.update_leaf_value(idx, new_value);

        self.pager.write_page(page_num, &page)?;

        Ok(())
    }

    /// Retrieves the value associated with the given key.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to look up.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Some`]\([`[u8; 16]`])) if found,
    /// [`Ok`]\([`None`]) if the key does not exist,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn get(&mut self, key: &[u8]) -> io::Result<Option<[u8; 16]>> {
        let page_num = self.find_leaf(key)?;
        let mut page = self.pager.read_page(page_num)?;

        let node = Node::new(&mut page);
        let (idx, found) = node.binary_search(key);

        if !found {
            return Ok(None);
        }

        let cell_data = node.get_cell_data(idx);
        let cell = LeafCell::from_bytes(cell_data);

        Ok(Some(*cell.value))
    }

    /// Performs a range scan returning all values with keys in `[start_key, end_key]`.
    ///
    /// ## Arguments
    ///
    /// * `start_key` - The inclusive lower bound of the scan range.
    /// * `end_key` - The inclusive upper bound of the scan range.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`Vec<[u8; 16]>`]) with matching values,
    /// or [`Err`]\([`io::Error`]) on I/O failure.
    pub fn scan(&mut self, start_key: &[u8], end_key: &[u8]) -> io::Result<Vec<[u8; 16]>> {
        let mut results = Vec::new();
        let mut page_num = self.find_leaf(start_key)?;
        let mut is_first_page = true;

        loop {
            if page_num == 0 {
                break;
            }

            let mut page = self.pager.read_page(page_num)?;
            let node = Node::new(&mut page);
            let header = node.get_header();
            let (start, _) = if is_first_page {
                is_first_page = false;
                node.binary_search(start_key)
            } else {
                (0, false)
            };

            for i in start..header.keys_count {
                let cell_data = node.get_cell_data(i);
                let cell = LeafCell::from_bytes(cell_data);

                if cell.key > end_key {
                    return Ok(results);
                }

                results.push(*cell.value);
            }

            page_num = header.next_page;
        }

        Ok(results)
    }

    /// Deletes a key and its associated value from the tree.
    /// Attempts to merge underflowing leaf nodes after deletion.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to delete.
    pub fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        let page_num = self.find_leaf(key)?;
        let mut page = self.pager.read_page(page_num)?;

        let (idx, found) = {
            let node = Node::new(&mut page);
            node.binary_search(key)
        };

        if !found {
            return Ok(());
        }

        let used_space = {
            let mut node = Node::new(&mut page);
            node.delete_cell(idx);
            node.used_space()
        };

        self.pager.write_page(page_num, &page)?;

        let max_capacity = PAGE_SIZE - NodeHeader::SIZE;
        if used_space < max_capacity / 2 {
            self.attempt_merge(page_num)?;
        }

        Ok(())
    }

    /// Attempts to merge an underflowing leaf node with one of its siblings.
    ///
    /// ## Arguments
    ///
    /// * `page_num` - The page number of the underflowing node.
    pub fn attempt_merge(&mut self, page_num: u32) -> io::Result<()> {
        let parent_page_num = {
            let mut page = self.pager.read_page(page_num)?;
            let node = Node::new(&mut page);
            node.get_header().parent_page
        };

        if parent_page_num == 0 {
            return Ok(());
        }

        let mut parent_page = self.pager.read_page(parent_page_num)?;
        let parent_node = Node::new(&mut parent_page);
        let parent_header = parent_node.get_header();

        // page_idx = 0: header.first_child    (no slot)
        // page_idx = 1: node.get_cell_data(0) (first slot)
        let mut page_idx = None;

        if parent_header.first_child == page_num {
            page_idx = Some(0_usize);
        } else {
            for i in 0..parent_header.keys_count {
                let cell_data = parent_node.get_cell_data(i);
                let child_page = InternalCell::from_bytes(cell_data).child_page;
                if child_page == page_num {
                    page_idx = Some(i as usize + 1);
                    break;
                }
            }
        }

        let page_idx = if let Some(idx) = page_idx {
            idx
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Page corrupt: page {} should be a child of page {} but no pointer found in parent.",
                    page_num, parent_page_num
                ),
            ));
        };

        let left_sibling = if page_idx == 1 {
            Some((parent_header.first_child, 0))
        } else if page_idx > 1 {
            let cell_data = parent_node.get_cell_data((page_idx - 2) as u16);
            let child_page = InternalCell::from_bytes(cell_data).child_page;
            Some((child_page, (page_idx - 1) as u16))
        } else {
            None
        };

        let right_sibling = if page_idx < parent_header.keys_count as usize {
            let cell_data = parent_node.get_cell_data(page_idx as u16);
            let child_page = InternalCell::from_bytes(cell_data).child_page;
            Some((child_page, page_idx as u16))
        } else {
            None
        };

        if let Some((left_page_num, remove_idx)) = left_sibling
            && self.merge_leaves(left_page_num, page_num, parent_page_num, remove_idx)?
        {
            return Ok(());
        }

        if let Some((right_page_num, remove_idx)) = right_sibling {
            self.merge_leaves(page_num, right_page_num, parent_page_num, remove_idx)?;
        }

        Ok(())
    }

    /// Merges two adjacent leaf nodes if their combined data fits in a single page.
    ///
    /// ## Arguments
    ///
    /// * `left_page_num` - The page number of the left leaf node.
    /// * `right_page_num` - The page number of the right leaf node.
    /// * `parent_page_num` - The page number of the parent internal node.
    /// * `remove_idx` - The index of the separator key in the parent to remove.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\([`bool`]) indicating whether the merge was performed,
    /// or [`Err`]\([`io::Error`]) on failure.
    pub fn merge_leaves(
        &mut self,
        left_page_num: u32,
        right_page_num: u32,
        parent_page_num: u32,
        remove_idx: u16,
    ) -> io::Result<bool> {
        let mut left_page = self.pager.read_page(left_page_num)?;
        let mut right_page = self.pager.read_page(right_page_num)?;

        let left_used = {
            let node = Node::new(&mut left_page);
            node.used_space()
        };

        let right_used = {
            let node = Node::new(&mut right_page);
            node.used_space()
        };

        let max_capacity = PAGE_SIZE - NodeHeader::SIZE;
        if left_used + right_used > max_capacity {
            return Ok(false);
        }

        let mut left_node = Node::new(&mut left_page);
        let right_node = Node::new(&mut right_page);
        let right_header = right_node.get_header();

        for i in 0..right_header.keys_count {
            let cell_data = right_node.get_cell_data(i);
            left_node
                .insert_cell(left_node.get_header().keys_count, cell_data)
                .unwrap();
        }

        let mut left_header = left_node.get_header();
        left_header.next_page = right_header.next_page;
        left_node.set_header(left_header);
        self.pager.write_page(left_page_num, &left_page)?;

        self.pager.free_page(right_page_num)?;
        self.delete_internal(parent_page_num, remove_idx)?;

        Ok(true)
    }

    /// Removes a separator key from an internal node after a leaf merge.
    /// Collapses the root if it becomes empty.
    ///
    /// ## Arguments
    ///
    /// * `parent_page_num` - The page number of the internal node.
    /// * `remove_idx` - The index of the key to remove.
    pub fn delete_internal(&mut self, parent_page_num: u32, remove_idx: u16) -> io::Result<()> {
        let mut page = self.pager.read_page(parent_page_num)?;

        let header = {
            let mut node = Node::new(&mut page);
            node.delete_cell(remove_idx);
            node.get_header()
        };

        if parent_page_num == self.pager.root_page_num() && header.keys_count == 0 {
            self.pager.set_root(header.first_child)?;
            self.pager.free_page(parent_page_num)?;
            self.update_parent(header.first_child, 0)?;
        } else {
            self.pager.write_page(parent_page_num, &page)?;
        }

        Ok(())
    }

    /// Returns a reference to the underlying pager.
    pub fn pager(&mut self) -> &mut Pager {
        &mut self.pager
    }
}
