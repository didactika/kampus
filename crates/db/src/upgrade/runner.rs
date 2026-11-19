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
// @package   kampus-db
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The upgrade runner: from any supported release, every pending step of every module is applied in one
//! run, in dependency order, each in its own transaction with its action, and recorded in a ledger with
//! the SQL that undoes it. The version of every component is registered, and every run is kept in the
//! history. A database whose steps changed, that is newer than the code, or that has components the code
//! lacks, is refused; going back to an older release is a separate, explicit operation.

use std::collections::BTreeMap;
use std::fmt;

use kampus_common::{Error, Result, Timestamp};
use semver::Version;
use sha2::{Digest, Sha256};
use sqlx::Row;

use super::registry::{self, ComponentChange, ComponentRecord, Run};
use crate::schema::Change;
use crate::{ActionContext, Database, MODULE, ModuleSteps, Step};

/// The steps of this crate: the record of the installation, next to the ledger, and the registry of
/// components with the history of runs.
pub const OWN_STEPS: ModuleSteps = ModuleSteps {
    module: MODULE,
    steps: &[
        Step {
            sequence: 1,
            name: "installation record",
            revert: &[],
            action: None,
            changes: &[Change::Postgres {
                sql: "create table {schema}.installation (
            singleton boolean primary key default true check (singleton),
            prefix text not null,
            installed_version text not null,
            installed_at timestamptz not null default now(),
            upgraded_version text not null,
            upgraded_at timestamptz not null default now()
        );",
                reason: "kept as first written: changing its text would change the checksum of a step already applied",
            }],
        },
        Step {
            sequence: 2,
            name: "components and the history of runs",
            revert: &[],
            action: None,
            changes: registry::TABLES,
        },
    ],
};

/// Something that stops an upgrade or the start of the platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A step applied earlier differs from the one in the code.
    ChangedStep {
        /// The module of the step.
        module: String,
        /// The sequence of the step.
        sequence: u32,
    },
    /// The database has a step the code does not know: it is newer than the code.
    UnknownStep {
        /// The module of the step.
        module: String,
        /// The sequence of the step.
        sequence: u32,
    },
    /// The database was upgraded by a newer release than this code.
    NewerRelease {
        /// The release recorded in the database.
        recorded: String,
    },
    /// A component is installed in a newer version than the one in the code.
    NewerComponent {
        /// The component.
        component: String,
        /// The installed version.
        recorded: String,
        /// The version in the code.
        code: String,
    },
    /// A component is installed but no longer in the build.
    MissingComponent {
        /// The component.
        component: String,
        /// The installed version.
        version: String,
    },
    /// There is no installation with this prefix, but other installations share the database.
    OtherInstallations {
        /// The ledger schemas of the other installations.
        schemas: Vec<String>,
    },
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChangedStep { module, sequence } => {
                write!(f, "step {module}/{sequence} was changed after it was applied")
            }
            Self::UnknownStep { module, sequence } => write!(
                f,
                "step {module}/{sequence} is in the database but not in this code: the database is newer"
            ),
            Self::NewerRelease { recorded } => write!(
                f,
                "the database was upgraded by release {recorded}, newer than this code"
            ),
            Self::NewerComponent {
                component,
                recorded,
                code,
            } => write!(
                f,
                "{component} is installed in version {recorded}, newer than {code} in this build: run `kampus-server downgrade`"
            ),
            Self::MissingComponent { component, version } => write!(
                f,
                "{component} {version} is installed but not in this build: add it back, or run `kampus-server uninstall {component}`"
            ),
            Self::OtherInstallations { schemas } => write!(
                f,
                "no installation with this prefix, but the database holds others ({})",
                schemas.join(", ")
            ),
        }
    }
}

/// A step that is not applied yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingStep {
    /// The module of the step.
    pub module: &'static str,
    /// The sequence of the step.
    pub sequence: u32,
    /// What the step does.
    pub name: &'static str,
}

impl fmt::Display for PendingStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} {}", self.module, self.sequence, self.name)
    }
}

