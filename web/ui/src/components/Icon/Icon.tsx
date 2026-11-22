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


/**
 * An icon by name, in the colour of the text around it. It is decorative unless it has a label, which names it for
 * assistive technology. A theme that overrides the name changes it everywhere at once.
 */

import Box, { type BoxProps } from "@mui/material/Box";
import { useSyncExternalStore } from "react";

import "../../icons/core.js";
import { iconOf, onIconsChange } from "../../icons/registry.js";

/** The sizes of an icon: 20, 24 and 32 pixels at the default size of text. */
const SIZES = { small: "1.25rem", medium: "1.5rem", large: "2rem" } as const;

/** The props of `Icon`. */
export interface IconProps extends Omit<BoxProps<"span">, "children"> {
  /** The name of the icon, as Material Symbols names it (`delete`, `edit`, `visibility`…). */
  readonly name: string;
  /** What the icon means, when nothing beside it says so. */
  readonly label?: string;
  /** How large it is. */
  readonly size?: keyof typeof SIZES;
}

/** Counts changes of the registry, so a component renders again when the icons change. */
let version = 0;
const subscribe = (listener: () => void) =>
  onIconsChange(() => {
    version += 1;
    listener();
  });

/** `<Icon name="delete" />` */
export function Icon({ name, label, size = "medium", sx, ...rest }: IconProps) {
  useSyncExternalStore(subscribe, () => version, () => version);
  const source = iconOf(name);
  const a11y = label ? { role: "img", "aria-label": label } : { "aria-hidden": true };
  const box = {
    display: "inline-flex",
    flex: "none",
    inlineSize: SIZES[size],
    blockSize: SIZES[size],
    verticalAlign: "middle",
    "& svg": { inlineSize: "100%", blockSize: "100%", fill: "currentColor" },
  };
  if (source && "url" in source) {
    return (
      <Box
        component="span"
        data-icon={name}
        {...a11y}
        {...rest}
        sx={[
          box,
          { backgroundColor: "currentColor", mask: `url(${JSON.stringify(source.url)}) center / contain no-repeat` },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
      />
    );
  }
  return (
    <Box
      component="span"
      data-icon={name}
      {...a11y}
      {...rest}
      sx={[box, ...(Array.isArray(sx) ? sx : [sx])]}
      // The SVG is shipped with the code of the design system or of a module, never read from a person.
      dangerouslySetInnerHTML={source ? { __html: source.svg } : undefined}
    />
  );
}
