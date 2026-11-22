---
id: DESIGN-SYSTEM
status: accepted
origin: requirements
sourceOfTruth: model
---

# Kampus design system

How every screen of Kampus looks and behaves: in the shell, in the UI modules of Kampus modules and in plugins. It is implemented by `@kampus/ui` (`web/ui`) on React and MUI (DEC-089), with Didactika's identity (DEC-013), and themed per scope by configuration (RN-217).

## Principles

| Principle | What it means |
| --- | --- |
| Didactika prevails | Material Design and MUI are the base. Didactika's palette, typography, rules of meaning and patterns win wherever they differ |
| Latest stable versions | React, Next.js, MUI, Material Symbols and the tools follow their latest stable release; a version is pinned exactly and updated on purpose, never left behind |
| Scalable | One set of tokens, one theme, one set of components and patterns serves the shell, every module and every plugin; a new screen is made of what exists, and anything new is added here first |
| Dynamic | Colours, typefaces, size, corners, density, motion and scheme come from the theme of the scope at run time, with no code and no rebuild. Brand assets and icons follow in a later phase |
| Optimised | Pages are rendered on the server with their theme and data, so the first paint is complete. Components load only where they are used; icons are inline SVG with no icon font; fonts are served by Kampus itself |
| Intuitive | One meaning per element, the same everywhere; few choices per screen; every empty, loading and failed state says what is happening and what to do |
| Accessible | WCAG 2.2 AA in both schemes, checked in CI: contrast of every colour role, axe on every component, keyboard and screen-reader use |

## Layers

| Layer | Holds | Who changes it |
| --- | --- | --- |
| Reference tokens | Didactika's palettes: teal, slate, cyan and the colours of state (`web/ui/tokens/reference.tokens.json`) | Didactika |
| Colour roles | The roles of the light and the dark scheme (`light.tokens.json`, `dark.tokens.json`), which the server also grows from the seed colours of a scope | The theme of a scope |
| Theme | `createKampusTheme`: the roles and tokens of a scope as an MUI theme with CSS variables (`--k-palette-…`), and Didactika's look of every MUI component | Nobody per screen; the design system only |
| Components | Didactika's components over MUI, which fix meaning (`Button`, `TextField`, `ConfirmDialog`…) | The design system |
| Patterns | Compositions for recurring needs: page header, empty state, list toolbar, filters, table, pager, selection bar, wizard | The design system |
| Screens | Pages of the shell and of UI modules, made only of the layers above | Modules and plugins |

A screen never sets a colour, a font, a radius or a shadow of its own. It uses roles through the theme (`color="primary"`, `bgcolor: "primary.container"`, `var(--k-palette-surface-containerHigh)`), and components and patterns of the design system.

## Colour

| Role | Use |
| --- | --- |
| `primary` | What is chosen, active or the main action; links |
| `secondary` | Quieter actions and neutral badges |
| `accent` | Highlights and decoration (the tertiary role of Material) |
| `danger`, `success`, `warning`, `info` | Removes or fails; done or safe; needs attention; informs |
| `*-container`, `on-*-container` | Softer backgrounds of a colour and the text on them: tonal buttons, alerts, badges, selected rows |
| `brand`, `on-brand` | The app bar and the gradient of the brand |
| `surface`, `surface-container-lowest` to `-highest` | The page and the surfaces on it, from cards to dialogs |
| `on-surface`, `on-surface-variant`, `outline`, `outline-variant` | Text, quieter text, the edges of controls, dividers |
| `inverse-*`, `scrim`, `shadow` | Toasts and tooltips, the veil behind dialogs, tinted shadows |

In light, cards are white over a slate page; in dark, cards are one tone above the page. Each role keeps WCAG 2.2 AA contrast with the text on it, whatever the seeds of the scope.

## Typography

IBM Plex Sans for text and controls; Space Grotesk for page titles, section titles and the brand. Sizes are in rem, so the font size of the theme scales everything.

| MUI variant | Use | Size / line |
| --- | --- | --- |
| `h1` | The title of a page | 2 / 2.5 rem, Space Grotesk |
| `h2`, `h3` | Sections of a page | 1.75 and 1.5 rem, Space Grotesk |
| `h4` | Titles of cards and dialogs | 1.375 rem, Space Grotesk |
| `subtitle1`, `subtitle2` | Labels of lists and fields | 1 and 0.875 rem, medium |
| `body1`, `body2` | Text | 1 and 0.875 rem |
| `caption`, `overline` | Notes, codes, short names | 0.75 and 0.6875 rem |
| `button` | Buttons, sentence case | 0.875 rem, medium |

## Shape, density and motion

| Aspect | Rule |
| --- | --- |
| Corners | From the radius of the theme (12 px by default): fields, menus and chips at two thirds, cards at one, sheets at four thirds, dialogs at seven thirds; buttons and badges are pills. Radius 0 makes everything square |
| Density | `comfortable` (40 px controls, 48 px fields) or `compact` (32 px controls and small MUI sizes) |
| Elevation | Five levels of shadow tinted with the theme; cards are outlined by default and raise on hover when they open something |
| Motion | Short and purposeful: buttons narrow their corners while pressed with a spring; dialogs and toasts enter with emphasis. Reduced motion, from the theme or the system, removes it |