/// What an upgrade would do, and what stops it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradePlan {
    /// Whether the installation exists.
    pub installed: bool,
    /// The steps to apply, in order.
    pub pending: Vec<PendingStep>,
    /// The components to install or upgrade, with their versions.
    pub changes: Vec<ComponentChange>,
    /// What stops the upgrade or the start.
    pub problems: Vec<Problem>,
}

impl UpgradePlan {
    /// Whether the database matches the code exactly, so the platform may start.
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.installed && self.pending.is_empty() && self.changes.is_empty() && self.problems.is_empty()
    }
}

/// What an upgrade did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeReport {
    /// Whether the installation was created by this upgrade.
    pub installed: bool,
    /// The steps applied, in order.
    pub applied: Vec<PendingStep>,
    /// The components installed or upgraded.
    pub changes: Vec<ComponentChange>,
}

/// What a downgrade did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DowngradeReport {
    /// The steps undone, in the order they were undone.
    pub reverted: Vec<String>,
    /// The components taken back to the versions of the code.
    pub changes: Vec<ComponentChange>,
}

/// Plans and applies the upgrade of an installation to the steps and components of this code.
#[derive(Debug)]
pub struct Upgrader<'a> {
    /// The database of the installation.
    pub(super) db: &'a Database,
    /// The steps of every module, this crate's first, in dependency order.
    pub(super) modules: Vec<ModuleSteps>,
    /// The components of the build.
    pub(super) components: Vec<ComponentRecord>,
    /// The release of this code, recorded in the installation.
    pub(super) release: &'a str,
}

impl<'a> Upgrader<'a> {
    /// Prepares the upgrade with the steps of every module, given in dependency order.
    #[must_use]
    pub fn new(db: &'a Database, modules: &[ModuleSteps], release: &'a str) -> Self {
        let mut all = vec![OWN_STEPS];
        all.extend(modules.iter().copied().filter(|m| m.module != MODULE));
        Self {
            db,
            modules: all,
            components: Vec::new(),
            release,
        }
    }

    /// Registers the components of the build, so their versions are checked and recorded.
    #[must_use]
    pub fn with_components(mut self, components: Vec<ComponentRecord>) -> Self {
        self.components = components;
        self
    }

    /// The quoted ledger schema of the installation.
    pub(super) fn ledger(&self) -> String {
        self.db.prefix().schema(MODULE)
    }

    /// Works out the pending steps, the component changes and the problems, without changing anything.
    ///
    /// # Errors
    ///
    /// An `internal` error when a module declares steps out of order, and an `unavailable` error when the
    /// database cannot be read.
    pub async fn plan(&self) -> Result<UpgradePlan> {
        self.check_declarations()?;
        let mut conn = self.db.pool().acquire().await.map_err(unavailable)?;
        let ledger = self.ledger();
        let installed: bool = sqlx::query_scalar("select to_regclass($1) is not null")
            .bind(format!("{ledger}.migration"))
            .fetch_one(&mut *conn)
            .await
            .map_err(unavailable)?;

        let mut problems = Vec::new();
        let mut applied: Vec<(String, u32, Vec<u8>)> = Vec::new();
        if installed {
            let rows = sqlx::query(audited(format!(
                "select module, sequence, checksum from {ledger}.migration order by module, sequence"
            )))
            .fetch_all(&mut *conn)
            .await
            .map_err(unavailable)?;
            for row in rows {
                let sequence: i32 = row.get("sequence");
                applied.push((
                    row.get("module"),
                    u32::try_from(sequence).unwrap_or_default(),
                    row.get("checksum"),
                ));
            }
            if let Some(recorded) = self
                .recorded_release(&mut conn)
                .await?
                .filter(|recorded| newer(recorded, self.release))
            {
                problems.push(Problem::NewerRelease { recorded });
            }
        } else {
            let schemas: Vec<String> = sqlx::query_scalar(
                "select n.nspname from pg_class c join pg_namespace n on n.oid = c.relnamespace
                 where c.relname = 'migration' and n.nspname like '%db' and n.nspname <> $1 order by 1",
            )
            .bind(self.db.prefix().schema_name(MODULE))
            .fetch_all(&mut *conn)
            .await
            .map_err(unavailable)?;
            if !schemas.is_empty() {
                problems.push(Problem::OtherInstallations { schemas });
            }
        }

        for (module, sequence, checksum) in &applied {
            match self.step(module, *sequence) {
                Some(step) if &step.checksum() != checksum => problems.push(Problem::ChangedStep {
                    module: module.clone(),
                    sequence: *sequence,
                }),
                Some(_) => {}
                None => problems.push(Problem::UnknownStep {
                    module: module.clone(),
                    sequence: *sequence,
                }),
            }
        }

        let pending: Vec<PendingStep> = self
            .modules
            .iter()
            .flat_map(|m| m.steps.iter().map(move |step| (m.module, step)))
            .filter(|(module, step)| !applied.iter().any(|(m, s, _)| m == module && *s == step.sequence))
            .map(|(module, step)| PendingStep {
                module,
                sequence: step.sequence,
                name: step.name,
            })
            .collect();
        let recorded = registry::versions(&mut conn, &ledger).await?;
        let changes = self.component_changes(&recorded, &pending, &mut problems);
        Ok(UpgradePlan {
            installed,
            pending,
            changes,
            problems,
        })
    }

