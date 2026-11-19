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

//! Going back: a downgrade undoes the steps newer than the code with the SQL each step left in the ledger
//! when it was applied, and an uninstall removes a component that is no longer in the build. Both are
//! explicit operations, never part of a start, and both are kept in the history.

use kampus_common::{Error, Result};
use sqlx::Row;

use super::registry::{self, ComponentChange, Run};
use super::runner::{DowngradeReport, Problem, Upgrader, audited, unavailable};

impl Upgrader<'_> {
    /// Takes the installation back to the release and component versions of this code: every step the
    /// code does not know is undone, newest first, with the SQL it left when it was applied.
    ///
    /// # Errors
    ///
    /// A `conflict` error when a step changed, a component is missing from the build, or a newer step left
    /// no way to undo it (`db.downgrade_impossible`, naming the steps); an `unavailable` error when undoing
    /// fails.
    pub async fn downgrade(&self) -> Result<DowngradeReport> {
        self.locked(self.downgrade_locked()).await
    }

    /// Downgrades while holding the upgrade lock.
    async fn downgrade_locked(&self) -> Result<DowngradeReport> {
        let started = self.db_now().await?;
        let plan = self.plan().await?;
        let Targets {
            unknown,
            newer,
            newer_release,
        } = targets(&plan)?;
        if unknown.is_empty() && newer.is_empty() && !newer_release {
            return Ok(DowngradeReport {
                reverted: Vec::new(),
                changes: Vec::new(),
            });
        }
        let reverts = self.stored_reverts(unknown).await?;
        let ledger = self.ledger();

        let mut conn = self.db.pool().acquire().await.map_err(unavailable)?;
        let release_from = self.recorded_release(&mut conn).await?;
        drop(conn);
        let mut reverted = Vec::with_capacity(reverts.len());
        let mut outcome = Ok(());
        for (module, sequence, sql) in &reverts {
            let step = format!("{module}/{sequence}");
            if let Err(error) = self.revert_step(module, *sequence, sql).await {
                outcome = Err(error.with_param("step", step));
                break;
            }
            tracing::info!(step = %step, "upgrade step undone");
            reverted.push(step);
        }
        let changes: Vec<ComponentChange> = newer
            .into_iter()
            .map(|mut change| {
                let schema = self
                    .components
                    .iter()
                    .find(|c| c.name == change.component)
                    .and_then(|c| c.schema);
                change.steps = i32::try_from(reverts.iter().filter(|r| Some(r.0.as_str()) == schema).count())
                    .unwrap_or(i32::MAX);
                change
            })
            .collect();
        let run = Run {
            operation: "downgrade".to_owned(),
            release_from,
            release_to: self.release.to_owned(),
            started_at: started,
            finished_at: self.db_now().await?,
            outcome: if outcome.is_ok() { "succeeded" } else { "failed" }.to_owned(),
            error: outcome.as_ref().err().map(ToString::to_string),
            changes: changes.clone(),
        };
        self.record_run(&run).await?;
        outcome?;
        sqlx::query(audited(format!(
            "update {ledger}.installation set upgraded_version = $1, upgraded_at = now()"
        )))
        .bind(self.release)
        .execute(self.db.pool())
        .await
        .map_err(unavailable)?;
        Ok(DowngradeReport { reverted, changes })
    }

    /// The SQL each unknown step left to undo it, newest first.
    async fn stored_reverts(&self, mut unknown: Vec<(String, u32)>) -> Result<Vec<(String, u32, String)>> {
        let ledger = self.ledger();
        let order = |module: &str| {
            self.modules
                .iter()
                .position(|m| m.module == module)
                .unwrap_or(usize::MAX)
        };
        unknown.sort_by(|a, b| order(&b.0).cmp(&order(&a.0)).then(b.1.cmp(&a.1)));
        let mut reverts = Vec::with_capacity(unknown.len());
        let mut irreversible = Vec::new();
        for (module, sequence) in &unknown {
            let sql: Option<String> = sqlx::query(audited(format!(
                "select revert_sql from {ledger}.migration where module = $1 and sequence = $2"
            )))
            .bind(module)
            .bind(i32::try_from(*sequence).unwrap_or(i32::MAX))
            .fetch_one(self.db.pool())
            .await
            .map_err(unavailable)?
            .get("revert_sql");
            match sql {
                Some(sql) => reverts.push((module.clone(), *sequence, sql)),
                None => irreversible.push(format!("{module}/{sequence}")),
            }
        }
        if !irreversible.is_empty() {
            return Err(
                Error::conflict("db.downgrade_impossible").with_param("steps", irreversible.join(", "))
            );
        }
        Ok(reverts)
    }

    /// Undoes one step and forgets it, in one transaction.
    async fn revert_step(&self, module: &str, sequence: u32, sql: &str) -> Result<()> {
        let ledger = self.ledger();
        let mut tx = self.db.begin().await?;
        sqlx::raw_sql(audited(sql.to_owned()))
            .execute(&mut *tx)
            .await
            .map_err(|error| Error::unavailable("db.revert_failed").with_source(error))?;
        sqlx::query(audited(format!(
            "delete from {ledger}.migration where module = $1 and sequence = $2"
        )))
        .bind(module)
        .bind(i32::try_from(sequence).unwrap_or(i32::MAX))
        .execute(&mut *tx)
        .await
        .map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)
    }

    /// Removes a component that is installed but no longer in the build: its schema, its steps in the
    /// ledger and its registration. The caller checks that nothing installed needs it.
    ///
    /// # Errors
    ///
    /// `not_found` when it is not installed, `conflict` when it is still in the build, `unavailable` when
    /// the database fails.
    pub async fn uninstall(&self, component: &str) -> Result<ComponentChange> {
        self.locked(self.uninstall_locked(component)).await
    }

    /// Uninstalls while holding the upgrade lock.
    async fn uninstall_locked(&self, component: &str) -> Result<ComponentChange> {
        let started = self.db_now().await?;
        if self.components.iter().any(|c| c.name == component) {
            return Err(Error::conflict("db.component_in_build").with_param("component", component));
        }
        let ledger = self.ledger();
        let mut conn = self.db.pool().acquire().await.map_err(unavailable)?;
        let versions = registry::versions(&mut conn, &ledger).await?;
        let release = self.recorded_release(&mut conn).await?;
        drop(conn);
        let Some(version) = versions.get(component).cloned() else {
            return Err(Error::not_found("db.component_not_installed").with_param("component", component));
        };
        let mut tx = self.db.begin().await?;
        let steps = sqlx::query(audited(format!(
            "delete from {ledger}.migration where module = $1"
        )))
        .bind(component)
        .execute(&mut *tx)
        .await
        .map_err(unavailable)?
        .rows_affected();
        sqlx::raw_sql(audited(format!(
            "drop schema if exists {} cascade",
            self.db.prefix().schema(component)
        )))
        .execute(&mut *tx)
        .await
        .map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)?;
        let change = ComponentChange {
            component: component.to_owned(),
            from: Some(version),
            to: None,
            steps: i32::try_from(steps).unwrap_or(i32::MAX),
        };
        let run = Run {
            operation: "uninstall".to_owned(),
            release_from: release.clone(),
            release_to: release.unwrap_or_else(|| self.release.to_owned()),
            started_at: started,
            finished_at: self.db_now().await?,
            outcome: "succeeded".to_owned(),
            error: None,
            changes: vec![change.clone()],
        };
        self.record_run(&run).await?;
        tracing::info!(component, "component uninstalled");
        Ok(change)
    }
}

