---
id: DESIGN-MODULE-LAYOUT
status: accepted
origin: requirements
sourceOfTruth: model
---

# Module and crate layout

Every module crate, in this repository or outside it, has the same structure (DEC-087). The server reads only the module's descriptor; the layout tells developers, reviewers and `tools/repo-check` where each declaration lives.

## The descriptor

`src/module.rs` exposes the module as the server reads it:

```rust
pub const NAME: &str = "identity";

pub const MODULE: Module = Module {
    name: NAME,
    manifest: kampus_common::manifest!(),
    summary: "Accounts, credentials, sign-in, second factors, sessions and API tokens",
    steps: crate::db::STEPS,
    permissions: crate::permissions::PERMISSIONS,
    templates: crate::permissions::TEMPLATES,
    plugin_types: &[],
    commands: crate::commands::COMMANDS,
    preferences: crate::settings::PREFERENCES,
    settings: crate::settings::SETTINGS,
    terms: &[],
    flags: &[],
    field_kinds: &[],
    page_kinds: &[],
    widgets: &[],
    menu_kinds: &[],
    rule_actions: &[],
    file_areas: &[],
    activity_types: &[],
    events: crate::events::EVENTS,
    listeners: crate::observers::LISTENERS,
    jobs: crate::jobs::JOBS,
    schedules: crate::jobs::SCHEDULES,
    routes: Some(crate::http::routes),
};
```