    /// The release recorded in the installation, if any.
    pub(super) async fn recorded_release(&self, conn: &mut sqlx::PgConnection) -> Result<Option<String>> {
        let ledger = self.ledger();
        let has_record: bool = sqlx::query_scalar("select to_regclass($1) is not null")
            .bind(format!("{ledger}.installation"))
            .fetch_one(&mut *conn)
            .await
            .map_err(unavailable)?;
        if !has_record {
            return Ok(None);
        }
        sqlx::query_scalar(audited(format!(
            "select upgraded_version from {ledger}.installation"
        )))
        .fetch_optional(conn)
        .await
        .map_err(unavailable)
    }

    /// Compares the components of the build with those installed.
    fn component_changes(
        &self,
        recorded: &BTreeMap<String, String>,
        pending: &[PendingStep],
        problems: &mut Vec<Problem>,
    ) -> Vec<ComponentChange> {
        let mut changes = Vec::new();
        for component in &self.components {
            let steps = component.schema.map_or(0, |schema| {
                i32::try_from(pending.iter().filter(|p| p.module == schema).count()).unwrap_or(i32::MAX)
            });
            match recorded.get(&component.name) {
                Some(installed) if newer(installed, &component.version) => {
                    problems.push(Problem::NewerComponent {
                        component: component.name.clone(),
                        recorded: installed.clone(),
                        code: component.version.clone(),
                    });
                }
                Some(installed) if *installed == component.version && steps == 0 => {}
                installed => changes.push(ComponentChange {
                    component: component.name.clone(),
                    from: installed.cloned(),
                    to: Some(component.version.clone()),
                    steps,
                }),
            }
        }
        if !self.components.is_empty() {
            for (name, version) in recorded {
                if !self.components.iter().any(|c| &c.name == name) {
                    problems.push(Problem::MissingComponent {
                        component: name.clone(),
                        version: version.clone(),
                    });
                }
            }
        }
        changes
    }

    /// Applies every pending step and registers the components, creating the installation when
    /// `new_installation` allows it. The run is kept in the history, failed or not.
    ///
    /// Upgrades started at the same time by several instances run one after the other.
    ///
    /// # Errors
    ///
    /// A `conflict` error when a problem stops the upgrade, and an `unavailable` error when a step or its
    /// action fails; the steps applied before it stay applied.
    pub async fn apply(&self, new_installation: bool) -> Result<UpgradeReport> {
        self.locked(self.apply_locked(new_installation)).await
    }

    /// Runs work while holding the upgrade lock of the installation.
    pub(super) async fn locked<T>(&self, work: impl Future<Output = Result<T>>) -> Result<T> {
        let key = lock_key(self.db.prefix().as_str());
        let mut lock = self.db.pool().acquire().await.map_err(unavailable)?;
        // A session advisory lock serialises upgrades started by several replicas: the ledger may not exist
        // yet, so no row or table lock is available, and the lock dies with the connection if the process does.
        sqlx::query("select pg_advisory_lock($1)")
            .bind(key)
            .execute(&mut *lock)
            .await
            .map_err(unavailable)?;
        let result = work.await;
        sqlx::query("select pg_advisory_unlock($1)")
            .bind(key)
            .execute(&mut *lock)
            .await
            .map_err(unavailable)?;
        result
    }

