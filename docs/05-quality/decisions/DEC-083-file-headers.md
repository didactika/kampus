---
id: DEC-083
status: accepted
---

# DEC-083 A standard header in every file and documentation comments above every item

- Date: 2026-11-18 (header format revised the same day: banner, AGPL notice, author with address)
- Context: Files moved or copied out of context lose their origin, ownership and licence; code without documentation above its items is hard to maintain, while narrative comments inside bodies go stale.
- Options considered: no headers; a licence line only; a full standard header plus documentation comments on items, checked in CI (chosen).
- Decision:
  - Every source and configuration file outside `docs/` starts with the Kampus header, in the comment syntax of its type: the KAMPUS banner in the standard FIGlet font with the name of the platform, the project line, the notice of the GNU Affero General Public License (DEC-085), then `@package` (the crate or component), `@copyright` (the year and Didactika), `@author` (name and address of who created the file), `@license` and `@since` (the version that added it), and the SPDX identifier. In Rust:

    ```rust
    // ------------------------------------------------------------------------------
    //
    //     _  __    _    __  __ ____  _   _ ____
    //    | |/ /   / \  |  \/  |  _ \| | | / ___|
    //    | ' /   / _ \ | |\/| | |_) | | | \___ \
    //    | . \  / ___ \| |  | |  __/| |_| |___) |
    //    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
    //
    //    Knowledge Academic Modular Platform for Universal Study
    //
    // ------------------------------------------------------------------------------
    //
    // This file is part of Kampus - https://github.com/didactika/kampus
    //
    // Kampus is free software: you can redistribute it and/or modify it under the terms of the
    // GNU Affero General Public License as published by the Free Software Foundation, either
    // version 3 of the License, or (at your option) any later version.
    //
    // Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
    // even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    // GNU Affero General Public License for more details.
    //
    // You should have received a copy of the GNU Affero General Public License along with Kampus.
    // If not, see <https://www.gnu.org/licenses/>.
    //
    // @package   kampus-courses
    // @copyright 2026 Didactika
    // @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
    // @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
    // @since     0.1.0
    //
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // ------------------------------------------------------------------------------
    ```

    TOML, YAML, shell and Dockerfiles use `#`, CSS `/* */`, Svelte and HTML `<!-- -->`, SQL `--`, Astro components put it inside their front matter.
  - Files that cannot hold comments (JSON, lock files, images) and the specification in `docs/` are exempt.
  - Every item gets a documentation comment above it, public or not (Rust: `//!` for modules, `///` for items; TypeScript and Svelte: TSDoc), saying what it is for, not how it works. Comments inside bodies are kept for what the code cannot say.
  - `tools/repo-check` fails when a file lacks its header and adds missing headers with `--fix`; the Rust lints `missing_docs` and `missing_docs_in_private_items` fail on undocumented items.
- Consequences: Every file says what it belongs to and under which terms; the generated references (DEC-078) have text for every item.
- Affected artifacts: every file, `tools/repo-check`, the workspace lints.