| Field | Comes from | The server |
| --- | --- | --- |
| `name` | `src/module.rs` | Names the module's database schema and owns its grants |
| `manifest` | `version.toml`, `Cargo.toml` and `assets/icon.svg`, read by `manifest!()` | Checks it and records the version (see [the version file](#the-version-file)) |
| `commands` | `src/commands/mod.rs` | Offers them as `kampus-server <module> <command>` (see [commands](#commands-of-a-component)) |
| `settings` | `src/settings/mod.rs` | Records the definitions; values are set per scope and resolved through the tree (see [settings](#settings)) |
| `terms`, `flags`, `field_kinds`, `page_kinds`, `widgets`, `menu_kinds`, `rule_actions` | `src/settings/customise.rs` | Records the words, features, kinds of entity, pages, widgets, menus and rule actions a campus customises (see [customisation](#customisation)) |
| `file_areas` | `src/settings/files.rs` | Records the places the module keeps files, with their limits and who reads them (see [file areas](#file-areas)) |
| `activity_types` | `src/settings/` | Registers the types of activity and material the module provides (see [activity types](#activity-types)) |
| `preferences` | `src/settings/preferences.rs` | Kept by users per person: only declared keys, each checked against its type, with its default when unset |
| `plugin_types` | `src/module.rs` | Accepts plugins named `<type>_<name>` in `crates/<module>/<type>/<name>` |
| `steps` | `src/db/steps.rs` | Applies the pending upgrade steps (DEC-082) |
| `permissions` | `src/permissions/mod.rs` | Synchronises them at start-up and retires those no module declares (RN-021) |
| `templates` | `src/permissions/templates.rs` | Merges the values of every module for each role template |
| `events`, `listeners` | `src/events/mod.rs`, `src/observers/mod.rs` | Publishes nothing itself: it records the types, and runs the listeners in the worker roles (see [events and listeners](#events-and-listeners)) |
| `jobs`, `schedules` | `src/jobs/mod.rs` | Runs the jobs in the worker roles and fires the schedules in the scheduler roles (see [background jobs](#background-jobs)) |
| `routes` | `src/http/mod.rs`, `None` without routes | Serves them under `/api/v1`, behind the campus of the request and its authentication (see [routes](#routes)) |

## The folders

| Path | Holds | Required |
| --- | --- | --- |
| `Cargo.toml` | The package `kampus-<module>`, with `[package.metadata.kampus] kind = "module"` | yes |
| `src/lib.rs` | The crate documentation, the service type and its constructor, the module list and re-exports | yes |
| `src/module.rs` | `NAME` and `MODULE` | yes |
| `src/db/` | `steps.rs`, the upgrade steps in the neutral schema (DEC-065); queries stay with the services that run them | yes |
| `src/permissions/` | `mod.rs`, the permission codes and declarations; `templates.rs`, the values given in role templates | yes |
| `src/model/` | Entities, identifiers and values | when it has any |
| `src/service/` | The service API, one file per area of use cases (`account.rs`, `sign_in.rs`) | when it has any |
| `src/ports/` | Traits that other modules implement for this one (`Courier`, `Policies`) | when it has any |
| `src/support/` | Internal helpers that are not part of the API (hashing, encodings) | when it has any |
| `src/jobs/` | `mod.rs` with `JOBS`, `SCHEDULES` and their handlers, which call the services | when it has any |
| `src/commands/` | Its commands on the command line, `kampus-server <module> <command>` | when it has any |
| `src/events/` | `mod.rs` with `EVENTS` and the codes of the events it publishes | when it has any |
| `src/observers/` | `mod.rs` with `LISTENERS` and their handlers, which call the services | when it has any |
| `src/settings/` | `mod.rs` with `SETTINGS`, kept per scope, and `preferences.rs` with the per-person preferences (type and default) | when it has any |
| `src/http/` | `mod.rs` with `routes`, and one file per area of routes with their request and response types | when it has any |
| `src/privacy/` | Its export and erase handlers (DEC-061) | from the privacy module (T-C.12) |
| `tests/` | Integration tests against the real ports, one test per business rule or use case | yes |
| `web/` | Its part of the web client (DEC-004) | from the web client (T-C.13) |
| `lang/` | Its messages (DEC-062) | from the i18n module |
| `README.md` | Optional notes for developers | no |

Nothing else may sit at the top of the crate or of `src/`; `tools/repo-check` reports any other file or folder, a missing descriptor, and a registry out of date.

## Routes

A module serves HTTP by exposing `routes` in its descriptor: a function that builds its `axum::Router` from the `ServiceContext`, the services the server composed once per process. A handler reads the services it needs by type, never builds one itself.

```rust
pub fn routes(context: &ServiceContext) -> Router {
    let shared = Shared { users: context.service::<Users>().ok() };
    Router::new()
        .route("/me/profile", get(my_profile).put(update_my_profile))
        .with_state(shared)
}
```

The server serves the routes of each module under `/api/v1/<module>`, the name of the module: the route above is `/api/v1/users/me/profile`. A module writes its paths without that prefix and never reaches into the prefix of another, so two modules, or one written later, cannot collide whatever routes they add; a path inside a module does not repeat the name of the module either (`/api/v1/customise/imports`, not `/api/v1/customise/customisation-imports`). Links a module builds to its own routes, such as the return address of a sign-in through a provider, use its name from `src/module.rs`.

Before any handler the server runs two layers:

| Layer | Owner | What a handler finds |
| --- | --- | --- |
| Campus | `kampus-server` (`http/campus.rs`) | `CurrentCampus`: the campus of the installation, or of the request's domain with a campus per domain |
| Authentication | `kampus-identity` (`http/mod.rs`) | `Caller`: the actor, the session or API token, the campus and where the request comes from |

`Caller` gives what each kind of route needs: `session()` for routes that manage credentials, which API tokens never do; `account()` for the person signed in; and `acting(identity, permission, scope)` for an operation that needs a permission, which also limits an API token to the permissions it carries. The service still checks the permission itself.

A browser authenticates with the `kampus_session` cookie (HttpOnly, SameSite=Lax, Secure over HTTPS), and every request that changes something with it carries the `X-Kampus` header, which other sites cannot send. An integration sends an API token as `Authorization: Bearer kmp_...`.

`kampus_module::http` holds what every module's routes share:

| Item | Use |
| --- | --- |
| `ApiResult<T>` and `ApiError` | Return a `kampus_common::Error` with `?`; it is answered as problem details (RFC 9457) with the request id |
| `Body<T>` | A JSON body; one that does not parse is a `validation` problem (`module.body_invalid`) |
| `id::<T>(segment)` | An identifier from the path; one that does not parse is `not_found`, as an unknown one |
| `if_match(headers)` | The version an update was made on; required (DEC-026), `precondition_failed` without it |
| `with_etag(response, version)` | Answers a resource with its version as `ETag` |

The routes of a module live in `src/http/`: `mod.rs` with `routes` and what its handlers share, and one file per area (`auth.rs`, `me.rs`, `accounts.rs`). The OpenAPI fragment of the module in `docs/03-design/api/` lists every route. Tests of routes go through the server, in `crates/server/tests/integration/`, as a client sees them.

## Events and listeners

A module tells the others what changed by **publishing events**, and reacts to theirs with **listeners**; no module reads another's tables to find out. Events are declared in `src/events/mod.rs` and listeners in `src/observers/mod.rs`, exposed through `events` and `listeners` in the descriptor:

```rust
pub const EVENTS: &[EventDeclaration] = &[EventDeclaration {
    code: "enrol.enrolment_removed",
    version: 1,
    summary: "An enrolment was removed",
    fields: &["enrolment", "course", "account"],
    audited: true,
    personal_fields: &[],
}];

pub const LISTENERS: &[ListenerDeclaration] = &[ListenerDeclaration {
    event: "users.system_group_members_changed",
    name: "sync_linked_instances",
    run: sync_linked_instances,
}];
```

Publish inside the transaction of the change, with `Events::publish_in`: the event exists only if the change commits.

```rust
let mut event = Publish::new(ENROLMENT_REMOVED, Some(enrolment.as_uuid()), json!({ "enrolment": id, "course": course, "account": account }));
event.summary = Some(format!("enrolment {id} removed"));
Events::new(self.db.clone()).publish_in(&mut tx, event).await?;
```

| Rule | What it means for a module |
| --- | --- |
| Declared payload | Every field of `fields` is in the payload, or publishing fails; a change that breaks listeners is a new `version` |
| After commit | Listeners run after the transaction commits, never inside it, so a failing listener never undoes the change |
| At least once | A listener may receive an event twice: it uses the event's id or the state it finds, and does nothing twice |
| In order per object | Events about one `object` reach a listener in the order they happened; a delivery that keeps failing holds the next ones of that object until it is dead |
| Retries and dead letters | A failing delivery is retried with a wait that doubles, eight times, then it is dead and an operator replays it |
| Chains | A listener that publishes passes the event it handles as `cause`; a chain stops after ten events |
| Every event | A listener of `events.any` receives the events of every type; nothing publishes `events.any` itself |
| Audit | An `audited` type writes an append-only entry of the audit log in the same transaction, with its `summary` |

Payloads hold identifiers, not copies of personal data; the fields that do hold it are listed in `personal_fields` for the privacy handlers. The `worker` and `all` roles run the deliveries; operators see and replay them with `kampus-server events`.

## Settings

What a campus, a category, a course or a section may configure is a **setting** the module declares in `src/settings/mod.rs` and exposes through `settings` in its descriptor. Values are kept per scope and resolved by `kampus-settings`:

```rust
pub const SETTINGS: &[SettingDeclaration] = &[SettingDeclaration {
    key: "courses.group_mode",
    summary: "Whether people work in groups: none, separate or visible",
    kind: ValueKind::Choice { options: &["none", "separate", "visible"], default: "none" },
    levels: &[ScopeKind::Course, ScopeKind::Section, ScopeKind::Activity],
    permission: GROUPS_MANAGE,
    restart_required: false,
}];
```

| Field | Meaning |
| --- | --- |
| `key` | `<module>.<name>`, unique in the build |
| `kind` | `Boolean`, `Integer` with its bounds, `Text` with its length or `Choice` with its options, and the default; a value of another kind is refused |
| `levels` | The kinds of scope where a value may be set; anywhere else it is refused |
| `permission` | The permission, declared by the module, that sets it in a scope |
| `restart_required` | The roles that read it apply a change only at their next start |

Read a value with `Settings::resolve(key, scope)`, or every setting of a module at once with `Settings::resolve_prefix("<module>.", scope)`: the value of the nearest scope that sets it wins, unless a scope above set it **locked**; without any, the default. The answer says where the value came from. Every change records who made it with the value before and after, and publishes `settings.changed`.

A module never reads a setting it does not declare, and never keeps its own table of options: identity, for example, reads the policy of a campus (`identity.*`) through its `Policies` port from settings.

## Customisation

What a campus changes without code beyond settings, a module offers in `src/settings/customise.rs` and exposes through `terms`, `flags`, `field_kinds`, `page_kinds`, `widgets`, `menu_kinds` and `rule_actions` in its descriptor; `kampus-customise` keeps it per scope and resolves it like settings, from the nearest scope up to the platform, sections included, unless a scope above **locks** its value. Every answer says where it came from (declared, or the scope that set it), every change is recorded with before and after and publishes `customise.changed`.

```rust
pub const TERMS: &[TermDeclaration] = &[TermDeclaration {
    code: "courses.course",
    forms: &[
        TermForms { language: "en", singular: "course", plural: "courses" },
        TermForms { language: "es", singular: "curso", plural: "cursos" },
    ],
}];
pub const FLAGS: &[FlagDeclaration] = &[FlagDeclaration {
    code: "courses.groups",
    summary: "Groups and group sets in courses",
    default: true,
}];
pub const FIELD_KINDS: &[FieldKindDeclaration] = &[FieldKindDeclaration {
    code: "courses.course",
    summary: "Courses",
}];
```

| Declaration | A scope | The module |
| --- | --- | --- |
| A **term**, with its singular and plural per language | Renames it per language (`customise.term.manage`); a language the module does not declare falls back to its first | Shows `Customise::term(code, scope, language)` instead of a fixed word |
| A **flag**, with its default | Turns the feature on or off (`customise.flag.manage`); its data stays | Guards what the feature offers with `Customise::require_flag(code, scope)`, which answers `conflict` (`customise.feature_off`) |
| A **kind of entity** | Defines typed custom fields for it (`customise.field.manage`): text, integer, date, boolean, choice, choices or account, public or private (`customise.field.view`) | Keeps a link table `<entity>_field` (entity, field, value, all `CASCADE`), writes values with `Customise::write_value_in` inside its own transaction and reads them with `Customise::values` |
| A **page kind**, with its regions | Arranges widgets in its regions (`customise.layout.manage`, which teachers hold in their courses), each with its settings and, optionally, the permission that shows it; the layout of the nearest scope applies whole | Shows the placements of `Customise::layout(actor, page_kind, scope)`, or its own arrangement when the origin is declared; a widget no module declares any more is left out, never breaks the page |
| A **widget**, with the page kinds it fits and its typed settings (`WidgetSetting` with a `ValueKind`) | Places it; a setting it does not declare, or of another kind, is refused, and the defaults fill what is left out | Renders it from its settings |
| A **kind of menu** | Sets its items (`customise.menu.manage`): a label, a path or web address, a permission, at most 3 levels | Shows `Customise::menu(actor, kind, scope)`: only the items the reader may see |
| A **rule action**, with typed parameters (`RuleParameter`), the payload fields it reads (`needs`), the permission its rule owner needs and the events it may publish | Writes rules (`customise.rule.manage`): when an event of a declared type happens in the scope or below and the conditions on its declared fields hold, the actions run in order | Runs the action from `RuleActionRun` (event, parameters with defaults, owner, services), does nothing twice, and publishes with the event as `cause` |

Customisation never owns an entity of another module: the values of custom fields belong to their entity through the owning module's link table, so there is no polymorphic owner, and deleting the entity, the field or the value takes the link with it.

The theme of a scope is not declared by modules: it is a fixed set of design tokens (colours, font, sizes, motion) that each scope may set over those above it, checked for WCAG 2.2 AA contrast (`customise.theme.manage`). Modules style their pages with the `--kampus-*` custom properties of `GET /scopes/{scope}/theme.css`, never with colours of their own.

## External references

Every record an integration touches carries the identifiers other systems give it, as external references (system, identifier), never as a column of the record and never in its human `code`. A module adds one table per kind of record in a step, with `kampus_db::reference_table!("course_reference", "course")`, and reaches it through `kampus_db::references::References::new(&db, module::NAME, "course_reference")`: `set`/`set_in`, `remove`, `find` and `of`. `ExternalReference::new(system, external_id)` checks and trims what a caller sends.

| A module | Does |
| --- | --- |
| Sets and removes references | With the permission that manages the record, which also decides who reads them |
| Finds a record by one | `GET /api/v1/<module>/campuses/{campus}/<records>/by-reference/{system}/{external_id}` |
| Creates or updates by one, when integrations create the record | Sets the reference in the transaction that creates the record, so it never exists without it; `PUT …/by-reference/{system}/{external_id}` answers 201 or 200 |

## Activity types

A module that provides activities or materials declares their **types** in `src/settings/` and exposes them through `activity_types` in its descriptor; `kampus-activities` registers them, places course modules of them in sections, and never reads what a type keeps of its own:

```rust
pub const TYPES: &[ActivityTypeDeclaration] = &[ActivityTypeDeclaration {
    code: "activities.page",
    kind: ActivityKind::Material,
    summary: "A page of text",
    features: &["copy"],
    settings: &[ActivitySetting { key: "content", kind: ValueKind::Text { max: 20_000, default: "" } }],
}];
```

A course module is created, updated, copied and moved in one transaction with the settings its type declares, each checked against its kind with the defaults filled in. A type that keeps data of its own references `activities.course_module` with a foreign key and `CASCADE`, in its own table, and an activity that owns files declares a file area whose `readable` asks `Activities::visible_to`.

## File areas

A module that keeps files declares its **areas** in `src/settings/files.rs` and exposes them through `file_areas` in its descriptor; `kampus-files` stores the content once and runs the one upload pipeline, and the module owns its files through a link table of its own:

```rust
pub const FILE_AREAS: &[FileAreaDeclaration] = &[FileAreaDeclaration {
    code: "users.picture",
    summary: "The profile picture of a person",
    max_size: 5 * 1024 * 1024,
    allowed_types: &["image/png", "image/jpeg", "image/gif", "image/webp"],
    upload_permission: PROFILE_UPDATE_OWN,
    readable: picture_readable,
}];
```

| Field | Meaning |
| --- | --- |
| `max_size`, `allowed_types` | The largest file and the media types it accepts, exact or as a family (`image/*`); the type is read from the content, never taken from the sender |
| `upload_permission` | The permission an uploader holds in the scope the upload is for |
| `readable` | Whether a principal may read one of its files, given the file: when they may read the record that owns it. Files asks it, and has no other way to grant reading |

The owning module keeps a table `<entity>_file` (or a column in a table of its own) with foreign keys to its record and to `files.file`, both `CASCADE`, and calls `Files::attach_in` and `Files::detach_in` inside the transaction that links or unlinks them: the count of the uses of the content moves with them, and a file nobody attaches is removed after a day. A file is read through `POST /api/v1/files/files/{id}/url`, a link of a few minutes that never runs content in the platform's origin.

## Background jobs

Work that does not belong in a request, or that recurs, is a **job** in the queue of `kampus-tasks`. A module declares its kinds of job and their schedules in `src/jobs/mod.rs` and exposes them through `jobs` and `schedules` in its descriptor:

```rust
pub const JOBS: &[JobDeclaration] = &[JobDeclaration {
    code: "enrol.run_expiry",
    summary: "Apply the expiry rule to the enrolments whose end has passed",
    max_attempts: 5,
    backoff_seconds: 60,
    timeout_seconds: 1800,
    concurrency: Some("all"),
    run: run_expiry,
}];

pub const SCHEDULES: &[ScheduleDeclaration] = &[ScheduleDeclaration {
    job: "enrol.run_expiry",
    cron: "5 * * * *",
    time_zone: "UTC",
}];

fn run_expiry(run: JobRun<'_>) -> JobFuture<'_> {
    Box::pin(async move {
        run.services.service::<Enrol>()?.run_expiry().await?;
        Ok(())
    })
}
```

| Field | Meaning |
| --- | --- |
| `code` | `<module>.<name>`, unique in the build |
| `max_attempts` | Runs before the job is failed, with its last error kept and logged |
| `backoff_seconds` | The wait before the first retry; it doubles at each one |
| `timeout_seconds` | How long one run may take; a run past it is stopped and counts as a failed attempt. The worker's lease lasts as long, so a job whose worker stops is taken again |
| `concurrency` | A payload field: jobs of the type with the same value run one at a time (`Some("course")`); a field absent from the payload, such as `"all"`, keeps the whole type one at a time |
| `run` | The handler; it gets the job's id, its payload and the [`ServiceContext`](#routes) |

A handler may run again after a failure or a stopped worker, so it does what is left and nothing twice. Payloads hold identifiers, never copies of personal data. A long job reports its progress with `Tasks::report_progress` and stops at its next step when `Tasks::cancelled` says so.

To enqueue a job from a change, use `Tasks::enqueue_in` inside the change's transaction: the job exists only if the change commits.

The server synchronises job types and schedules at start-up; a schedule keeps the rule an operator set. The `worker` and `all` roles run jobs; the `scheduler` and `all` roles fire schedules, once per instant across every node. Operators see and steer them with `kampus-server tasks`.

## Commands of a component

A module offers commands on the command line in `src/commands/mod.rs`, listed in the `commands` field of its descriptor. The server reads them from the registry, so every module, inside the repository or not, gets its commands without any change to the server:

```text
kampus-server identity --help          lists the commands of identity
kampus-server identity purge           runs one
kampus-server access grant <user> [until]
```

```rust
pub const COMMANDS: &[Command] = &[Command {
    name: "purge",
    summary: "Remove expired one-time tokens and sessions",
    arguments: &[],
    run: purge,
}];

fn purge<'c>(context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async move {
        let purged = identity(context)?.purge().await?;
        Ok(vec![format!("{} session(s) removed", purged.sessions)])
    })
}
```

A command receives the database of the installation and its secret key in a `CommandContext`, gets its arguments already counted against its declaration, and returns the lines it reports. Names are in kebab case; a wrong number of arguments answers with the usage line.

## Tests

Tests never live in the code: no `#[cfg(test)]` in `src/`. Each crate keeps them in `tests/`, in one folder per kind; each folder is one test binary whose `main.rs` declares one file per topic:

```text
tests/
  unit/          pure checks, without databases or services
    main.rs      mod keys; mod schema; …
    schema.rs
  integration/   against PostgreSQL and the services of Docker Compose
    main.rs
    rules.rs     one test per behaviour
  conformance/   adapters: the conformance suite of their port
    main.rs
  support/       helpers shared by the folders above, included with #[path]
```

Tests go through the public API of the crate, as any caller would. `tools/repo-check` rejects a `#[cfg(test)]` in `src/`, a file directly in `tests/`, any other folder, and a folder without its `main.rs`.

## Every other crate

Foundation, port, adapter and server crates follow the same principle: `src/` holds only its entry files, and everything else lives in a folder named after its concern, each with a `mod.rs`. Folder names are snake case. Public paths do not depend on folders: `lib.rs` re-exports what callers use.

| Kind | At the top of `src/` | Required folders | Also at the crate root |
| --- | --- | --- | --- |
| `foundation` | `lib.rs` | — | — |
| `port` | `lib.rs` | `port/` (the trait and its types), `registry/` (`Adapter` and `open`), `conformance/` (the suite every adapter passes) | `adapters/` |
| `adapter` | `lib.rs` with `ADAPTER` | `store/` (the implementation) | — |
| `server` | `lib.rs`, `main.rs` | `commands/`, `config/`, `http/`, `wiring/` (the generated registries and the composition) | `adapters.toml`, `modules.toml` |
| `module` | `lib.rs`, `module.rs` | `db/`, `permissions/`, and the folders above | `web/`, `lang/`, `assets/` |

The crates of the repository today:

| Crate | Folders of `src/` |
| --- | --- |
| `kampus-common` | `adapter/` (settings of adapters), `clock/` (instants and clocks), `error/` (the error model), `id/` (typed identifiers), `page/` (pagination), `request/` (request context) |
| `kampus-db` | `connection/` (pool, prefix, error classes), `schema/` (the neutral schema), `upgrade/` (steps and the runner), `lint/` (the schema linter) |
| `kampus-module` | `descriptor/` (`Module`), `permission/` (the vocabulary of declarations) |
| `kampus-storage`, `kampus-cache` | `port/`, `registry/`, `conformance/`; the cache adds `layered/` |
| `kampus-server` | `commands/` (serve, upgrade, check), `config/`, `http/`, `wiring/` |

`tools/repo-check` rejects a file at the top of `src/` that is not an entry file, a folder without `mod.rs`, a missing required folder, and a crate without its kind.

## Names and folders

The name of a component comes from its package, and its folder follows from its name:

| Component | Package | Name | Folder |
| --- | --- | --- | --- |
| Core module | `kampus-identity` | `identity` | `crates/identity` |
| Adapter of the `cache` port | `kampus-cache-redis` | `cache_redis` | `crates/cache/adapters/redis` |
| Plugin of a type that a module defines | `kampus-<type>-<name>` | `<type>_<name>` | `crates/<owner>/<type>/<name>` |

Plugins and adapters carry their type as prefix, so their names never collide with the core. Only plugins and adapters may come from outside the repository; they are declared in `crates/server/modules.toml` or `crates/server/adapters.toml`. A module that defines a plugin type lists it in `plugin_types` of its descriptor.

`tools/repo-check` checks that every package matches its folder. The server checks names, versions, supported releases and requirements before it installs, upgrades or starts, and refuses with the list of problems.

## The version file

Every component has a `version.toml` next to its `Cargo.toml`, in the same form as the platform's `version.toml` at the root. The descriptor reads it with `kampus_common::manifest!()`, so no version data is written in code:

```toml
[component]
version = "1.2.0"
maturity = "beta"
kampus = ">=0.1.0, <0.3.0"
date = "2026-11-19"

[requires]
access = ">=0.1.0"
```

```rust
manifest: kampus_common::manifest!(),
```

| Key | Default | Meaning |
| --- | --- | --- |
| `component.version` | required | The version, semantic; equal to the package version |
| `component.maturity` | `stable` | `alpha`, `beta`, `rc` or `stable` |
| `component.kampus` | any release | The Kampus releases it works with, as a Cargo version requirement |
| `component.date` | none | The date of its release, `YYYY-MM-DD` |
| `component.icon` | `assets/icon.svg` when it exists, otherwise a generic icon | Another path, relative to the crate, only when the icon lives elsewhere |
| `[requires]` | none | Other components it needs, each with the versions it works with |

Name, summary and licence come from `Cargo.toml`. Everything but the version is optional: a component without it builds and runs, and `kampus-server components` shows what it lacks. A version that differs from the package, an unknown maturity, a malformed date or an icon that does not exist stops the build with the file and the key. Core components share the Kampus version (`version.workspace = true`); external ones have their own.

## Versions and the registry

The installation records the installed version of every component and every run that changed it:

| Table | Holds |
| --- | --- |
| `component` | Each installed component: kind, origin, version, maturity, when it was installed and last changed |
| `upgrade_run` | Each upgrade, downgrade or uninstall: the releases before and after, when, and how it ended |
| `component_change` | What each run did to each component: versions before and after, and how many steps |

A component with a new version needs an upgrade, even when it has no new steps. A component installed in a newer version than the build, or installed but missing from the build, stops the start: the operator downgrades, adds it back, or uninstalls it.

## Install actions and data migrations

A step may carry an action: Rust code that runs after the step's changes, in the same transaction, once. The action of step 1 is what the component sets up when it is installed; actions of later steps migrate data:

```rust
Step {
    sequence: 1,
    name: "create notes",
    revert: &[],
    action: Some(Action { name: "first note", run: first_note }),
    changes: &[/* tables */],
}

fn first_note<'c>(conn: &'c mut PgConnection, context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async move {
        sqlx::query(sqlx::AssertSqlSafe(format!("insert into {} (id) values ($1)", context.table("note"))))
            .bind(uuid::Uuid::now_v7())
            .execute(conn)
            .await
            .map(drop)
            .map_err(kampus_db::classify)
    })
}
```

If the action fails, the step's changes are undone with it and the run is recorded as failed. The action's name is part of the step's checksum, so renaming it after a release counts as changing the step.

## Going back

Versions never go back by themselves. A step that can be undone says how in `revert`; the SQL is kept in the installation when the step is applied, so an older build can undo steps it has never seen:

```rust
Step {
    sequence: 2,
    name: "add note text",
    revert: &[Change::DropColumn { table: "note", column: "body", unused_since: "1.1.0" }],
    action: None,
    changes: &[Change::AddColumn { table: "note", column: Column::new("body", Type::Text).nullable() }],
}
```

`kampus-server downgrade --yes`, run with the older build, undoes every newer step, newest first, and takes the components back to the build's versions. When a newer step left no way back, nothing is undone and the command names the steps. Actions are not undone: a step whose action changes data must say in `revert` how to bring the data back, or leave `revert` empty.

## Uninstalling

`kampus-server uninstall <component> --yes` removes a component that is installed but no longer in the build: its tables, its steps in the ledger and its registration. It is refused while a component of the build requires it.

## Commands of the operator

| Command | Does |
| --- | --- |
| `kampus-server components [--json]` | Lists the components of the build and of the installation: versions, maturity, state and what each lacks |
| `kampus-server history [--limit N]` | Shows the latest upgrades, downgrades and uninstalls, with what each did to each component |
| `kampus-server check` | Says whether the installation matches the build, without changing it |
| `kampus-server upgrade` | Applies the pending steps and registers the components |
| `kampus-server downgrade --yes` | Goes back to the build's versions |
| `kampus-server uninstall <component> --yes` | Removes a component no longer in the build |
| `kampus-server copy-storage <url>` | Copies every stored file to another store, verifying each by its hash; a new run skips what is there. Switch `KAMPUS_STORAGE_URL` to it afterwards, then delete the old store |

## Registering a module

The server's registry, `crates/server/src/wiring/registry.rs`, lists every module of the build in dependency order, and the `# modules:start` block of `crates/server/Cargo.toml` depends on each. `tools/repo-check --fix` writes both:

- from every crate of kind `module` under `crates/`, ordered by the module table of the [architecture](./architecture.md#modules);
- then from `crates/server/modules.toml`, for modules published outside the repository:

```toml
[[module]]
crate = "kampus-attendance"
source = 'git = "https://example.org/kampus-attendance", tag = "v1.0.0"'
```

An external module depends only on published Kampus crates, follows this layout, and passes the same checks.

## Adding a kind of contribution

A new kind (events, settings, routes…) is introduced by the core module that owns it, in one pull request:

1. a field in `kampus_module::Module`, documented with the folder it comes from;
2. the folder in `SRC_FOLDERS` of `tools/repo-check/modules.mjs`, and, when every module must have it, a line in `REQUIRED`;
3. what the server does with it, in `crates/server/src/wiring/`;
4. its row in the tables above, and its section in the platform and plugin developer guides.
