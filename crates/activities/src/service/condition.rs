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
// @package   kampus-activities
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Access conditions: a tree of `all`, `any` and `not` over leaves of kinds that modules provide, on a course
//! module or on a section (and so on everything inside it). Each kind keeps its leaves in its own table; a
//! leaf of an unknown kind, or one that lost what it depends on, never holds, so a course module is never
//! opened by mistake.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use kampus_access::{Actor, PrincipalId, ScopeId};
use kampus_common::{Error, FieldError, Result};
use kampus_courses::{CourseId, SectionId};
use kampus_module::{
    ConditionCheck, ConditionKindDeclaration, ConditionTarget, ConditionWrite, ServiceContext,
};
use serde_json::json;
use sqlx::{PgConnection, Row};

use crate::permissions::{CONDITIONS_IGNORE, CONDITIONS_MANAGE};
use crate::{
    Activities, Availability, ConditionNode, Conditions, Module, ModuleId, Operator, Restricted, events, sql,
};

/// The most nodes a tree may have.
const MAX_NODES: usize = 100;

/// The deepest a tree may go.
const MAX_DEPTH: usize = 10;

/// A future of this module that borrows the service.
type Boxed<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// A stored node, before the tree is built.
#[derive(Debug, Clone)]
struct Stored {
    /// The node.
    id: uuid::Uuid,
    /// Its parent; none for the root.
    parent: Option<uuid::Uuid>,
    /// How it combines its children.
    operator: Operator,
    /// The kind of a leaf.
    kind: Option<String>,
    /// For the root: whether people who miss it still see what it is on.
    show_when_unmet: bool,
}

/// The person behind an actor; none for a visitor.
const fn principal_of(actor: &Actor) -> Option<PrincipalId> {
    match actor {
        Actor::Principal(p) | Actor::ViewAs(p, _) => Some(*p),
        Actor::Visitor(_) => None,
    }
}

/// A problem with the shape of a tree.
fn shape(rule: &str) -> Error {
    Error::validation(
        "activities.conditions_invalid",
        vec![FieldError::new("tree", rule)],
    )
}

/// Checks the shape of a tree: leaves name a kind and have no children, `not` has one child, `all` and `any`
/// at least one, and the tree stays within its limits.
fn check_shape(node: &ConditionNode, depth: usize, count: &mut usize) -> Result<()> {
    *count += 1;
    if *count > MAX_NODES || depth > MAX_DEPTH {
        return Err(shape("too_large"));
    }
    match node.operator {
        Operator::Leaf if node.kind.is_none() || !node.children.is_empty() => Err(shape("leaf")),
        Operator::Leaf => Ok(()),
        Operator::Not if node.children.len() != 1 => Err(shape("not")),
        Operator::All | Operator::Any if node.children.is_empty() => Err(shape("empty")),
        _ => {
            for child in &node.children {
                check_shape(child, depth + 1, count)?;
            }
            Ok(())
        }
    }
}

