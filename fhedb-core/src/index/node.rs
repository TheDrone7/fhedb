//! # B+ Tree Node
//!
//! Provides node structures and operations for the B+ tree index.

use crate::index::pager::{PAGE_SIZE, Page};
use std::{cmp::Ordering, convert::TryInto};

/// A cell within a leaf node, containing a key and a 16-byte record pointer.
pub struct LeafCell<'a> {
    /// The key bytes for this cell.
    pub key: &'a [u8],
    /// The 16-byte value (record pointer) associated with the key.
    pub value: &'a [u8; 16],
}

impl<'a> LeafCell<'a> {
    /// Parses a leaf cell from its raw byte representation.
    ///
    /// ## Arguments
    ///
    /// * `bytes` - The raw byte slice containing the serialized leaf cell.
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        let key_length = u16::from_le_bytes(bytes[0..2].try_into().unwrap()) as usize;
        let key = &bytes[2..2 + key_length];
        let value = bytes[2 + key_length..2 + key_length + 16]
            .try_into()
            .unwrap();
        Self { key, value }
    }

    /// Converts the leaf cell to a byte array for storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.key.len() + 16);
        bytes.extend_from_slice(&(self.key.len() as u16).to_le_bytes());
        bytes.extend_from_slice(self.key);
        bytes.extend_from_slice(self.value);
        bytes
    }
}

/// A cell within an internal node, containing a key and a child page pointer.
pub struct InternalCell<'a> {
    /// The key bytes for this cell.
    pub key: &'a [u8],
    /// The page number of the child node to the right of this key.
    pub child_page: u32,
}

impl<'a> InternalCell<'a> {
    /// Parses an internal cell from its raw byte representation.
    ///
    /// ## Arguments
    ///
    /// * `bytes` - The raw byte slice containing the serialized internal cell.
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        let key_length = u16::from_le_bytes(bytes[0..2].try_into().unwrap()) as usize;
        let key = &bytes[2..2 + key_length];
        let child_page = u32::from_le_bytes(
            bytes[2 + key_length..2 + key_length + 4]
                .try_into()
                .unwrap(),
        );

        Self { key, child_page }
    }

    /// Converts the internal cell to a byte array for storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 + self.key.len() + 4);
        bytes.extend_from_slice(&(self.key.len() as u16).to_le_bytes());
        bytes.extend_from_slice(self.key);
        bytes.extend_from_slice(&self.child_page.to_le_bytes());
        bytes
    }
}

/// The type of a node in the B+ tree.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum NodeType {
    /// Internal node that contains keys and child pointers.
    Internal = 0,
    /// Leaf node that contains keys and record pointers.
    Leaf = 1,
}

impl From<u8> for NodeType {
    fn from(value: u8) -> Self {
        match value {
            0 => NodeType::Internal,
            1 => NodeType::Leaf,
            _ => panic!("Invalid node type value: {}", value),
        }
    }
}

/// The header of a node in B+ tree, containing relevant metadata for the node.
pub struct NodeHeader {
    /// The type of the node (internal or leaf).
    pub node_type: NodeType,
    /// The number of keys currently stored in the node.
    pub keys_count: u16,
    /// The offset in bytes to the start of the free heap space in the node.
    pub heap_pointer: u16,
    /// The page number of the parent node. If 0, there is no parent (root node).
    pub parent_page: u32,
    /// The page number of the next leaf node (only for leaf nodes). If 0, there is no next page.
    pub next_page: u32,
    /// The first child for internal nodes.
    pub first_child: u32,
}

impl NodeHeader {
    /// The size of the node header in bytes (offset for data).
    pub const SIZE: usize = 1 + 2 + 2 + 4 + 4 + 4;

    /// Reads a [`NodeHeader`] from the raw bytes of a node page.
    ///
    /// ## Arguments
    ///
    /// * `bytes` - The raw bytes of the node page from which to read the header.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_type: NodeType::from(bytes[0]),
            keys_count: u16::from_le_bytes(bytes[1..3].try_into().unwrap()),
            heap_pointer: u16::from_le_bytes(bytes[3..5].try_into().unwrap()),
            parent_page: u32::from_le_bytes(bytes[5..9].try_into().unwrap()),
            next_page: u32::from_le_bytes(bytes[9..13].try_into().unwrap()),
            first_child: u32::from_le_bytes(bytes[13..17].try_into().unwrap()),
        }
    }

    /// Converts the node header to a byte array for storage.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];

        bytes[0] = self.node_type as u8;
        bytes[1..3].copy_from_slice(&self.keys_count.to_le_bytes());
        bytes[3..5].copy_from_slice(&self.heap_pointer.to_le_bytes());
        bytes[5..9].copy_from_slice(&self.parent_page.to_le_bytes());
        bytes[9..13].copy_from_slice(&self.next_page.to_le_bytes());
        bytes[13..17].copy_from_slice(&self.first_child.to_le_bytes());

        bytes
    }
}

