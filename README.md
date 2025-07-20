# FHEDB

A database management system written in Rust.

The goal of this project is to create a simple database management system.

The "fancy" part of this **_(planned)_** would be that the DBMS applies FHE (Fully Homomorphic Encryption) to the data before storing it in the files. This way, you could allow processing of the data without decrypting it, which is useful for privacy-preserving applications.

---

### Roadmap

- [x] Schema management
- [x] Collection (metadata) management
- [x] Document creation
- [x] Indexing
- [ ] CRUD operations (other than create)
- [ ] Database metadata management
- [ ] Server for accessing the database
- [ ] Querying (basic, fields other than ID)
- [ ] Secondary indices
- [ ] Custom query language (GraphQL-like, allows for easy joins)
- [ ] Basic security (authentication, authorization)
- [ ] FHE support (encryption, decryption, processing)

#### Why?

Fun
