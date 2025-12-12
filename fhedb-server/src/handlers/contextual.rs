use fhedb_query::ast::ContextualQuery;

use crate::state::ServerState;

use crate::handlers::collection::execute_collection_query;

pub(crate) fn execute_contextual_query(
    db_name: String,
    query: ContextualQuery,
    state: &ServerState,
) -> Result<String, String> {
    match query {
        ContextualQuery::Collection(collection_query) => {
            execute_collection_query(db_name, collection_query, state)
        }
        ContextualQuery::Document(_) => Err("Not implemented".to_string()),
    }
}