/// The size of each slot in the node's pointer array.
pub const SLOT_SIZE: usize = 4;

/// Represents a node in a B+ tree index structure.
///
/// Contains a reference to the page data for the node.
pub struct Node<'a>(pub &'a mut Page);

impl<'a> Node<'a> {
    /// Creates a new node instance from a mutable reference to a page.
    ///
    /// ## Arguments
    ///
    /// * `page` - A mutable reference to the page data for the node.
    pub fn new(page: &'a mut Page) -> Self {
        Self(page)
    }

    /// Initializes a new node in the tree.
    ///
    /// ## Arguments
    ///
    /// * `node_type` - The [`NodeType`] for the node (internal or leaf).
    /// * `parent_page` - The page number for the parent node. If 0, this is the root node.
    pub fn init(&mut self, node_type: NodeType, parent_page: u32) {
        let header = NodeHeader {
            node_type,
            keys_count: 0,
            heap_pointer: PAGE_SIZE as u16,
            parent_page,
            next_page: 0,
            first_child: 0,
        };
        self.set_header(header);
    }

    /// Returns the header for the node.
    pub fn get_header(&self) -> NodeHeader {
        NodeHeader::from_bytes(&self.0[0..NodeHeader::SIZE])
    }

    /// Sets the header for the node.
    ///
    /// ## Arguments
    ///
    /// * `header` - The [`NodeHeader`] instance containing the metadata.
    pub fn set_header(&mut self, header: NodeHeader) {
        self.0[0..NodeHeader::SIZE].copy_from_slice(&header.to_bytes());
    }

    /// Returns the data for a specific cell in the node.
    ///
    /// ## Arguments
    ///
    /// * `idx` - The index of the cell to retrieve (0-based).
    pub fn get_cell_data(&self, idx: u16) -> &[u8] {
        debug_assert!(
            idx < self.get_header().keys_count,
            "Cell index out of bounds"
        );

        let slot_offset = NodeHeader::SIZE + (idx as usize * SLOT_SIZE);
        let cell_offset =
            u16::from_le_bytes(self.0[slot_offset..slot_offset + 2].try_into().unwrap()) as usize;
        let cell_length =
            u16::from_le_bytes(self.0[slot_offset + 2..slot_offset + 4].try_into().unwrap())
                as usize;

        &self.0[cell_offset..cell_offset + cell_length]
    }

    /// Inserts a new cell into the node at the specified index.
    ///
    /// ## Arguments
    ///
    /// * `idx` - The index at which to insert the new cell (0-based).
    /// * `data` - The byte slice to be written as the cell data.
    ///
    /// ## Returns
    ///
    /// Returns [`Ok`]\(()) if successful,
    /// or [`Err`]\([`&'static str`]) if the node does not have enough free space.
    pub fn insert_cell(&mut self, idx: u16, data: &[u8]) -> Result<(), &'static str> {
        let mut header = self.get_header();
        let data_length = data.len() as u16;

        let ptr_array_end = NodeHeader::SIZE + ((header.keys_count + 1) as usize * SLOT_SIZE);
        if ptr_array_end > (header.heap_pointer - data_length) as usize {
            return Err("Node overflow: insufficient space for new cell");
        }

        let new_data_offset = header.heap_pointer - data_length;
        self.0[new_data_offset as usize..(new_data_offset + data_length) as usize]
            .copy_from_slice(data);

        let slot_start_index = NodeHeader::SIZE + (idx as usize * SLOT_SIZE);
        if idx < header.keys_count {
            let shift_length = (header.keys_count - idx) as usize * SLOT_SIZE;
            self.0.copy_within(
                slot_start_index..slot_start_index + shift_length,
                slot_start_index + SLOT_SIZE,
            );
        }

        self.0[slot_start_index..slot_start_index + 2]
            .copy_from_slice(&new_data_offset.to_le_bytes());
        self.0[slot_start_index + 2..slot_start_index + 4]
            .copy_from_slice(&data_length.to_le_bytes());

