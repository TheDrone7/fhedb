use crate::state::ServerState;
use fhedb_core::db::{
    collection_schema_ops::CollectionSchemaOps,
    schema::{FieldDefinition, FieldType},
};
use fhedb_query::ast::{CollectionQuery, FieldModification};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct JsonFieldDefinition {
    #[serde(rename = "type")]
    field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<String>,
    nullable: bool,
}

impl From<&FieldDefinition> for JsonFieldDefinition {
    fn from(def: &FieldDefinition) -> Self {
        let (type_str, nullable) = extract_type_info(&def.field_type);
        JsonFieldDefinition {
            field_type: type_str,
            default: def.default_value.as_ref().map(|v| v.to_string()),
            nullable,
        }
    }
}

fn extract_type_info(ft: &FieldType) -> (String, bool) {
    match ft {
        FieldType::Nullable(inner) => {
            let (inner_str, _) = extract_type_info(inner);
            (inner_str, true)
        }
        other => (format_field_type(other), false),
    }
}

fn format_field_type(ft: &FieldType) -> String {
    match ft {
        FieldType::Int => "int".to_string(),
        FieldType::Float => "float".to_string(),
        FieldType::Boolean => "boolean".to_string(),
        FieldType::String => "string".to_string(),
        FieldType::IdString => "id_string".to_string(),
        FieldType::IdInt => "id_int".to_string(),
        FieldType::Array(inner) => format!("array({})", format_field_type(inner)),
        FieldType::Reference(r) => format!("reference({})", r),
        FieldType::Nullable(inner) => format!("nullable({})", format_field_type(inner)),
    }
}

pub fn execute_collection_query(
    db_name: String,
    query: CollectionQuery,
    state: &ServerState,
) -> Result<String, String> {
    let mut dbs = state.databases.write().map_err(|e| e.to_string())?;
    let db = dbs
        .get_mut(&db_name)
        .ok_or_else(|| "Database not found".to_string())?;

    match query {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            if drop_if_exists {
                if db.has_collection(&name) {
                    db.drop_collection(&name)?;
                }
            }
            db.create_collection(&name, schema)?;
            let col = db
                .get_collection(&name)
                .ok_or("Collection not found after creation")?;

            let schema_map: std::collections::HashMap<String, JsonFieldDefinition> = col
                .schema()
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), JsonFieldDefinition::from(v)))
                .collect();
            Ok(serde_json::to_string_pretty(&schema_map).map_err(|e| e.to_string())?)
        }
        CollectionQuery::Drop { name } => {
            db.drop_collection(&name)?;
            Ok(serde_json::to_string_pretty(&json!({"dropped": name}))
                .map_err(|e| e.to_string())?)
        }
        CollectionQuery::List => {
            let names = db.collection_names();
            Ok(serde_json::to_string_pretty(&names).map_err(|e| e.to_string())?)
        }
        CollectionQuery::GetSchema { name } => {
            let col = db
                .get_collection(&name)
                .ok_or_else(|| format!("Collection '{}' not found", name))?;
            let schema_map: std::collections::HashMap<String, JsonFieldDefinition> = col
                .schema()
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), JsonFieldDefinition::from(v)))
                .collect();
            Ok(serde_json::to_string_pretty(&schema_map).map_err(|e| e.to_string())?)
        }
        CollectionQuery::Modify {
            name,
            modifications,
        } => {
            let col = db
                .get_collection_mut(&name)
                .ok_or_else(|| format!("Collection '{}' not found", name))?;
            for (field_name, modification) in modifications {
                match modification {
                    FieldModification::Drop => {
                        col.remove_field(&field_name)?;
                    }
                    FieldModification::Set(def) => {
                        if col.has_field(&field_name) {
                            col.modify_field(&field_name, def)?;
                        } else {
                            col.add_field(field_name, def)?;
                        }
                    }
                }
            }

            let schema_map: std::collections::HashMap<String, JsonFieldDefinition> = col
                .schema()
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), JsonFieldDefinition::from(v)))
                .collect();
            Ok(serde_json::to_string_pretty(&schema_map).map_err(|e| e.to_string())?)
        }
    }
}