/// What a downgrade must undo.
struct Targets {
    /// The steps the code does not know.
    unknown: Vec<(String, u32)>,
    /// The components installed in a newer version, going back to the code's.
    newer: Vec<ComponentChange>,
    /// Whether the installation was upgraded by a newer release.
    newer_release: bool,
}

/// What a downgrade must undo, or why it may not run.
fn targets(plan: &super::runner::UpgradePlan) -> Result<Targets> {
    let refused: Vec<String> = plan
        .problems
        .iter()
        .filter(|p| {
            matches!(
                p,
                Problem::ChangedStep { .. }
                    | Problem::MissingComponent { .. }
                    | Problem::OtherInstallations { .. }
            )
        })
        .map(ToString::to_string)
        .collect();
    if !refused.is_empty() {
        return Err(Error::conflict("db.downgrade_refused").with_param("problems", refused.join("; ")));
    }
    let unknown: Vec<(String, u32)> = plan
        .problems
        .iter()
        .filter_map(|p| match p {
            Problem::UnknownStep { module, sequence } => Some((module.clone(), *sequence)),
            _ => None,
        })
        .collect();
    let newer: Vec<ComponentChange> = plan
        .problems
        .iter()
        .filter_map(|p| match p {
            Problem::NewerComponent {
                component,
                recorded,
                code,
            } => Some(ComponentChange {
                component: component.clone(),
                from: Some(recorded.clone()),
                to: Some(code.clone()),
                steps: 0,
            }),
            _ => None,
        })
        .collect();
    let newer_release = plan
        .problems
        .iter()
        .any(|p| matches!(p, Problem::NewerRelease { .. }));
    Ok(Targets {
        unknown,
        newer,
        newer_release,
    })
}
