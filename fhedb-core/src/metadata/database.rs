use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Database {
    pub name: String,
    pub version: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub document_count: u64,
    pub offset: u64,
    pub size: u64,
    pub documents: Vec<bson::Document>,
}

impl Database {
    pub fn new(name: String) -> Self {
        let mut db = Database {
            name,
            version: std::env::var("CARGO_PKG_VERSION").unwrap(),
            created: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            document_count: 0,
            offset: 0,
            size: 0,
            documents: Vec::new(),
        };

        db.fix_offset();

        db
    }

    pub fn from(file_contents: &[u8]) -> Result<Self, bson::de::Error> {
        bson::from_slice(file_contents)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bson::to_vec(self).unwrap()
    }

    pub fn fix_offset(&mut self) {
        self.offset = self.find_offset();
    }

    fn find_offset(&self) -> u64 {
        let mut offset = 0;
        let bytes = self.to_bytes();
        let documents_bytes = "documents".as_bytes();

        for i in 0..bytes.len() {
            if bytes[i..i + documents_bytes.len()] == *documents_bytes {
                offset = i as u64 + documents_bytes.len() as u64 + 5;
                break;
            }
        }

        offset
    }
}
