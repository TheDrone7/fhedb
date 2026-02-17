# FHEDB

A database management system written in Rust.

The goal of this project is to create a simple database management system.

It is slightly inspired by MongoDB and PostgreSQL, mixing features from both, but it is a very minimal implementation.

It uses the document and collection model, similar to MongoDB. Here are some of the specifics:

- **Document**: A JSON-like object that can contain basic types such as strings, int, float, bool, and arrays.
- **Collection**: A set of documents, similar to a table in a relational database. Each collection has a unique name and can contain multiple documents.
- **Schema**: A collection must have a schema, which defines the structure of the documents in the collection. The schema is required (unlike mongodb).
- **Supported types**: The database supports basic types such as strings, int, float, bool, and arrays. It also supports references: these are similar to foreign keys in relational databases. This allows linking documents between different collections.
- **IDs**: In each collection's schema, there will always be one and only one field that will be used as the ID of the document. This field must either be a string(default value: random uuid) or an int(default value: auto-incrementing integer). If the user does not define an ID field for documents in the schema, the database will generate one automatically called `id` with the auto-incrementing integer ID type.

The "fancy" part of this **_(planned)_** would be that the DBMS applies FHE (Fully Homomorphic Encryption) to the data before storing it in the files. This way, you could allow processing of the data without decrypting it, which is useful for privacy-preserving applications.

---

### Roadmap

- [x] Schema management
- [x] Collection (metadata) management
- [x] Document creation
- [x] Indexing
- [x] CRUD operations (other than create)
- [x] Database metadata management
- [x] Server for accessing the database
- [x] Querying (basic, fields other than ID)
- [ ] Secondary indices
- [x] Custom query language (GraphQL-like, allows for easy joins)
- [ ] Basic security (authentication, authorization)
- [ ] FHE support (encryption, decryption, processing)

#### Why?

Fun
