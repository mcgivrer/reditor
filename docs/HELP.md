# reditor — User Guide

`reditor` is a terminal-based, IDE-style text editor written in Rust. It runs
entirely inside a terminal — no GUI, no server — and gives you a file
explorer, a tabbed multi-file editor, syntax highlighting, a symbol outline
panel, and a desktop-style menu bar, all driven from the keyboard (with
optional mouse support).

You are reading the same manual that ships inside reditor itself. Open it
any time from **Aide > Manuel utilisateur** in the menu bar (see
[§10](#10-opening-this-guide-again)).

> **A note on language.** reditor's on-screen interface (menus, panel
> titles, dialog titles, status messages) is in French. The diagrams below
> reproduce the exact text you will see on screen, with English
> explanations alongside, so this guide stays accurate to what's actually
> running.

## Table of contents

1. [Getting started](#1-getting-started)
2. [The main window](#2-the-main-window)
3. [File explorer](#3-file-explorer)
4. [Tabs and editing](#4-tabs-and-editing)
5. [Syntax highlighting](#5-syntax-highlighting)
6. [Outline panel](#6-outline-panel)
7. [Menu bar](#7-menu-bar)
8. [Open / Save As dialog](#8-open--save-as-dialog)
9. [About dialog and prompts](#9-about-dialog-and-prompts)
10. [Opening this guide again](#10-opening-this-guide-again)
11. [Keyboard shortcuts reference](#11-keyboard-shortcuts-reference)
12. [Mouse support](#12-mouse-support)
13. [Tips and known limitations](#13-tips-and-known-limitations)

## 1. Getting started

```sh
reditor                 # open the current directory
reditor my-project/     # open a specific directory (explorer shown)
reditor my-file.rs      # open a single file directly (explorer hidden)
reditor --help          # command-line usage
reditor --version       # print the version
```

When you point reditor at a directory, the file explorer is shown by
default. When you point it at a single file (or start it with no argument
at all), the explorer starts hidden so the editor gets the full width — you
can still open it any time with **Ctrl+B**.

## 2. The main window

The screen is split into four fixed zones: a menu bar on top, an explorer
panel on the left, the tabbed editor in the center, an outline panel on the
right, and a status bar at the bottom. The explorer and outline panels are
optional and hide themselves automatically if the terminal is too narrow.

```text
┌ Fichier  Édition  Affichage  Aide ──────────────────────────────────────────┐
├ Explorateur ──────────┬ notes.txt │ main.rs* ────────────┬ Structure ───────┤
│ ▾ my-project           │   1 fn main() {                  │ fn main          │
│   ▾ src                │   2     let name = "reditor";    │ struct Config    │
│     main.rs            │   3     println!("hi, {name}");  │ impl Config      │
│     lib.rs             │   4 }                            │   fn load        │
│   Cargo.toml           │   5                              │   fn save        │
│   README.md            │   6                              │                  │
├───────────────────────┴──────────────────────────────────┴──────────────────┤
│ Rust — main.rs [modifié]  Ln 1, Col 1     F10 menu  F1 aide  Ctrl+S sauver… │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **Menu bar** (top) — always visible. Press **F10** to activate it.
- **Explorateur** (left, optional) — the working directory tree.
- **Editor** (center) — open tabs plus the text area.
- **Structure** (right, optional) — symbols found in the active file.
- **Status bar** (bottom) — detected language, file name, modified state,
  cursor position, and either a shortcut reminder or the last status
  message (e.g. a save confirmation or an error).

## 3. File explorer

```text
┌ Explorateur ─────────────┐
│ ▾ my-project              │
│   ▾ src                   │
│     main.rs                │
│     lib.rs                  │
│   ▸ tests                    │
│   Cargo.toml                  │
│   README.md                    │
└─────────────────────────────────┘
```

- `▾` / `▸` mark expanded / collapsed directories.
- Files that are open in a tab with unsaved changes are shown in italics.
- Navigation: **↑/↓** (or `k`/`j`) to move the selection, **→ / Enter / l**
  to open a file or expand/collapse a directory, **← / h** to collapse or
  go back up to the parent.
- Opening a file from the explorer opens it in a new tab (or focuses the
  existing tab if it's already open) and returns focus to the editor.
- Focus the panel with **F2** or **Ctrl+E**; return to the editor with
  **Esc** or **F3**.

## 4. Tabs and editing

```text
 notes.txt │ main.rs* │ Cargo.toml
┌──────────────────────────────────────────────────────────┐
│   1 fn main() {                                           │
│   2     let name = "reditor";                             │
│   3     println!("hello, {name}!");                       │
│   4 }                                                      │
│   5                                                        │
└──────────────────────────────────────────────────────────┘
```

- Every open file gets its own tab. The active tab is highlighted; a `*`
  after the name means unsaved changes.
- Standard editing: typing inserts characters, **Enter** splits the line,
  **Backspace**/**Delete** remove characters, **Tab** inserts 4 spaces,
  arrow keys / **Home** / **End** / **Page Up** / **Page Down** move the
  cursor.
- Line numbers are shown in the left gutter. The view scrolls
  automatically, both horizontally and vertically, to keep the cursor
  visible.
- A tab with no file yet is shown as "sans titre" ("untitled"); saving it
  goes through **Save As...**.
- Switch tabs with **Ctrl+←** / **Ctrl+→**; close the active tab with
  **Ctrl+W**.
- Click and drag with the mouse (or use **Ctrl+X/C/V**) to cut, copy and
  paste — either the current selection, or, with no selection, the whole
  current line.

## 5. Syntax highlighting

The language of a file is detected automatically from its extension (or,
for a few extensionless files, from its exact name — e.g. `Makefile`,
`Dockerfile`). Keywords, strings, numbers, comments, functions, tags,
attributes and section headers are then colored according to rules
specific to the detected language.

Explicitly supported languages: **Rust, Java, Kotlin, JavaScript,
TypeScript, Bash, Markdown, HTML, CSS, Properties, INI**, plus **JSON,
TOML, YAML, Python, C, C++, Go, XML**. Anything else is treated as plain
text, with no highlighting.

The detected language is shown at the start of the status bar, and updates
automatically after a **Save As...** to a new extension.

## 6. Outline panel

```text
┌ Structure ───────────┐
│ fn main               │
│ struct Config          │
│ impl Config              │
│   fn load                 │
│   fn save                   │
│ trait Loader                  │
└─────────────────────────────────┘
```

Lists the notable symbols of the active file, indented by nesting depth:

| Language | Symbols listed |
|---|---|
| Rust | `fn`, `struct`, `enum`, `trait`, `impl`, `mod` |
| Java / Kotlin | `class`, `interface`, `enum`, `object`, `fun` |
| JavaScript / TypeScript | classes, functions, constants assigned an arrow function |
| Python | `def`, `class` (indented per Python's own indentation level) |
| Go / C / C++ | `func`, `struct`, `class`, `interface`, `type` |
| Markdown | headings `#`…`######`, indented by level |
| Bash | declared functions (`function name` or `name()`) |
| CSS | rule selectors |
| HTML / XML | `h1`–`h6`, `script`, `style`, `body`, `head`, elements with an `id` |
| INI / TOML | `[section]` headers |
| Properties | every declared key |
| YAML | top-level keys, indented by depth |

Navigation: **↑/↓** to select a symbol, **Enter** to move the editor
cursor to that line. Focus the panel with **F4** or **Ctrl+L**; return to
the editor with **Esc** or **F3**.

## 7. Menu bar

A desktop-style menu bar, activated with **F10** and navigated with the
arrow keys (**←/→** to change menu, **↑/↓** to change entry — separators
are skipped), **Enter** to run the selected entry, **Esc** or **F10** to
close without doing anything.

```text
┌──────────────────────────────────┐
│ Nouveau                   Ctrl+N │
│ Ouvrir...                 Ctrl+O │
│ Enregistrer               Ctrl+S │
│ Enregistrer sous...              │
│ Fermer l'onglet           Ctrl+W │
├──────────────────────────────────┤
│ Quitter                   Ctrl+Q │
└──────────────────────────────────┘
```
*The "Fichier" (File) menu, open.*

| Menu | On-screen entry | Shortcut | What it does |
|---|---|---|---|
| Fichier (File) | Nouveau | Ctrl+N | New empty ("sans titre") tab |
| Fichier | Ouvrir... | Ctrl+O | Open the file picker dialog |
| Fichier | Enregistrer | Ctrl+S | Save the active tab to its file |
| Fichier | Enregistrer sous... | — | Save-as dialog; language is re-detected from the new extension |
| Fichier | Fermer l'onglet | Ctrl+W | Close the active tab |
| Fichier | Quitter | Ctrl+Q | Quit (asks for confirmation if there are unsaved changes) |
| Édition (Edit) | Couper | Ctrl+X | Cut the selection, or the current line |
| Édition | Copier | Ctrl+C | Copy the selection, or the current line |
| Édition | Coller | Ctrl+V | Paste |
| Édition | Aller à la ligne... | Ctrl+G | Prompt for a line number to jump to |
| Affichage (View) | Explorateur | Ctrl+B | Toggle the explorer panel (checkbox reflects state) |
| Affichage | Structure | — | Toggle the outline panel (checkbox reflects state) |
| Aide (Help) | À propos | F1 | Show the shortcut-reminder popup |
| Aide | Manuel utilisateur | — | Open this user guide in a new tab |

## 8. Open / Save As dialog

Both **Ouvrir...** (Ctrl+O) and **Enregistrer sous...** open the same
directory-tree picker.

```text
┌ Ouvrir un fichier ─────────────────────────────────────────────────────┐
│ ▾ my-project                                                            │
│   ▾ src                                                                 │
│     main.rs                                                             │
│     lib.rs                                                              │
│   ▸ tests                                                                │
│   Cargo.toml                                                              │
│                                                                             │
│ ↑↓ naviguer   →/Entrée ouvrir ou déplier   ← replier   Échap annuler       │
└─────────────────────────────────────────────────────────────────────────────┘
```

```text
┌ Enregistrer sous ───────────────────────────────────────────────────────────┐
│ ▾ my-project                                                                 │
│   ▸ src                                                                      │
│   Cargo.toml                                                                 │
│   README.md                                                                  │
│                                                                                │
│ Nom : notes.txt                                                                │
│ ↑↓ naviguer   →/Entrée déplier/choisir   Tab nom de fichier   Échap annuler     │
└──────────────────────────────────────────────────────────────────────────────────┘
```

- **↑/↓** (or `j`/`k`) moves the selection, **→ / Enter / l** expands a
  directory or picks a file, **←/h** collapses.
- In **Enregistrer sous...**, picking an entry fills in its name in the
  **Nom** (Name) field below the tree; press **Tab** to switch focus to
  that field and edit it directly.
- **Esc** cancels and closes the dialog without changing anything.

## 9. About dialog and prompts

**F1** toggles a small reminder of the core shortcuts:

```text
┌ À propos ──────────────────────────────────────────────────┐
│ reditor — éditeur de texte façon IDE dans le terminal        │
│                                                                 │
│ F10  Menu    F1  Aide    Ctrl+Q  Quitter                        │
│ F2/F3/F4  Explorateur / Éditeur / Structure                      │
│ Ctrl+N/O/S/W  Nouveau / Ouvrir / Enregistrer / Fermer              │
│ Ctrl+X/C/V  Couper / Copier / Coller la ligne                       │
│ Ctrl+G  Aller à la ligne    Ctrl+B  Basculer l'explorateur            │
│                                                                          │
│ Appuyez sur Échap ou Entrée pour fermer                                   │
└────────────────────────────────────────────────────────────────────────────┘
```

Small pop-up prompts use the same style, for example **Ctrl+G** (go to
line) or the unsaved-changes confirmation shown on **Ctrl+Q**:

```text
┌ Saisie ──────────────────────────────────┐
│ Aller à la ligne : 42_                    │
└───────────────────────────────────────────┘
```

Press **Enter** to confirm, **Esc** to cancel.

## 10. Opening this guide again

From anywhere in reditor:

1. Press **F10** to activate the menu bar (or click a menu title).
2. Move to the **Aide** menu (rightmost).
3. Select **Manuel utilisateur** and press **Enter** (or click it).

This guide opens as a regular, read-only-by-convention tab named
`HELP.md`, rendered with Markdown syntax highlighting like any other
Markdown file — reopening it while it's already open just switches back to
its tab instead of opening a duplicate.

## 11. Keyboard shortcuts reference

| Shortcut | Action |
|---|---|
| F10 | Open/close the menu bar |
| F1 | Show/hide the "À propos" (About) popup |
| F2 | Focus the explorer |
| F3 | Focus the editor |
| F4 | Focus the outline |
| Ctrl+N | New file |
| Ctrl+O | Open a file (picker dialog) |
| Ctrl+S | Save |
| Ctrl+W | Close the active tab |
| Ctrl+Q | Quit (asks for confirmation if there are unsaved changes) |
| Ctrl+B | Toggle the explorer panel |
| Ctrl+G | Go to a given line |
| Ctrl+X / Ctrl+C / Ctrl+V | Cut / copy / paste the selection or current line |
| Ctrl+E | Focus the explorer |
| Ctrl+L | Focus the outline |
| Ctrl+← / Ctrl+→ | Previous / next tab |
| Esc | Close a menu/prompt/dialog, or return to the editor from a panel |

## 12. Mouse support

Where the terminal relays mouse events, reditor also responds to:

- **Click** a menu title to open it; click an entry to run it.
- **Click** a tab to switch to it.
- **Click** an entry in the explorer to select it, or open it if it's a
  file (expanding/collapsing directories).
- **Click** an entry in the outline to jump the cursor to that symbol.
- **Click** in the editor to place the cursor; **click-drag** to select
  text; **double-click** a word to select it.
- **Scroll wheel** scrolls the focused panel (editor, explorer or
  outline).

## 13. Tips and known limitations

- There is no undo/redo.
- Cut/Copy/Paste act on the current text selection, or, with nothing
  selected, on the whole current line. Panels can't be resized, and there
  is no mouse-draggable scrollbar.
- Syntax highlighting is based on simple per-language lexical rules, not a
  full parser — rare edge cases can be colored incorrectly (for instance,
  variable interpolation inside a quoted Bash string).
- The explorer is for browsing and opening only: it can't create, rename
  or delete files or directories.
- A tab without a file path (a new, never-saved tab, or this help guide)
  can't be saved with **Ctrl+S** directly — use **Enregistrer sous...** to
  give it a location on disk.
