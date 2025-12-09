use fhedb_core::prelude::Database;
use fhedb_query::ast::DatabaseQuery;
use log::warn;
use std::fs::{create_dir_all, remove_dir_all};

use crate::state::ServerState;

pub(crate) fn execute_base_query(
    query: DatabaseQuery,
    state: &ServerState,
) -> Result<String, String> {
    match query {
        DatabaseQuery::Create {
            name,
            drop_if_exists,
        } => {
            if drop_if_exists {
                _ = drop_db(name.clone(), state);
            }
            return create_db(name, state);
        }
        DatabaseQuery::Drop { name } => return drop_db(name.clone(), &state),
        DatabaseQuery::List => {
            let entries = state.data_dir.read_dir().map_err(|e| e.to_string())?;
            let mut dbs = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.file_type().is_ok_and(|e| e.is_dir()) {
                        if let Ok(db_name) = entry.file_name().into_string() {
                            dbs.push(db_name);
                        } else {
                            warn!("Unable to read directory entry name at base data location.");
                        }
                    } else {
                        warn!("Unknown directory entry at base data location.");
                    }
                } else {
                    warn!("Unable to read directory entry at base data location.");
                }
            }
            return Ok(dbs.join("\n"));
        }
    };
}

fn drop_db(name: String, state: &ServerState) -> Result<String, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let exists_in_memory = dbs.contains_key(&name);
    let db_path = state.data_dir.join(&name);
    let exists_on_disk = db_path.exists();

    if exists_in_memory || exists_on_disk {
        dbs.remove(&name);
        if exists_on_disk {
            remove_dir_all(&db_path).map_err(|e| e.to_string())?;
        }
        Ok(format!("Database {} dropped successfully!", name))
    } else {
        Err("Database does not exist".to_string())
    }
}

fn create_db(name: String, state: &ServerState) -> Result<String, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let exists_in_memory = dbs.contains_key(&name);
    let db_path = state.data_dir.join(&name);
    let exists_on_disk = db_path.exists();

    if exists_in_memory || exists_on_disk {
        Err("Database already exists".to_string())
    } else {
        let db = Database::new(&name, &state.data_dir);
        create_dir_all(db.path()).map_err(|e| e.to_string())?;
        dbs.insert(name.clone(), db);
        Ok(format!("Database {} created successfully!", name))
    }
}
