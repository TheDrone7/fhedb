use serde::{Deserialize, Serialize};

/// A struct to hold metadata about the database.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DbMetadata {
    /// The name of the database.
    pub name: String,
    /// The version of the FHEDB the database was created with.
    pub version: String,
    /// The date and time the database was created.
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created: chrono::DateTime<chrono::Utc>,
    /// The date and time the database was last modified.
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// The number of documents in the database.
    pub document_count: u64,
    /// The size of the database in bytes.
    pub size: u64,
}

/// Implementation of the DbMetadata struct.
impl DbMetadata {
    /// Create a new DbMetadata struct.
    ///
    /// # Arguments
    /// * `name` - The name of the database.
    ///
    /// # Returns
    /// A new DbMetadata struct.
    ///
    /// # Example
    /// ```
    /// use fhedb_core::prelude::DbMetadata;
    /// let db = DbMetadata::new("test".to_owned());
    /// assert_eq!(db.name, "test");
    /// ```
    pub fn new(name: String) -> Self {
        DbMetadata {
            name,
            version: std::env::var("CARGO_PKG_VERSION").unwrap(),
            created: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            document_count: 0,
            size: 0,
        }
    }
}

/// Implementations of the TryFrom trait for the DbMetadata struct.
/// This allows the DbMetadata struct to be created from a byte slice.
impl TryFrom<&[u8]> for DbMetadata {
    /// The error type for the TryFrom trait.
    /// Just forwards the bson error type.
    type Error = bson::de::Error;

    /// Create a new DbMetadata struct from a byte slice.
    ///
    /// # Arguments
    /// * `file_contents` - The byte slice to create the DbMetadata struct from.
    ///
    /// # Returns
    /// A new DbMetadata struct.
    ///
    /// # Example
    /// ```
    /// use fhedb_core::prelude::DbMetadata;
    /// let bytes: [u8; 4] = [0, 0, 0, 0];
    /// let db = DbMetadata::try_from(&bytes[..]);
    /// assert_eq!(db.is_err(), true);
    /// ```
    fn try_from(file_contents: &[u8]) -> Result<Self, Self::Error> {
        bson::from_slice(file_contents)
    }
}

/// Implementations of the TryInto trait for the DbMetadata struct.
/// This allows the DbMetadata struct to be converted to a byte vector.
impl TryInto<Vec<u8>> for &DbMetadata {
    /// The error type for the TryInto trait.
    /// Just forwards the bson error type.
    type Error = bson::ser::Error;

    /// Convert the DbMetadata struct to a byte vector.
    ///
    /// # Returns
    /// A byte vector representation of the DbMetadata struct.
    ///
    /// # Example
    /// ```
    /// use fhedb_core::prelude::DbMetadata;
    /// let db = &DbMetadata::new("test".to_owned());
    /// let bytes: Vec<u8> = db.try_into().unwrap();
    /// assert_eq!(bytes.len(), 117);
    /// ```
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        bson::to_vec(self)
    }
}