    /// Applies the upgrade while holding the upgrade lock.
    async fn apply_locked(&self, new_installation: bool) -> Result<UpgradeReport> {
        let started = self.db_now().await?;
        let plan = self.plan().await?;
        let blocking: Vec<&Problem> = plan
            .problems
            .iter()
            .filter(|p| !(new_installation && matches!(p, Problem::OtherInstallations { .. })))
            .collect();
        if let Some(first) = blocking.first() {
            let code = if matches!(first, Problem::OtherInstallations { .. }) {
                "db.other_installations"
            } else {
                "db.upgrade_refused"
            };
            let reasons = blocking
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(Error::conflict(code).with_param("problems", reasons));
        }
        if plan.pending.is_empty() && plan.changes.is_empty() && plan.installed {
            return Ok(UpgradeReport {
                installed: false,
                applied: Vec::new(),
                changes: Vec::new(),
            });
        }

        let ledger = self.ledger();
        sqlx::raw_sql(audited(format!(
            "create schema if not exists {ledger};
             create table if not exists {ledger}.migration (
                 module text not null,
                 sequence integer not null check (sequence > 0),
                 name text not null,
                 checksum bytea not null,
                 release text not null,
                 applied_at timestamptz not null default now(),
                 primary key (module, sequence)
             );
             alter table {ledger}.migration add column if not exists revert_sql text;"
        )))
        .execute(self.db.pool())
        .await
        .map_err(unavailable)?;

        let mut conn = self.db.pool().acquire().await.map_err(unavailable)?;
        let release_from = self.recorded_release(&mut conn).await?;
        drop(conn);
        let outcome = self.apply_steps(&plan.pending).await;
        let finished = self.db_now().await?;
        let run = Run {
            operation: "upgrade".to_owned(),
            release_from,
            release_to: self.release.to_owned(),
            started_at: started,
            finished_at: finished,
            outcome: if outcome.is_ok() { "succeeded" } else { "failed" }.to_owned(),
            error: outcome.as_ref().err().map(ToString::to_string),
            changes: plan.changes.clone(),
        };
        self.record_run(&run).await?;
        outcome?;

        sqlx::query(audited(format!(
            "insert into {ledger}.installation (prefix, installed_version, upgraded_version) values ($1, $2, $2)
             on conflict (singleton) do update set upgraded_version = excluded.upgraded_version, upgraded_at = now()
             where {ledger}.installation.upgraded_version <> excluded.upgraded_version"
        )))
        .bind(self.db.prefix().as_str())
        .bind(self.release)
        .execute(self.db.pool())
        .await
        .map_err(unavailable)?;
        Ok(UpgradeReport {
            installed: !plan.installed,
            applied: plan.pending,
            changes: plan.changes,
        })
    }

    /// Applies the pending steps in order, each with its action, in its own transaction.
    async fn apply_steps(&self, pending: &[PendingStep]) -> Result<()> {
        let ledger = self.ledger();
        for pending in pending {
            let Some(step) = self.step(pending.module, pending.sequence) else {
                continue;
            };
            let mut tx = self.db.begin().await?;
            if pending.sequence == 1 {
                sqlx::raw_sql(audited(format!(
                    "create schema if not exists {}",
                    self.db.prefix().schema(pending.module)
                )))
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
            }
            sqlx::raw_sql(audited(step.render(pending.module, self.db.prefix())))
                .execute(&mut *tx)
                .await
                .map_err(|error| {
                    Error::unavailable("db.step_failed")
                        .with_param("step", pending.to_string())
                        .with_source(error)
                })?;
            if let Some(action) = step.action {
                let context = ActionContext {
                    module: pending.module,
                    prefix: self.db.prefix().clone(),
                    release: self.release.to_owned(),
                };
                (action.run)(&mut tx, &context).await.map_err(|error| {
                    Error::unavailable("db.action_failed")
                        .with_param("step", pending.to_string())
                        .with_param("action", action.name)
                        .with_param("cause", error.to_string())
                })?;
            }
            sqlx::query(audited(format!(
                "insert into {ledger}.migration (module, sequence, name, checksum, release, revert_sql)
                 values ($1, $2, $3, $4, $5, $6)"
            )))
            .bind(pending.module)
            .bind(i32::try_from(pending.sequence).unwrap_or(i32::MAX))
            .bind(pending.name)
            .bind(step.checksum())
            .bind(self.release)
            .bind(step.render_revert(pending.module, self.db.prefix()))
            .execute(&mut *tx)
            .await
            .map_err(unavailable)?;
            tx.commit().await.map_err(unavailable)?;
            tracing::info!(step = %pending, release = self.release, "upgrade step applied");
        }
        Ok(())
    }

