use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DbMetadata {
    pub name: String,
    pub version: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub document_count: u64,
    pub offsets: Vec<u64>,
    pub size: u64,
}

impl DbMetadata {
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

    pub fn from(file_contents: &[u8]) -> Result<Self, bson::de::Error> {
        bson::from_slice(file_contents)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bson::to_vec(self).unwrap()
    }
}
