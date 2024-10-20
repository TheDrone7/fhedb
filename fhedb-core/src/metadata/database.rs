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
    /// The offsets of the documents in the database.
    pub offsets: Vec<u64>,
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
    /// use fhedb_core::metadata::DbMetadata;
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
            offsets: Vec::new(),
        }
    }

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
    /// use fhedb_core::metadata::DbMetadata;
    /// let db = DbMetadata::from(&[0, 0, 0, 0]);
    /// assert_eq!(db.is_err(), true);
    /// 
    /// 
    /// let file = std::fs::read("my_database.fhedb").unwrap();
    /// let db = DbMetadata::from(file.as_slice()).unwrap();
    /// assert_eq!(db.name, "test");
    /// ```
    pub fn from(file_contents: &[u8]) -> Result<Self, bson::de::Error> {
        bson::from_slice(file_contents)
    }

    /// Convert the DbMetadata struct to a byte vector.
    /// 
    /// # Returns
    /// A byte vector of the DbMetadata struct.
    /// 
    /// # Example
    /// ```
    /// use fhedb_core::metadata::DbMetadata;
    /// let db = DbMetadata::new("test".to_owned());
    /// let bytes = db.to_bytes();
    /// assert_eq!(bytes.len(), 131);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        bson::to_vec(self).unwrap()
    }
}
