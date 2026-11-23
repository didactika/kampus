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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/**
 * The extensions of the rich editor. The core registers the formats of user content; a module or a plugin registers
 * more (a button, a kind of embedded content, a mention) with `registerEditorExtension`, and every rich editor of the
 * client then offers them. What an extension writes must stay within the format the server keeps, or it is removed
 * when it is saved.
 */

import type { AnyExtension, Editor } from "@tiptap/react";

/** Asks the person for a text, with a label from the strings; nothing when they cancel. */
export type Ask = (label: string, initial?: string) => Promise<string | null>;

/** A button of the toolbar of the editor. */
export interface EditorButton {
  /** Its name, unique among the buttons. */
  readonly name: string;
  /** Its icon, by the name the design system gives it. */
  readonly icon: string;
  /** The key of its label in the strings of the client or of its plugin. */
  readonly label: string;
  /** Whether it is on where the cursor is. */
  readonly active: (editor: Editor) => boolean;
  /** What it does; it may ask the person for a text, such as an address, in the design of the editor. */
  readonly run: (editor: Editor, ask: Ask) => void | Promise<void>;
}

/** What an extension brings: Tiptap extensions and buttons for the toolbar. */
export interface EditorExtension {
  /** Its name, unique in the build: `<module or plugin>.<extension>`. */
  readonly name: string;
  readonly extensions?: readonly AnyExtension[];
  readonly buttons?: readonly EditorButton[];
}

const registered = new Map<string, EditorExtension>();

/** Registers an extension of the rich editor; registering one name again replaces it. */
export function registerEditorExtension(extension: EditorExtension): void {
  registered.set(extension.name, extension);
}

/** The extensions registered, in the order they were. */
export function editorExtensions(): readonly EditorExtension[] {
  return [...registered.values()];
}

/** The buttons the core gives every rich editor: the formats of user content. */
export const CORE_BUTTONS: readonly EditorButton[] = [
  { name: "bold", icon: "format_bold", label: "editor.bold", active: (e) => e.isActive("bold"), run: (e) => void e.chain().focus().toggleBold().run() },
  { name: "italic", icon: "format_italic", label: "editor.italic", active: (e) => e.isActive("italic"), run: (e) => void e.chain().focus().toggleItalic().run() },
  { name: "underline", icon: "format_underlined", label: "editor.underline", active: (e) => e.isActive("underline"), run: (e) => void e.chain().focus().toggleUnderline().run() },
  { name: "strike", icon: "strikethrough_s", label: "editor.strike", active: (e) => e.isActive("strike"), run: (e) => void e.chain().focus().toggleStrike().run() },
  { name: "bullets", icon: "format_list_bulleted", label: "editor.bullets", active: (e) => e.isActive("bulletList"), run: (e) => void e.chain().focus().toggleBulletList().run() },
  { name: "numbers", icon: "format_list_numbered", label: "editor.numbers", active: (e) => e.isActive("orderedList"), run: (e) => void e.chain().focus().toggleOrderedList().run() },
  { name: "quote", icon: "format_quote", label: "editor.quote", active: (e) => e.isActive("blockquote"), run: (e) => void e.chain().focus().toggleBlockquote().run() },
  {
    name: "link",
    icon: "link",
    label: "editor.link",
    active: (e) => e.isActive("link"),
    run: async (e, ask) => {
      if (e.isActive("link")) {
        e.chain().focus().unsetLink().run();
        return;
      }
      // Only the web and email: the server removes any other address when it keeps the text.
      const address = (await ask("editor.address"))?.trim();
      if (address && /^(https?:\/\/|mailto:)/.test(address)) e.chain().focus().setLink({ href: address }).run();
    },
  },
];
