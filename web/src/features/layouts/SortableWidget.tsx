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
 * A widget of a page being arranged, dragged by its whole frame: with the pointer or by touch, or with the keyboard
 * once it has the focus (space or enter lifts it, the arrows move it, space or enter drops it, escape cancels).
 * While it is dragged, the room it will take shows where it lands and the others make way for it.
 */

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { KeyboardEvent, ReactNode } from "react";

import { useI18n } from "../../i18n/I18nProvider";
import { WidgetFrame } from "./WidgetFrame";

/** The props of `SortableWidget`. */
export interface SortableWidgetProps {
  /** The key of the widget in the page being arranged. */
  readonly id: number;
  /** The name of the widget, in the language of the page. */
  readonly name: string;
  /** Its icon. */
  readonly icon: string;
  /** Whether it is the one selected. */
  readonly selected: boolean;
  /** Selects it. */
  readonly onSelect: () => void;
  /** Opens its settings; none when it declares no settings. */
  readonly onSettings?: () => void;
  /** Removes it from the page. */
  readonly onRemove: () => void;
  /** Its content, or what stands for it until the page is saved. */
  readonly children: ReactNode;
}

/** `<SortableWidget id={3} name="Catalogue" icon="school" … >{preview}</SortableWidget>` */
export function SortableWidget({ id, name, icon, selected, onSelect, onSettings, onRemove, children }: SortableWidgetProps) {
  const { t } = useI18n();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id });
  // Keys pressed on the buttons inside the frame are theirs; only keys on the frame itself move it.
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.target === event.currentTarget) listeners?.["onKeyDown"]?.(event);
  };
  return (
    <WidgetFrame
      ref={setNodeRef}
      name={name}
      icon={icon}
      selected={selected}
      placeholder={isDragging}
      onSettings={onSettings}
      onRemove={onRemove}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      {...attributes}
      {...listeners}
      onKeyDown={onKeyDown}
      aria-label={t("drag.handle", { widget: name })}
      onClick={onSelect}
    >
      {children}
    </WidgetFrame>
  );
}