## Components

| Component | Meaning and rule |
| --- | --- |
| `Button` | `filled` is the main action of an area, and there is one per area; `tonal` and `elevated` are important actions beside it; `outlined` is an ordinary action (the default); `text` is the least. `tone="danger"` removes or stops and always goes through `ConfirmDialog`; `tone="success"` starts a process. `busy` while its action runs, so it is never pressed twice. An icon goes before the text |
| `ButtonGroup` | Buttons that belong together; `connected` makes them one control (a view switch, a segmented choice) |
| `TextField`, `SelectField` | The label is always above the field and never covers its value; the hint and the error sit under it and are tied to it; the error of the server shows until the person types |
| `ConfirmDialog` | Asks before removing or stopping: the title asks, the body says what stays and what is lost, the confirm button takes the tone of the action |
| `StatusBadge` | A state or a count, tinted with the colour of its state |
| `Loading` | The loading indicator, for short waits; long or measurable waits use MUI's progress |
| `ToastProvider`, `useToast` | The result of an asynchronous action, at the foot of the page, read out without moving focus; errors are announced at once |
| `Icon` | An icon by name; decorative unless labelled |
| MUI components | Used directly when the design system has no component of its own (cards, lists, tables, tabs, menus, drawers); the theme gives them Didactika's look |

## Icons

Icons are Material Symbols in the rounded style, by name, as inline SVG in the colour of the text. They live in a registry, where the design system and each UI module define the icons they use. One meaning has one icon everywhere:

| Icon | Meaning |
| --- | --- |
| `delete` | Removes; always in the danger tone |
| `edit` | Changes |
| `visibility`, `visibility_off` | Shows or hides; never "view" |
| `info` | Opens the details of something that cannot be changed any more |
| `history` | Opens the versions |

A theme overrides any icon by name with the address of an image, drawn as a mask in the colour of the text, so it follows the theme and runs nothing. The icons of the design system are generated into `src/icons/symbols.generated.ts` by `npm run icons -w @kampus/ui`.

## Patterns

| Pattern | Rule |
| --- | --- |
| Frame | The app bar of the brand with the main navigation, the appearance and the account; on a narrow window the navigation folds into a drawer; a link skips to the content |
| Page header | An optional way back, the title, one line that says what the page is for, and the actions of the page at its end |
| Lists | A toolbar with a search that applies as the person types and a Filters button with the number of filters on, then the actions of the page. The table has a fixed height and a sticky head, with the pager under it (12, 24, 48, 96, and All up to 500 rows). The name of a row opens it. An action a row cannot take is disabled, never hidden. A selection gets a floating bar |
| Cards of items | A grid of cards that open their item, with its code, its name and the start of its description |
| Wizards | A stepper and the step in a card, with Previous and Next at its foot |
| Empty, loading and failure | Empty says why and what to do; loading shows the indicator; a failure says so, offers to try again, and leaves the frame working |

## Theming per scope

The platform, a tenant, a campus, a category or a course sets the tokens of its theme; a scope below inherits them unless it sets its own or a scope above locks them.

| Step | What happens |
| --- | --- |
| Default | The tokens of `@kampus/ui` hold Didactika's theme in both schemes |
| A scope sets tokens | Seed colours, typefaces, size, radius, density, motion and preferred scheme; the server grows the roles of both schemes, keeping WCAG 2.2 AA contrast |
| A page renders | The server of the client reads the theme of the scope of the page and makes the MUI theme from it before the first paint; a missing role falls back to Didactika's |
| The person chooses | Light, dark or like the device, kept in their browser and applied before the page shows |

The default theme and the one the server grows for the default tokens are the same; a test of the customisation module compares them.

### Later phases

| What | How it will work |
| --- | --- |
| Every colour | Beyond the seeds, an advanced mode sets any colour role of either scheme, checked for contrast like the grown ones |
| Logos and favicon | The logo for light and dark backgrounds, the compact mark, the favicon and the application icon. They are uploaded once, and the sizes a browser asks for are derived from them |
| Brand images | The image of the sign-in page, the default cover of a course, the header of mail |
| Icon set | The style of Material Symbols (rounded, outlined or sharp), filled or not, and their weight |
| Own icons | Any icon replaced by an uploaded SVG or PNG, sanitised when stored. The editor lists every icon name with search, a preview and a way back. Rules can scope an override to a page kind or a region. Packages carry overrides to other scopes and campuses |

## Adding to the design system

1. Look for an existing component or pattern first. A screen that needs something new asks for it here, not in its module.
2. A component is a folder in `web/ui/src/components/<Component>/` with `<Component>.tsx` and `index.ts`. It wraps MUI, takes its look from the theme, and states its meaning in its documentation comment.
3. Every text comes from the caller; every colour is a role; corners, sizes and motion come from the theme.
4. Tests in `web/ui/tests/` run in Chromium: behaviour, keyboard, and axe in both schemes.
5. This document and the changelog say what it is for.
