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
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/** The messages of the shell in every language it speaks; the UI modules bring their own. */

/** The messages of the shell, by language and key. */
export const MESSAGES: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  en: {
    "nav.primary": "Primary",
    "nav.home": "Home",
    "nav.account": "Your account",
    "scheme.label": "Appearance",
    "scheme.light": "Light",
    "scheme.dark": "Dark",
    "scheme.system": "Like the device",
    "toast.close": "Dismiss",
    "notifications.allread": "All your notifications are read.",
    "courses.lead": "The courses of your campus, by category.",
    "courses.open": "Open {name}",
    "notfound.title": "This page does not exist",
    "notfound.lead": "The address may be mistyped, or the page may have moved.",
    "error.retry": "Try again",
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
    "nav.primary": "Principal",
    "nav.home": "Inicio",
    "nav.account": "Tu cuenta",
    "scheme.label": "Apariencia",
    "scheme.light": "Claro",
    "scheme.dark": "Oscuro",
    "scheme.system": "Como el dispositivo",
    "toast.close": "Cerrar",
    "notifications.allread": "Todas tus notificaciones están leídas.",
    "courses.lead": "Los cursos de tu campus, por categoría.",
    "courses.open": "Abrir {name}",
    "notfound.title": "Esta página no existe",
    "notfound.lead": "Puede que la dirección esté mal escrita o que la página se haya movido.",
    "error.retry": "Reintentar",
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
