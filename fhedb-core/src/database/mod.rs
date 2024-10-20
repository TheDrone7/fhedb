use crate::metadata::database::DbMetadata;

pub struct Database {
    pub path: String,
    pub metadata: DbMetadata,
    pub documents: Vec<bson::Document>,
}

impl Database {
    pub fn new(path: String) -> Self {
        let path = std::path::Path::new(&path);
        let filename = path.file_name().unwrap().to_str().unwrap();

        let metadata = DbMetadata::new(filename.to_owned());
        let path = path.to_str().unwrap().to_owned();

        Database {
            path,
            metadata,
            documents: Vec::new(),
        }
    }
}
