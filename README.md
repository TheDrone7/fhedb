# FHEDB

A file-based database management system written in Rust.

The goal of this project is to create a simple database management system that stores data in files.

The "fancy" part of this **_(planned)_** would be that the DBMS applies FHE (Fully Homomorphic Encryption) to the data before storing it in the files. This way, the data is always encrypted and can be processed without the need to decrypt it first.

---

### Roadmap

- [ ] Have a working basic file-based database (supports CRUD)
- [ ] Add collections for grouping documents
- [ ] Add schema support
- [ ] Add transactions
- [ ] Implement FHE
- [ ] Add FHE support to the database

### Other features

- [ ] indexing
- [ ] querying (by other fields, joins, etc.)
- [ ] logging
- [ ] constraints (unique, required, etc.)

#### Why?

Fun