    /// Keeps a run in the history, once the registry exists.
    pub(super) async fn record_run(&self, run: &Run) -> Result<()> {
        let ledger = self.ledger();
        let mut tx = self.db.begin().await?;
        if registry::exists(&mut tx, &ledger).await? {
            registry::record(&mut tx, &ledger, run, &self.components).await?;
        }
        tx.commit().await.map_err(unavailable)
    }

    /// The current instant of the database.
    pub(super) async fn db_now(&self) -> Result<Timestamp> {
        sqlx::query_scalar::<_, time::OffsetDateTime>("select clock_timestamp()")
            .fetch_one(self.db.pool())
            .await
            .map(Timestamp::from_offset_date_time)
            .map_err(unavailable)
    }

    /// Finds a step of a module in the code.
    pub(super) fn step(&self, module: &str, sequence: u32) -> Option<&'static Step> {
        self.modules
            .iter()
            .find(|m| m.module == module)?
            .steps
            .iter()
            .find(|s| s.sequence == sequence)
    }

    /// Checks that each module declares its steps from 1, in order and without gaps, and appears once.
    fn check_declarations(&self) -> Result<()> {
        for (index, module) in self.modules.iter().enumerate() {
            let ordered = module
                .steps
                .iter()
                .zip(1..)
                .all(|(step, expected)| step.sequence == expected);
            let unique = !self.modules[..index].iter().any(|m| m.module == module.module);
            if !ordered || !unique {
                return Err(Error::internal("db.steps_invalid").with_param("module", module.module));
            }
        }
        Ok(())
    }
}

/// Marks SQL built by this crate as audited: the only text interpolated into it is a schema name made by
/// [`crate::Prefix::schema`] from a validated prefix and a module name of the code, always quoted; every
/// value is bound as a parameter.
pub(super) fn audited(sql: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(sql)
}

/// Wraps a database failure as `unavailable`.
pub(super) fn unavailable(error: sqlx::Error) -> Error {
    Error::unavailable("db.query").with_source(error)
}

/// The advisory lock key of an installation's upgrades.
fn lock_key(prefix: &str) -> i64 {
    let digest = Sha256::digest(format!("kampus-upgrade:{prefix}").as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

/// Whether the version `recorded` is newer than `current`, by semantic versioning; versions that cannot be
/// read are never newer.
pub(super) fn newer(recorded: &str, current: &str) -> bool {
    matches!((Version::parse(recorded), Version::parse(current)), (Ok(r), Ok(c)) if r > c)
}

#[cfg(test)]
mod tests {
    use super::{lock_key, newer};

    #[test]
    fn versions_compare_semantically() {
        assert!(newer("0.2.0", "0.1.9"));
        assert!(newer("1.0.0", "0.10.0"));
        assert!(newer("0.2.0", "0.2.0-rc.1"));
        assert!(!newer("0.1.0", "0.1.0"));
        assert!(!newer("0.2.0-rc.1", "0.2.0"));
        assert!(!newer("garbage", "0.1.0"));
    }

    #[test]
    fn each_installation_has_its_own_lock() {
        assert_eq!(lock_key("kmp_"), lock_key("kmp_"));
        assert_ne!(lock_key("kmp_"), lock_key("t1_"));
    }
}
