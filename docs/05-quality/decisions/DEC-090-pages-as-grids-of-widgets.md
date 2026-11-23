---
id: DEC-090
status: accepted
---

# DEC-090 Pages as grids of widgets, each declaring the sizes it takes

- Date: 2026-11-23
- Context: The owner wants the home and the course page arranged visually, as freely as the widgets of a phone's home screen: each widget takes a space of its own, can grow or shrink within what it allows, and changes how it shows its content when its size changes. A widget must state in its declaration, without exception, where it can be placed and the sizes it takes there, from sizes everyone knows. Settings must not be left empty when they cannot be, texts must be edited in place and in every installed language, and long texts need a rich editor that modules and plugins extend. Until now a page had three fixed regions (top, main, side) and a widget only an order in its region (RN-216).
- Options considered:
  - Fixed regions with an order (until now).
  - Sections with columns, each column a stack of widgets.
  - A free grid of cells, each widget a rectangle within its declared sizes (chosen by the owner).
- Decision:
  - **Grid.** A page kind declares a grid: `columns` (12 for the home and the course page) and the height of a row, from typed row heights (`compact` 48 px, `regular` 72 px, `tall` 96 px). A placement is a rectangle of cells: `x`, `y`, `w`, `h`. Rectangles never overlap; moving or growing a widget pushes the ones in its way down. On a narrow screen the grid becomes one column, in reading order (by `y`, then `x`), each widget keeping its height.
  - **Sizes, typed and mandatory.** Every widget declares, for every page kind it can be placed on, its `min`, `default` and `max` size in cells, and how it may be resized: `none`, `horizontal`, `vertical` or `both`. A declaration without them does not compile. Where the widget is not declared, it cannot be placed: the page kinds of its sizes are the only places it goes (no wildcard). Sizes are checked against the grid of the page kind when the declarations are read, and every placement is checked against them when a layout is saved.
  - **Content by size.** A widget renders knowing the size it has in cells, and chooses how to show its content for it: the catalogue shows as many cards per row and per page as fit, the latest notifications show more or fewer, the welcome hides its figures when short. The sizes are not hints: a widget never overflows its rectangle and pages its content instead.
  - **Settings.** A setting declares whether it is `required` (an empty value is refused) and, for text, whether it is `translatable` (one value per installed language, falling back to the default language). A new kind, `rich_text`, holds HTML: it is sanitised on the server with ammonia before it is stored and again when it is read, so no script ever reaches a page.
  - **Editing in place.** In edit mode, the texts of a widget that declares them as editable in place (the heading of a widget, the text of the text widget, the title and the lead of the welcome) are edited on the page itself; a widget's other settings stay in its side panel. When more than one language is installed, a text being edited shows the languages it has, and the person switches between them without leaving the page.
  - **Rich editor.** Rich text is edited with an editor built on Tiptap, with the formatting of the design system. Its toolbar and its content are extensible: a module or a plugin registers extensions (a button, a kind of embedded content, a mention) through the registry of the client, as the server registers its modules (DEC-089).
  - **Fixed widgets.** A widget may declare one size for a page kind (`min`, `default` and `max` equal, resize `none`): the content of a course is such a widget, always the same size, so a course reads the same however it is arranged around it.
- Consequences:
  - Regions disappear from page kinds and layouts. Existing layouts are converted once by an upgrade step: the top region becomes full-width rows, the main region the left eight columns and the side the right four, each widget at its default height.
  - `WidgetDeclaration.page_kinds` is replaced by `sizes`, one entry per page kind. Every module that declares widgets declares their sizes.
  - The layout API returns and takes rectangles; the widgets a reader may place come with their sizes for the page kind.
  - The editor of a page is a grid where widgets are dragged and resized from their edges, with the pointer, by touch or with the keyboard (arrows move, shift and arrows resize).
- Delivered in two steps: the grid, the sizes and the editor first; required, translatable and rich settings, editing in place and the rich editor next.
- Later, noted by the owner: a left side as well as a right one, and preset arrangements of each page kind (such as content with a left side, or content between two sides) that the administration offers and a scope chooses as its default.
- Affected artifacts: the customisation data model and its API, `kampus-module` (page kind and widget declarations), every module that declares widgets, the [Kampus design system](../../03-design/web/design-system.md), RN-216, the web client.
