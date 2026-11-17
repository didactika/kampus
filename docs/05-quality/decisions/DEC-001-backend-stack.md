---
id: DEC-001
status: accepted
---

# DEC-001 Backend in Rust with Axum, SQLx and PostgreSQL

- Date: 2026-11-17
- Context: Kampus is a Rust platform; the stack must be mature, fast and safe, with proven libraries for web services and database access.
- Options considered: Axum or Actix Web; SQLx, SeaORM or Diesel; PostgreSQL or several databases from the start.
- Decision: Axum on Tokio with tower middleware, and SQLx drivers. PostgreSQL is the first database. Other databases come through the database port (DEC-009).
- Consequences: The first release has one database to tune. The design keeps the data model portable from day one, so MySQL, MariaDB, SQLite, SQL Server and Oracle can be added without rewriting modules (DEC-009).
- Affected artifacts: the project configuration (`03-design/project-configuration.md`), `crates/`.