impl Activities {
    /// The declaration of a kind of condition of the build.
    fn condition_kind(&self, code: &str) -> Option<&'static ConditionKindDeclaration> {
        self.modules
            .iter()
            .flat_map(|m| m.condition_kinds.iter())
            .find(|k| k.code == code)
    }

    /// The scope and the course of what conditions are on.
    async fn restricted(&self, target: Restricted) -> Result<(ScopeId, CourseId)> {
        match target {
            Restricted::Module(id) => {
                let module = self.stored(id).await?;
                let course = self.course_of_module(id).await?;
                Ok((module.scope, course))
            }
            Restricted::Section(id) => {
                let section = self.courses.section(id).await?;
                Ok((section.scope, section.course))
            }
        }
    }

    /// The column that ties a root to what it is on, with the id.
    fn root_of(target: Restricted) -> (&'static str, uuid::Uuid) {
        match target {
            Restricted::Module(id) => ("course_module_id", id.as_uuid()),
            Restricted::Section(id) => ("section_id", id.as_uuid()),
        }
    }

    /// Replaces the access conditions of a course module or a section, or removes them with `None`; needs
    /// `activities.conditions.manage` there. Each leaf is kept by its kind, in the same transaction.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` (`activities.conditions_invalid`) for a tree
    /// of the wrong shape, (`activities.condition_kind_unknown`) for a kind the build does not have, and the
    /// errors of each kind for its parameters.
    pub async fn set_conditions(
        &self,
        actor: &Actor,
        target: Restricted,
        conditions: Option<Conditions>,
        services: &ServiceContext,
    ) -> Result<Option<Conditions>> {
        let (scope, course) = self.restricted(target).await?;
        self.access.require(actor, CONDITIONS_MANAGE, scope).await?;
        if let Some(conditions) = &conditions {
            check_shape(&conditions.tree, 1, &mut 0)?;
        }
        let (column, id) = Self::root_of(target);
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where {column} = $1",
            self.table("condition")
        )))
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        if let Some(conditions) = &conditions {
            let root = uuid::Uuid::now_v7();
            sqlx::query(sql(format!(
                "insert into {} (id, {column}, operator, kind, show_when_unmet) values ($1, $2, $3, $4, $5)",
                self.table("condition")
            )))
            .bind(root)
            .bind(id)
            .bind(conditions.tree.operator.as_str())
            .bind(conditions.tree.kind.as_deref())
            .bind(conditions.show_when_unmet)
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?;
            self.write_node(&mut tx, root, &conditions.tree, course, services)
                .await?;
        }
        let (module, section) = match target {
            Restricted::Module(m) => (Some(m.as_uuid()), None),
            Restricted::Section(s) => (None, Some(s.as_uuid())),
        };
        let mut event = kampus_events::Publish::new(
            events::CONDITIONS_CHANGED,
            Some(id),
            json!({ "module": module, "section": section }),
        );
        event.scope = Some(scope.as_uuid());
        event.actor = principal_of(actor).map(|p| p.as_uuid());
        event.summary = Some(format!("{} {id}", events::CONDITIONS_CHANGED));
        kampus_events::Events::new(self.db.clone())
            .publish_in(&mut tx, event)
            .await?;
        tx.commit().await.map_err(kampus_db::classify)?;
        self.stored_conditions(target, services).await
    }

    /// Writes the children of a node, or the leaf it is, depth first.
    fn write_node<'a>(
        &'a self,
        conn: &'a mut PgConnection,
        id: uuid::Uuid,
        node: &'a ConditionNode,
        course: CourseId,
        services: &'a ServiceContext,
    ) -> Boxed<'a, ()> {
        Box::pin(async move {
            if node.operator == Operator::Leaf {
                let code = node.kind.as_deref().unwrap_or_default();
                let kind = self.condition_kind(code).ok_or_else(|| {
                    Error::validation(
                        "activities.condition_kind_unknown",
                        vec![FieldError::new("kind", "one_of")],
                    )
                    .with_param("kind", code.to_owned())
                })?;
                let params = node.params.clone().unwrap_or_else(|| json!({}));
                return (kind.store)(ConditionWrite {
                    condition: id,
                    course: course.as_uuid(),
                    params: &params,
                    conn: &mut *conn,
                    services,
                })
                .await;
            }
            for (position, child) in node.children.iter().enumerate() {
                let child_id = uuid::Uuid::now_v7();
                sqlx::query(sql(format!(
                    "insert into {} (id, parent_id, position, operator, kind) values ($1, $2, $3, $4, $5)",
                    self.table("condition")
                )))
                .bind(child_id)
                .bind(id)
                .bind(i32::try_from(position).unwrap_or(i32::MAX))
                .bind(child.operator.as_str())
                .bind(child.kind.as_deref())
                .execute(&mut *conn)
                .await
                .map_err(kampus_db::classify)?;
                self.write_node(&mut *conn, child_id, child, course, services)
                    .await?;
            }
            Ok(())
        })
    }

    /// The access conditions of a course module or a section, for whoever may manage them there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without `activities.conditions.manage`.
    pub async fn conditions(
        &self,
        actor: &Actor,
        target: Restricted,
        services: &ServiceContext,
    ) -> Result<Option<Conditions>> {
        let (scope, _) = self.restricted(target).await?;
        self.access.require(actor, CONDITIONS_MANAGE, scope).await?;
        self.stored_conditions(target, services).await
    }

    /// The nodes of the tree on something, root first.
    async fn nodes(&self, target: Restricted) -> Result<Vec<Stored>> {
        let (column, id) = Self::root_of(target);
        let rows = sqlx::query(sql(format!(
            "with recursive t as (
                 select id, parent_id, position, operator, kind, show_when_unmet, 0 as depth
                 from {c} where {column} = $1
                 union all
                 select c.id, c.parent_id, c.position, c.operator, c.kind, c.show_when_unmet, t.depth + 1
                 from {c} c join t on c.parent_id = t.id where t.depth < {MAX_DEPTH}
             ) select * from t order by depth, position",
            c = self.table("condition")
        )))
        .bind(id)
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        Ok(rows
            .iter()
            .map(|r| Stored {
                id: r.get("id"),
                parent: r.get("parent_id"),
                operator: Operator::read(r.get("operator")),
                kind: r.get("kind"),
                show_when_unmet: r.get("show_when_unmet"),
            })
            .collect())
    }

    /// The tree on something as it would be given again, with the leaves that broke marked.
    async fn stored_conditions(
        &self,
        target: Restricted,
        services: &ServiceContext,
    ) -> Result<Option<Conditions>> {
        let nodes = self.nodes(target).await?;
        let Some(root) = nodes.first().cloned() else {
            return Ok(None);
        };
        let mut children: BTreeMap<uuid::Uuid, Vec<Stored>> = BTreeMap::new();
        for node in nodes.iter().skip(1) {
            if let Some(parent) = node.parent {
                children.entry(parent).or_default().push(node.clone());
            }
        }
        let tree = self.build(&root, &children, services).await?;
        Ok(Some(Conditions {
            show_when_unmet: root.show_when_unmet,
            tree,
        }))
    }

    /// Builds a node and its children, asking each kind for the parameters of its leaves.
    fn build<'a>(
        &'a self,
        node: &'a Stored,
        children: &'a BTreeMap<uuid::Uuid, Vec<Stored>>,
        services: &'a ServiceContext,
    ) -> Boxed<'a, ConditionNode> {
        Box::pin(async move {
            if node.operator == Operator::Leaf {
                let code = node.kind.clone().unwrap_or_default();
                let params = match self.condition_kind(&code) {
                    Some(kind) => {
                        (kind.params)(ConditionCheck {
                            condition: node.id,
                            account: uuid::Uuid::nil(),
                            services,
                        })
                        .await?
                    }
                    None => None,
                };
                return Ok(ConditionNode {
                    operator: Operator::Leaf,
                    children: Vec::new(),
                    kind: Some(code),
                    broken: params.is_none(),
                    params,
                });
            }
            let mut built = Vec::new();
            for child in children.get(&node.id).map_or(&[][..], Vec::as_slice) {
                built.push(self.build(child, children, services).await?);
            }
            Ok(ConditionNode {
                operator: node.operator,
                children: built,
                kind: None,
                params: None,
                broken: false,
            })
        })
    }

    /// Whether a person meets a node.
    fn holds<'a>(
        &'a self,
        node: &'a Stored,
        children: &'a BTreeMap<uuid::Uuid, Vec<Stored>>,
        account: uuid::Uuid,
        services: &'a ServiceContext,
    ) -> Boxed<'a, bool> {
        Box::pin(async move {
            let kids = children.get(&node.id).map_or(&[][..], Vec::as_slice);
            match node.operator {
                Operator::Leaf => match node.kind.as_deref().and_then(|k| self.condition_kind(k)) {
                    Some(kind) => {
                        (kind.met)(ConditionCheck {
                            condition: node.id,
                            account,
                            services,
                        })
                        .await
                    }
                    None => Ok(false),
                },
                Operator::All => {
                    for child in kids {
                        if !self.holds(child, children, account, services).await? {
                            return Ok(false);
                        }
                    }
                    Ok(!kids.is_empty())
                }
                Operator::Any => {
                    for child in kids {
                        if self.holds(child, children, account, services).await? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                Operator::Not => match kids {
                    [only] => Ok(!self.holds(only, children, account, services).await?),
                    _ => Ok(false),
                },
            }
        })
    }

    /// Whether a person meets the tree on something, and whether that tree shows it when they do not; true
    /// when there is no tree.
    async fn meets(
        &self,
        target: Restricted,
        account: uuid::Uuid,
        services: &ServiceContext,
    ) -> Result<(bool, bool)> {
        let nodes = self.nodes(target).await?;
        let Some(root) = nodes.first() else {
            return Ok((true, true));
        };
        let mut children: BTreeMap<uuid::Uuid, Vec<Stored>> = BTreeMap::new();
        for node in nodes.iter().skip(1) {
            if let Some(parent) = node.parent {
                children.entry(parent).or_default().push(node.clone());
            }
        }
        let met = self.holds(root, &children, account, services).await?;
        Ok((met, root.show_when_unmet))
    }

    /// Whether the actor may open a course module: the conditions on it and on its section and the sections
    /// above all hold, or they hold `activities.conditions.ignore` there. A visitor meets no condition.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn availability(
        &self,
        actor: &Actor,
        module: &Module,
        services: &ServiceContext,
    ) -> Result<Availability> {
        if self
            .access
            .check(actor, CONDITIONS_IGNORE, module.scope)
            .await?
            .allowed
        {
            return Ok(Availability {
                available: true,
                shown: true,
            });
        }
        let account = principal_of(actor).map_or_else(uuid::Uuid::nil, |p| p.as_uuid());
        let mut targets = vec![Restricted::Module(module.id)];
        let mut section: Option<SectionId> = Some(module.section);
        while let Some(id) = section {
            targets.push(Restricted::Section(id));
            section = self.courses.section(id).await?.parent;
        }
        let mut availability = Availability {
            available: true,
            shown: true,
        };
        for target in targets {
            let (met, show) = self.meets(target, account, services).await?;
            if !met {
                availability.available = false;
                availability.shown &= show;
            }
        }
        Ok(availability)
    }

    /// The course modules of a section the actor sees, each with whether they may open it; those whose
    /// conditions hide them are left out, and those they may not open come without their settings.
    ///
    /// # Errors
    ///
    /// As [`Activities::modules`].
    pub async fn available_modules(
        &self,
        actor: &Actor,
        section: SectionId,
        services: &ServiceContext,
    ) -> Result<Vec<Module>> {
        let mut listed = Vec::new();
        for module in self.modules(actor, section).await? {
            if let Some(module) = self.as_available(actor, module, services).await? {
                listed.push(module);
            }
        }
        Ok(listed)
    }

    /// A course module the actor sees with whether they may open it: without its settings when they may not,
    /// and not found when its conditions hide it.
    ///
    /// # Errors
    ///
    /// As [`Activities::module`].
    pub async fn available_module(
        &self,
        actor: &Actor,
        id: ModuleId,
        services: &ServiceContext,
    ) -> Result<Module> {
        let module = self.module(actor, id).await?;
        self.as_available(actor, module, services)
            .await?
            .ok_or_else(|| Error::not_found("activities.module_not_found"))
    }

    /// A course module as the actor may have it.
    async fn as_available(
        &self,
        actor: &Actor,
        mut module: Module,
        services: &ServiceContext,
    ) -> Result<Option<Module>> {
        let availability = self.availability(actor, &module, services).await?;
        if !availability.shown {
            return Ok(None);
        }
        if !availability.available {
            module.available = false;
            module.settings.clear();
        }
        Ok(Some(module))
    }

    /// The leaves of every kind that depend on a course module.
    ///
    /// # Errors
    ///
    /// The errors of the kinds.
    pub async fn conditions_depending_on(
        &self,
        module: ModuleId,
        services: &ServiceContext,
    ) -> Result<Vec<uuid::Uuid>> {
        let mut found = Vec::new();
        for kind in self.modules.iter().flat_map(|m| m.condition_kinds.iter()) {
            found.extend(
                (kind.depending_on)(ConditionTarget {
                    module: module.as_uuid(),
                    services,
                })
                .await?,
            );
        }
        Ok(found)
    }

    /// Moves a course module to the recycle bin, unless access conditions depend on it: then it is refused,
    /// or, with `remove_conditions`, those leaves are removed first, with the nodes they leave empty.
    ///
    /// # Errors
    ///
    /// `conflict` (`activities.module_required_by_conditions`) when conditions depend on it and they are
    /// not to be removed, and the errors of [`Activities::delete_module`].
    pub async fn delete_module_checked(
        &self,
        actor: &Actor,
        id: ModuleId,
        remove_conditions: bool,
        services: &ServiceContext,
    ) -> Result<()> {
        let module = self.stored(id).await?;
        self.access
            .require(actor, crate::permissions::MODULE_MANAGE, module.scope)
            .await?;
        let depending = self.conditions_depending_on(id, services).await?;
        if !depending.is_empty() {
            if !remove_conditions {
                return Err(Error::conflict("activities.module_required_by_conditions")
                    .with_param("conditions", depending.len().to_string()));
            }
            self.remove_leaves(&depending).await?;
        }
        self.delete_module(actor, id).await
    }

    /// Removes leaves, and then the nodes that are left with no children, up to the roots.
    async fn remove_leaves(&self, leaves: &[uuid::Uuid]) -> Result<()> {
        let mut tx = self.db.begin().await?;
        sqlx::query(sql(format!(
            "delete from {} where id = any($1)",
            self.table("condition")
        )))
        .bind(leaves)
        .execute(&mut *tx)
        .await
        .map_err(kampus_db::classify)?;
        loop {
            let removed = sqlx::query(sql(format!(
                "delete from {c} n where n.operator <> 'leaf'
                 and not exists (select 1 from {c} k where k.parent_id = n.id)",
                c = self.table("condition")
            )))
            .execute(&mut *tx)
            .await
            .map_err(kampus_db::classify)?
            .rows_affected();
            if removed == 0 {
                break;
            }
        }
        tx.commit().await.map_err(kampus_db::classify)
    }
}
