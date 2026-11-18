---
id: DEC-079
status: accepted
---

# DEC-079 Table ownership by declaring module, supertypes in the lowest module, and checked dependencies

- Date: 2026-11-18
- Context: Checking every data model against the module ranking found foreign keys pointing to later modules: grants referenced accounts while accounts referenced scopes (a cycle between access and identity), settings and languages referenced modules after them, a grade condition lived in activities but referenced records, custom fields referenced courses and files from a lower module, and profile languages and profile fields duplicated what languages and customisation provide.
- Options considered: allow cross-level foreign keys and rely on code; merge the modules involved; fix ownership and order with explicit rules (chosen).
- Decision:
  - Concepts referenced by many modules get a supertype in the lowest module that needs it: `access.principal` is the supertype of accounts, so access never references identity.
  - The module that declares a type used by another module's entity owns the leaf or link table: grade conditions are in `records`, profile custom fields link from `users`, attachments link from their owner.
  - Level 1 is ordered access, settings, identity, i18n; access is configured by its own declarations, not by settings; customisation moves to level 2 after files.
  - A person's language is a column of the profile and a course's forced language a column of the course; languages keep no copy. Profile fields are custom fields of the entity kind `account`.
  - The documentation check fails when a data model has a foreign key to a later module.
- Consequences: The dependency graph of the schemas is acyclic and matches the crates; one place for each fact.
- Affected artifacts: `docs/03-design/architecture.md`, the data models of access, identity, settings, i18n, courses, customise, activities and records, `tools/docs-check`.