        header.keys_count += 1;
        header.heap_pointer = new_data_offset;
        self.set_header(header);
        Ok(())
    }

    /// Returns the key bytes for the cell at the specified index.
    ///
    /// ## Arguments
    ///
    /// * `idx` - The index of the cell whose key to retrieve (0-based).
    pub fn get_key_at(&self, idx: u16) -> &[u8] {
        let data = self.get_cell_data(idx);
        let key_length = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;
        &data[2..2 + key_length]
    }

    /// Performs a binary search for the given key within the node's cells.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key bytes to search for.
    ///
    /// ## Returns
    ///
    /// A tuple of (`index`, `found`) where `index` is the position the key was found at
    /// or the position where it would be inserted, and `found` indicates whether an exact
    /// match was found.
    pub fn binary_search(&self, key: &[u8]) -> (u16, bool) {
        let count = self.get_header().keys_count;
        let mut low = 0;
        let mut high = count;

        while low < high {
            let mid = low + (high - low) / 2;
            let mid_key = self.get_key_at(mid);

            match key.cmp(mid_key) {
                Ordering::Equal => return (mid, true),
                Ordering::Less => high = mid,
                Ordering::Greater => low = mid + 1,
            }
        }

        (low, false)
    }

    /// Deletes the cell at the specified index from the node.
    ///
    /// ## Arguments
    ///
    /// * `idx` - The index of the cell to delete (0-based).
    pub fn delete_cell(&mut self, idx: u16) {
        let mut header = self.get_header();

        if idx >= header.keys_count {
            return;
        }

        let slot_offset = NodeHeader::SIZE + (idx as usize * SLOT_SIZE);
        let cell_offset =
            u16::from_le_bytes(self.0[slot_offset..slot_offset + 2].try_into().unwrap());
        let cell_length =
            u16::from_le_bytes(self.0[slot_offset + 2..slot_offset + 4].try_into().unwrap());

        let shift_start = slot_offset + SLOT_SIZE;
        let shift_end = NodeHeader::SIZE + (header.keys_count as usize * SLOT_SIZE);
        if idx < header.keys_count - 1 {
            self.0.copy_within(shift_start..shift_end, slot_offset);
        }

        let move_start = header.heap_pointer as usize;
        let move_end = cell_offset as usize;
        let move_dest = move_start + cell_length as usize;

        if move_start < move_end {
            self.0.copy_within(move_start..move_end, move_dest);

            let remaining_keys = header.keys_count - 1;
            for i in 0..remaining_keys {
                let current_slot = NodeHeader::SIZE + (i as usize * SLOT_SIZE);
                let mut ptr_offset =
                    u16::from_le_bytes(self.0[current_slot..current_slot + 2].try_into().unwrap());

                if ptr_offset < cell_offset {
                    ptr_offset += cell_length;
                    self.0[current_slot..current_slot + 2]
                        .copy_from_slice(&ptr_offset.to_le_bytes());
                }
            }
        }

        header.keys_count -= 1;
        header.heap_pointer += cell_length;
        self.set_header(header);
    }

    /// Updates the value of a cell in the node.
    ///
    /// ## Arguments
    ///
    /// * `idx` - The index of the cell to be updated.
    /// * `new_value` - The new value to be stored for the cell.
    pub fn update_leaf_value(&mut self, idx: u16, new_value: &[u8; 16]) {
        let header = self.get_header();
        if idx >= header.keys_count {
            return;
        }

        let slot_offset = NodeHeader::SIZE + (idx as usize * SLOT_SIZE);
        let cell_offset =
            u16::from_le_bytes(self.0[slot_offset..slot_offset + 2].try_into().unwrap()) as usize;
        let cell_length =
            u16::from_le_bytes(self.0[slot_offset + 2..slot_offset + 4].try_into().unwrap())
                as usize;
        let value_offset = cell_offset + cell_length - 16;

        self.0[value_offset..value_offset + 16].copy_from_slice(new_value);
    }

    /// Returns the amount of space in the node currently in use.
    /// The amount is in bytes and excludes the node header.
    pub fn used_space(&self) -> usize {
        let header = self.get_header();
        let slot_space = (header.keys_count as usize) * SLOT_SIZE;
        let heap_space = PAGE_SIZE - (header.heap_pointer as usize);

        slot_space + heap_space
    }
}
