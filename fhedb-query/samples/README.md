# FHEDB Sample Queries

This directory contains sample queries for various operations in FHEDB.
This is a reference for development of the query parser and executor.

## Reading Samples

Starting with the simplest sample, `create_database.fhedb`:

```plaintext
create database <database_name> [drop if exists]
```

- Here `create database` is the command.
    - Plain text such as this is the syntax with keywords.
- `<database_name>` is a placeholder for the actual name of the database to be created.
    - Text inside angle brackets (`< >`) represents a variable or placeholder that is **required** should be replaced with actual values when executing the command.
- `[drop if exists]` is an optional clause that can be included to drop the database if it already exists before creating a new one.
    - Text inside square brackets (`[ ]`) indicates that this part of the command is **optional** and can be omitted.

> **NOTE**: The samples also use `{}` braces that are a part of the syntax itself, not placeholders.

## Available Samples

- Database
    - `create_database.fhedb`: Create a new database, with an optional clause to drop it if it already exists.
    - `drop_database.fhedb`: Drop an existing database.

- Collection
    - `create_collection.fhedb`: Create a new collection within a specified database, with an optional clause to drop it if it already exists.
    - `drop_collection.fhedb`: Drop an existing collection from a specified database.
    - `modify_collection.fhedb`: Modify the schema of an existing collection in a specified database.
    - `list_collections.fhedb`: List all collections in a specified database.
    - `get_collection_schema.fhedb`: Retrieve the schema of a specified collection in a specified database.

- Document
    - `insert_document.fhedb`: Insert a new document into a specified collection.
    - `update_document.fhedb`: Update an existing document in a specified collection.
    - `delete_document.fhedb`: Delete a document from a specified collection.
    - `get_document.fhedb`: Get a specific document (by ID) from a specified collection.
    - `list_documents.fhedb`: List all documents in a specified collection.

> **NOTE**: Difference between querying a single document and listing all documents is merely specifying a value for the ID field.
> This can later 