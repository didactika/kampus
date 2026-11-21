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
// @package   web/src
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

const bundles: Record<string, Record<string, string>> = {
  en: {
    skip: "Skip to content",
    "nav.courses": "Courses",
    "nav.notifications": "Notifications",
    "nav.signout": "Sign out",
    "signin.title": "Sign in",
    "signin.name": "Username or email",
    "signin.password": "Password",
    "signin.submit": "Sign in",
    "signin.failed": "The name or the password is not right.",
    "courses.title": "Courses",
    "courses.empty": "There are no courses to show.",
    "course.sections": "Sections",
    "course.untitled": "Section {n}",
    "course.grades": "My grades",
    "course.calendar": "Calendar",
    "course.progress": "My progress",
    "course.nothing": "Nothing here yet.",
    "grades.record": "Record",
    "grades.grade": "Grade",
    "calendar.title": "Calendar of the course",
    "calendar.next": "Next 30 days",
    "calendar.opens": "Opens",
    "calendar.due": "Due",
    "calendar.closes": "Closes",
    "calendar.cutoff": "Last day for late work",
    "calendar.entry": "Entry",
    "notifications.title": "Notifications",
    "notifications.empty": "You have no notifications.",
    "notifications.unread": "{n} unread",
    "notifications.markall": "Mark all as read",
    "notifications.read": "Read",
    "state.not_started": "Not started",
    "state.in_progress": "In progress",
    "state.done": "Done",
    "state.done_passed": "Passed",
    "state.done_failed": "Not passed",
    "error.generic": "Something went wrong. Try again.",
    "error.denied": "You are not allowed to see this.",
    "loading": "Loading…",
    "back": "Back",
  },
  es: {
    skip: "Saltar al contenido",
    "nav.courses": "Cursos",
    "nav.notifications": "Notificaciones",
    "nav.signout": "Cerrar sesión",
    "signin.title": "Iniciar sesión",
    "signin.name": "Usuario o correo",
    "signin.password": "Contraseña",
    "signin.submit": "Entrar",
    "signin.failed": "El nombre o la contraseña no son correctos.",
    "courses.title": "Cursos",
    "courses.empty": "No hay cursos que mostrar.",
    "course.sections": "Secciones",
    "course.untitled": "Sección {n}",
    "course.grades": "Mis notas",
    "course.calendar": "Calendario",
    "course.progress": "Mi progreso",
    "course.nothing": "Todavía no hay nada.",
    "grades.record": "Registro",
    "grades.grade": "Nota",
    "calendar.title": "Calendario del curso",
    "calendar.next": "Próximos 30 días",
    "calendar.opens": "Se abre",
    "calendar.due": "Entrega",
    "calendar.closes": "Se cierra",
    "calendar.cutoff": "Último día para entregas tardías",
    "calendar.entry": "Entrada",
    "notifications.title": "Notificaciones",
    "notifications.empty": "No tienes notificaciones.",
    "notifications.unread": "{n} sin leer",
    "notifications.markall": "Marcar todas como leídas",
    "notifications.read": "Leída",
    "state.not_started": "Sin empezar",
    "state.in_progress": "En curso",
    "state.done": "Hecho",
    "state.done_passed": "Aprobado",
    "state.done_failed": "No aprobado",
    "error.generic": "Algo salió mal. Inténtalo de nuevo.",
    "error.denied": "No tienes permiso para ver esto.",
    loading: "Cargando…",
    back: "Volver",
  },
};

let language = "en";

/** The languages the client has messages for. */
export const languages = Object.keys(bundles);

/** Chooses the language of the client: the one asked for if there is a bundle, English otherwise. */
export function setLanguage(code: string): string {
  const short = code.toLowerCase().slice(0, 2);
  language = bundles[short] ? short : "en";
  document.documentElement.lang = language;
  document.documentElement.dir = "ltr";
  return language;
}

/** The current language. */
export function currentLanguage(): string {
  return language;
}

/** A message in the current language, with `{name}` placeholders filled in. */
export function t(key: string, values: Record<string, string | number> = {}): string {
  const text = bundles[language]?.[key] ?? bundles["en"]?.[key] ?? key;
  return text.replace(/\{(\w+)\}/g, (_, name: string) => String(values[name] ?? ""));
}

/** A date and time in the viewer's locale and time zone. */
export function formatDate(iso: string): string {
  return new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short" }).format(new Date(iso));
}

/** A number in the viewer's locale. */
export function formatNumber(value: string | number): string {
  return new Intl.NumberFormat(language, { maximumFractionDigits: 5 }).format(Number(value));
}
