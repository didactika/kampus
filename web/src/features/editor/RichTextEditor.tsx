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

"use client";

/**
 * The rich editor of Kampus, on Tiptap: the formats of user content (bold, italic, underline, strike, lists, quotes,
 * links) and whatever the modules and plugins of the build register. It writes HTML in the format the server keeps and
 * sanitises, so what it shows is what will be stored. Bare, it is part of the page it edits: no border, and its
 * toolbar shows only while it has the focus.
 */

import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import Popover from "@mui/material/Popover";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import { EditorContent, useEditor, useEditorState } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useEffect, useRef, useState } from "react";

import { Button, Icon, TextField } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";
import { RICH_TEXT_LOOK } from "./RichText";
import { CORE_BUTTONS, editorExtensions, type Ask } from "./registry";

/** The props of `RichTextEditor`. */
export interface RichTextEditorProps {
  /** The HTML it starts with; another value replaces what it shows. */
  readonly value: string;
  /** Called with the HTML as it changes; empty when nothing is written. */
  readonly onChange: (html: string) => void;
  /** What it says while empty, already in the language of the page. */
  readonly placeholder?: string;
  /** Its name, for assistive technology. */
  readonly label: string;
  /** Whether it shows as part of the page it edits, without a border. */
  readonly bare?: boolean;
}

/** A question the editor asks, such as the address of a link. */
interface Question {
  readonly label: string;
  readonly text: string;
  readonly done: (text: string | null) => void;
}

/** `<RichTextEditor value={html} onChange={setHtml} label="Text" />` */
export function RichTextEditor({ value, onChange, placeholder, label, bare = false }: RichTextEditorProps) {
  const { t } = useI18n();
  const extensions = editorExtensions();
  const buttons = [...CORE_BUTTONS, ...extensions.flatMap((e) => e.buttons ?? [])];
  const editor = useEditor({
    immediatelyRender: false,
    extensions: [
      // The formats of user content: no headings, rules or images, which the server would remove.
      StarterKit.configure({ heading: false, horizontalRule: false, link: { openOnClick: false, autolink: true } }),
      ...extensions.flatMap((e) => e.extensions ?? []),
    ],
    content: value,
    editorProps: { attributes: { "aria-label": label, "aria-multiline": "true", role: "textbox" } },
    onUpdate: ({ editor: e }) => onChange(e.isEmpty ? "" : e.getHTML()),
  });
  // A value given from outside, such as another language, replaces what the editor shows.
  useEffect(() => {
    if (editor && value !== (editor.isEmpty ? "" : editor.getHTML())) editor.commands.setContent(value, { emitUpdate: false });
  }, [editor, value]);
  const active = useEditorState({
    editor,
    selector: ({ editor: e }) => Object.fromEntries(buttons.map((b) => [b.name, e ? b.active(e) : false])),
  });
  const toolbar = useRef<HTMLDivElement>(null);
  const [question, setQuestion] = useState<Question | null>(null);
  const ask: Ask = (key, initial = "") => new Promise((done) => setQuestion({ label: t(key), text: initial, done }));
  const answer = (text: string | null) => {
    question?.done(text);
    setQuestion(null);
  };
  return (
    <Box
      sx={{
        position: "relative",
        borderRadius: 3,
        border: bare ? 0 : 1,
        borderColor: "divider",
        bgcolor: bare ? "transparent" : "background.paper",
        "&:focus-within": bare ? {} : { borderColor: "primary.main" },
        "& .tiptap": { outline: "none", minBlockSize: bare ? 24 : 120, px: bare ? 0 : 1.5, py: bare ? 0 : 1, ...RICH_TEXT_LOOK },
        "& .tiptap p.is-editor-empty:first-of-type::before": {
          content: "attr(data-placeholder)",
          color: "text.disabled",
          float: "inline-start",
          blockSize: 0,
          pointerEvents: "none",
        },
        ...(bare ? { "& .editor-toolbar": { display: "none" }, "&:focus-within .editor-toolbar": { display: "flex" } } : {}),
      }}
    >
      <Stack
        ref={toolbar}
        className="editor-toolbar"
        direction="row"
        role="toolbar"
        aria-label={t("editor.toolbar")}
        sx={{
          flexWrap: "wrap",
          gap: 0.25,
          p: 0.5,
          ...(bare
            ? { position: "absolute", insetBlockEnd: "100%", insetInlineStart: 0, mb: 0.5, zIndex: 3, borderRadius: 999, bgcolor: "background.paper", border: 1, borderColor: "divider", boxShadow: 3 }
            : { borderBottom: 1, borderColor: "divider" }),
        }}
      >
        {buttons.map((button) => (
          <Tooltip key={button.name} title={t(button.label)}>
            <IconButton
              size="small"
              aria-label={t(button.label)}
              aria-pressed={Boolean(active?.[button.name])}
              color={active?.[button.name] ? "primary" : "default"}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => {
                if (editor) void button.run(editor, ask);
              }}
            >
              <Icon name={button.icon} size="small" />
            </IconButton>
          </Tooltip>
        ))}
      </Stack>
      <EditorContent editor={editor} data-placeholder={placeholder} />
      <Popover open={question !== null} anchorEl={toolbar.current} onClose={() => answer(null)} anchorOrigin={{ vertical: "bottom", horizontal: "left" }}>
        <Box
          component="form"
          sx={{ p: 2, display: "flex", gap: 1, alignItems: "flex-end", minInlineSize: 320 }}
          onSubmit={(event) => {
            event.preventDefault();
            answer(question?.text ?? null);
          }}
        >
          <TextField
            autoFocus
            label={question?.label ?? ""}
            value={question?.text ?? ""}
            onChange={(event) => setQuestion((q) => (q ? { ...q, text: event.target.value } : q))}
            sx={{ flex: 1 }}
          />
          <Button variant="filled" type="submit">
            {t("dashboard.apply")}
          </Button>
        </Box>
      </Popover>
    </Box>
  );
}
