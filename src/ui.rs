use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus, PromptKind};
use crate::compile::CompileOutcome;
use crate::dialog::{CompileDialog, DialogMode, FileDialog};
use crate::hitbox::{DropdownHitbox, EditorHitbox, FileDialogHitbox, Hitboxes, PanelHitbox};
use crate::menu::{Action, MenuItem};
use crate::syntax::{self, style_for};

/// Vrai si `path` correspond à un onglet actuellement ouvert et portant des
/// modifications non enregistrées (affiché en italique dans l'explorateur).
fn has_unsaved_changes(app: &App, path: &Path) -> bool {
    app.tabs
        .iter()
        .any(|b| b.modified && b.path.as_deref() == Some(path))
}

const EXPLORER_WIDTH: u16 = 28;
const OUTLINE_WIDTH: u16 = 32;
const EXPLORER_MIN_TOTAL: u16 = 70;
const OUTLINE_MIN_TOTAL: u16 = 110;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(size);
    let menu_area = root[0];
    let main_area = root[1];
    let status_area = root[2];

    let show_explorer = app.show_explorer && main_area.width >= EXPLORER_MIN_TOTAL;
    let show_outline = app.show_outline && main_area.width >= OUTLINE_MIN_TOTAL;

    let mut constraints = Vec::new();
    if show_explorer {
        constraints.push(Constraint::Length(EXPLORER_WIDTH));
    }
    constraints.push(Constraint::Min(20));
    if show_outline {
        constraints.push(Constraint::Length(OUTLINE_WIDTH));
    }
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(main_area);

    let mut idx = 0;
    let explorer_area = if show_explorer {
        let a = cols[idx];
        idx += 1;
        Some(a)
    } else {
        None
    };
    let center_area = cols[idx];
    idx += 1;
    let outline_area = if show_outline { Some(cols[idx]) } else { None };

    let menu_titles = draw_menu_bar(frame, app, menu_area);
    let explorer_hitbox = explorer_area.map(|area| draw_explorer(frame, app, area));
    let (tabs_hitboxes, editor_hitbox) = draw_center(frame, app, center_area);
    let outline_hitbox = outline_area.map(|area| draw_outline(frame, app, area));
    draw_status(frame, app, status_area);

    let menu_dropdown = if app.menu.active {
        Some(draw_menu_dropdown(frame, app, menu_area, size))
    } else {
        None
    };
    // `main_area` (jamais la ligne 0) sert de zone d'ancrage pour les popups,
    // afin que la barre de menu reste toujours visible tout en haut de l'écran.
    let prompt_hitbox = app.prompt.as_ref().map(|prompt| draw_prompt(frame, prompt, main_area));
    let about_hitbox = if app.about_open { Some(draw_about(frame, main_area)) } else { None };
    let file_dialog_hitbox = app
        .file_dialog
        .as_ref()
        .map(|dialog| draw_file_dialog(frame, dialog, main_area));
    if let Some(dialog) = &app.compile_dialog {
        draw_compile_dialog(frame, dialog, main_area);
    }
    if let Some(outcome) = &app.compile_result {
        draw_compile_result(frame, outcome, main_area);
    }

    app.hitboxes = Hitboxes {
        menu_titles,
        menu_dropdown,
        tabs: tabs_hitboxes,
        explorer: explorer_hitbox,
        outline: outline_hitbox,
        editor: Some(editor_hitbox),
        file_dialog: file_dialog_hitbox,
        prompt: prompt_hitbox,
        about: about_hitbox,
    };
}

fn draw_menu_bar(frame: &mut Frame, app: &App, area: Rect) -> Vec<Rect> {
    let mut spans = Vec::with_capacity(app.menu.menus.len());
    let mut titles = Vec::with_capacity(app.menu.menus.len());
    let mut x = area.x;
    for (i, menu_def) in app.menu.menus.iter().enumerate() {
        let selected = app.menu.active && i == app.menu.selected_menu;
        let style = if selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };
        let label = format!(" {} ", menu_def.title);
        let width = label.chars().count() as u16;
        titles.push(Rect { x, y: area.y, width, height: 1 });
        x += width;
        spans.push(Span::styled(label, style));
    }
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(paragraph, area);
    titles
}

fn menu_x_offset(app: &App, index: usize) -> u16 {
    app.menu.menus[..index]
        .iter()
        .map(|m| m.title.chars().count() as u16 + 2)
        .sum()
}

/// Ajoute une case à cocher devant les entrées qui basculent la visibilité
/// d'un panneau, pour refléter leur état actuel dans le menu Affichage.
fn item_label(app: &App, item: &MenuItem) -> String {
    match item.action {
        Some(Action::ToggleExplorer) => {
            let mark = if app.show_explorer { 'x' } else { ' ' };
            format!("[{mark}] {}", item.label)
        }
        Some(Action::ToggleOutline) => {
            let mark = if app.show_outline { 'x' } else { ' ' };
            format!("[{mark}] {}", item.label)
        }
        _ => item.label.to_string(),
    }
}

fn draw_menu_dropdown(frame: &mut Frame, app: &App, menu_area: Rect, screen: Rect) -> DropdownHitbox {
    let menu_def = &app.menu.menus[app.menu.selected_menu];
    let width = menu_def
        .items
        .iter()
        .map(|it| item_label(app, it).chars().count() + it.shortcut.chars().count() + 4)
        .max()
        .unwrap_or(10)
        .max(menu_def.title.chars().count() + 4) as u16
        + 2;
    let height = menu_def.items.len() as u16 + 2;
    let x = (menu_area.x + menu_x_offset(app, app.menu.selected_menu))
        .min(screen.x + screen.width.saturating_sub(width));
    let y = menu_area.y + 1;
    let area = Rect {
        x,
        y,
        width: width.min(screen.width),
        height: height.min(screen.height.saturating_sub(y)),
    };

    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut item_hitboxes = Vec::with_capacity(menu_def.items.len());
    let items: Vec<ListItem> = menu_def
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let row = Rect { x: inner.x, y: inner.y + i as u16, width: inner.width, height: 1 };
            if it.action.is_none() {
                item_hitboxes.push(None);
                return ListItem::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            item_hitboxes.push(Some(row));
            let selected = i == app.menu.selected_item;
            let style = if selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let label = item_label(app, it);
            let pad = (inner.width as usize)
                .saturating_sub(label.chars().count() + it.shortcut.chars().count() + 1);
            ListItem::new(Line::from(Span::styled(
                format!("{}{}{}", label, " ".repeat(pad.max(1)), it.shortcut),
                style,
            )))
        })
        .collect();
    let list = List::new(items);
    frame.render_widget(list, inner);
    DropdownHitbox { area, items: item_hitboxes }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

fn draw_prompt(frame: &mut Frame, prompt: &crate::app::Prompt, screen: Rect) -> Rect {
    let width = (prompt.label.chars().count().max(prompt.input.chars().count()) as u16 + 6)
        .clamp(30, 90);
    let area = centered_rect(width, 3, screen);
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Saisie ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = format!("{} {}", prompt.label, prompt.input);
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);

    if !matches!(prompt.kind, PromptKind::ConfirmQuit) {
        let cursor_x = inner.x
            + (prompt.label.chars().count() as u16 + 1 + prompt.input.chars().count() as u16)
                .min(inner.width.saturating_sub(1));
        frame.set_cursor_position((cursor_x, inner.y));
    }
    area
}

fn draw_about(frame: &mut Frame, screen: Rect) -> Rect {
    let lines = [
        "reditor — éditeur de texte façon IDE dans le terminal",
        "",
        "F10  Menu    F1  Aide    Ctrl+Q  Quitter",
        "F2/F3/F4  Explorateur / Éditeur / Structure",
        "Ctrl+N/O/S/W  Nouveau / Ouvrir / Enregistrer / Fermer",
        "Ctrl+X/C/V  Couper / Copier / Coller la ligne",
        "Ctrl+G  Aller à la ligne    Ctrl+B  Basculer l'explorateur",
        "",
        "Appuyez sur Échap ou Entrée pour fermer",
    ];
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(10) as u16 + 4;
    let height = lines.len() as u16 + 2;
    let area = centered_rect(width, height, screen);
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" À propos ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text: Vec<Line> = lines.iter().map(|l| Line::from(*l)).collect();
    frame.render_widget(Paragraph::new(text), inner);
    area
}

fn draw_file_dialog(frame: &mut Frame, dialog: &FileDialog, screen: Rect) -> FileDialogHitbox {
    let width = (screen.width * 3 / 4).clamp(40, 100).min(screen.width);
    let height = (screen.height * 3 / 4).clamp(10, screen.height);
    let area = centered_rect(width, height, screen);
    frame.render_widget(Clear, area);

    let title = match dialog.mode {
        DialogMode::Open => " Ouvrir un fichier ",
        DialogMode::SaveAs => " Enregistrer sous ",
        DialogMode::OpenFolder => " Ouvrir un dossier ",
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show_filename_row = dialog.mode == DialogMode::SaveAs;
    let mut constraints = vec![Constraint::Min(1), Constraint::Length(1)];
    if show_filename_row {
        constraints.insert(1, Constraint::Length(1));
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);
    let tree_area = rows[0];
    let filename_area = if show_filename_row { Some(rows[1]) } else { None };
    let hint_area = rows[rows.len() - 1];

    let items: Vec<ListItem> = dialog
        .browser
        .entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };
            let name = entry
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("?");
            let style = if entry.is_dir {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{indent}{icon}{name}"),
                style,
            )))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(dialog.browser.selected));
    let tree_focused = !dialog.editing_filename;
    let highlight_style = if tree_focused {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };
    let list = List::new(items).highlight_style(highlight_style);
    frame.render_stateful_widget(list, tree_area, &mut state);
    let tree_hitbox = PanelHitbox { inner: tree_area, offset: state.offset() };

    if let Some(fa) = filename_area {
        let label_style = if dialog.editing_filename {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let text = format!("Nom : {}", dialog.filename);
        frame.render_widget(Paragraph::new(text).style(label_style), fa);
        if dialog.editing_filename {
            let cursor_x = fa.x + 6 + dialog.filename.chars().count() as u16;
            frame.set_cursor_position((cursor_x.min(fa.x + fa.width.saturating_sub(1)), fa.y));
        }
    }

    let hint = match dialog.mode {
        DialogMode::Open => "↑↓ naviguer   →/Entrée ouvrir ou déplier   ← replier   Échap annuler",
        DialogMode::SaveAs => {
            "↑↓ naviguer   →/Entrée déplier/choisir   Tab nom de fichier   Échap annuler"
        }
        DialogMode::OpenFolder => {
            "↑↓ naviguer   → déplier   ← replier/remonter   Entrée choisir ce dossier   Échap annuler"
        }
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        hint_area,
    );

    FileDialogHitbox { area, tree: tree_hitbox, filename_area }
}

fn draw_compile_dialog(frame: &mut Frame, dialog: &CompileDialog, screen: Rect) {
    let width = 64u16.min(screen.width);
    let height = (dialog.jdks.len().max(1) as u16 + 4).min(screen.height);
    let area = centered_rect(width, height, screen);
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Configurer la compilation (Java) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let list_area = rows[0];
    let hint_area = rows[1];

    if dialog.jdks.is_empty() {
        let msg = Paragraph::new("Aucun JDK détecté (sdkman, JAVA_HOME, PATH)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, list_area);
    } else {
        let items: Vec<ListItem> = dialog
            .jdks
            .iter()
            .map(|jdk| ListItem::new(Line::from(format!("{}  ({})", jdk.label, jdk.javac.display()))))
            .collect();
        let mut state = ListState::default();
        state.select(Some(dialog.selected));
        let list = List::new(items).highlight_style(
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD),
        );
        frame.render_stateful_widget(list, list_area, &mut state);
    }

    frame.render_widget(
        Paragraph::new("↑↓ choisir   Entrée valider   Échap annuler")
            .style(Style::default().fg(Color::DarkGray)),
        hint_area,
    );
}

fn draw_compile_result(frame: &mut Frame, outcome: &CompileOutcome, screen: Rect) {
    let title = if outcome.success {
        " Compilation réussie "
    } else {
        " Échec de la compilation "
    };
    let width = (screen.width * 3 / 4).clamp(40, 100).min(screen.width);
    let height = (screen.height * 2 / 3).clamp(6, screen.height);
    let area = centered_rect(width, height, screen);
    frame.render_widget(Clear, area);
    let border_color = if outcome.success { Color::Green } else { Color::Red };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let text = if outcome.output.trim().is_empty() {
        "(aucune sortie)".to_string()
    } else {
        outcome.output.clone()
    };
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), rows[0]);
    frame.render_widget(
        Paragraph::new("Échap ou Entrée pour fermer").style(Style::default().fg(Color::DarkGray)),
        rows[1],
    );
}

fn draw_explorer(frame: &mut Frame, app: &App, area: Rect) -> PanelHitbox {
    let focused = app.focus == Focus::Explorer;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(" Explorateur ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = app
        .explorer
        .entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };
            let name = entry
                .path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("?");
            let mut style = if entry.is_dir {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            if !entry.is_dir && has_unsaved_changes(app, &entry.path) {
                style = style.add_modifier(Modifier::ITALIC);
            }
            ListItem::new(Line::from(Span::styled(
                format!("{indent}{icon}{name}"),
                style,
            )))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.explorer.selected));
    let highlight_style = if focused {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };
    let list = List::new(items).highlight_style(highlight_style);
    frame.render_stateful_widget(list, inner, &mut state);
    PanelHitbox { inner, offset: state.offset() }
}

fn draw_outline(frame: &mut Frame, app: &mut App, area: Rect) -> PanelHitbox {
    let focused = app.focus == Focus::Outline;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(" Structure ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items_data = app.current_outline();
    if app.outline_selected >= items_data.len() && !items_data.is_empty() {
        app.outline_selected = items_data.len() - 1;
    }

    if items_data.is_empty() {
        let msg = Paragraph::new("(aucun symbole détecté)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return PanelHitbox { inner, offset: 0 };
    }

    let items: Vec<ListItem> = items_data
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.depth);
            ListItem::new(Line::from(format!("{indent}{}", item.title)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.outline_selected));
    let highlight_style = if focused {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };
    let list = List::new(items).highlight_style(highlight_style);
    frame.render_stateful_widget(list, inner, &mut state);
    PanelHitbox { inner, offset: state.offset() }
}

fn draw_center(frame: &mut Frame, app: &mut App, area: Rect) -> (Vec<Rect>, EditorHitbox) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    let tabs_area = rows[0];
    let editor_area = rows[1];

    // Rendu manuel des onglets (plutôt que le widget `Tabs`) afin que chaque
    // onglet ait un `Rect` calculé par la même boucle qui construit l'affichage,
    // condition nécessaire pour qu'un clic souris retrouve l'onglet visé.
    let mut tab_spans = Vec::with_capacity(app.tabs.len() * 3);
    let mut tabs_hitboxes = Vec::with_capacity(app.tabs.len());
    let mut x = tabs_area.x;
    let tab_count = app.tabs.len();
    for (i, b) in app.tabs.iter().enumerate() {
        let marker = if b.modified { "*" } else { "" };
        let title = format!("{}{}", b.display_name(), marker);
        let title_style = if i == app.active_tab {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let tab_width = title.chars().count() as u16 + 2;
        tabs_hitboxes.push(Rect { x, y: tabs_area.y, width: tab_width, height: 1 });
        x += tab_width;
        tab_spans.push(Span::raw(" "));
        tab_spans.push(Span::styled(title, title_style));
        tab_spans.push(Span::raw(" "));
        if i + 1 < tab_count {
            tab_spans.push(Span::raw("│"));
            x += 1;
        }
    }
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tabs_area);

    let focused = app.focus == Focus::Editor;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(editor_area);
    frame.render_widget(block, editor_area);

    let height = inner.height as usize;
    let width = inner.width.saturating_sub(6) as usize; // place pour la marge de numéros de ligne
    let buf = app.current_buffer_mut();
    buf.ensure_cursor_visible(height, width.max(1));

    let start = buf.scroll_row;
    let end = (start + height).min(buf.lines.len());
    let gutter_width = buf.lines.len().to_string().len().max(3);
    let selection = buf.selection_range();

    let mut lines: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for i in start..end {
        let mut state = buf.highlight_states[i];
        let tokens = syntax::highlight_line(&buf.lines[i], buf.language, &mut state);
        let mut spans: Vec<Span> = Vec::with_capacity(tokens.len() + 1);
        spans.push(Span::styled(
            format!("{:>width$} ", i + 1, width = gutter_width),
            Style::default().fg(Color::DarkGray),
        ));
        // Sélection sur cette ligne (bornes en colonnes de caractères), ou `None`.
        let sel_on_line = selection.and_then(|((sl, sc), (el, ec))| {
            if i < sl || i > el {
                None
            } else {
                let from = if i == sl { sc } else { 0 };
                let to = if i == el { ec } else { buf.line_char_len(i) };
                (to > from).then_some((from, to))
            }
        });
        let mut col = 0usize;
        for (kind, text) in tokens {
            let token_len = text.chars().count();
            let token_start = col;
            let token_end = col + token_len;
            col = token_end;
            let base_style = style_for(kind);
            match sel_on_line {
                Some((sel_from, sel_to)) if token_end > sel_from && token_start < sel_to => {
                    let chars: Vec<char> = text.chars().collect();
                    let pre = sel_from.saturating_sub(token_start).min(token_len);
                    let post = sel_to.saturating_sub(token_start).min(token_len);
                    if pre > 0 {
                        spans.push(Span::styled(
                            chars[..pre].iter().collect::<String>(),
                            base_style,
                        ));
                    }
                    if post > pre {
                        spans.push(Span::styled(
                            chars[pre..post].iter().collect::<String>(),
                            base_style.add_modifier(Modifier::REVERSED),
                        ));
                    }
                    if post < token_len {
                        spans.push(Span::styled(
                            chars[post..].iter().collect::<String>(),
                            base_style,
                        ));
                    }
                }
                _ => spans.push(Span::styled(text, base_style)),
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).scroll((0, buf.scroll_col as u16));
    frame.render_widget(paragraph, inner);

    if focused {
        let cursor_x = inner.x
            + gutter_width as u16
            + 1
            + (buf.cursor_col.saturating_sub(buf.scroll_col)) as u16;
        let cursor_y = inner.y + (buf.cursor_line.saturating_sub(buf.scroll_row)) as u16;
        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    (tabs_hitboxes, EditorHitbox { inner, gutter_width })
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let buf = app.current_buffer();
    let modified = if buf.modified { " [modifié]" } else { "" };
    let left = format!(
        " {} — {}{}  Ln {}, Col {}",
        buf.language.label(),
        buf.display_name(),
        modified,
        buf.cursor_line + 1,
        buf.cursor_col + 1,
    );
    let right = app
        .status_message
        .clone()
        .unwrap_or_else(|| "F10 menu  F1 aide  Ctrl+S sauver  Ctrl+Q quitter".to_string());
    let width = area.width as usize;
    let mut text = left.clone();
    let padding = width.saturating_sub(left.len() + right.len() + 1);
    text.push_str(&" ".repeat(padding));
    text.push(' ');
    text.push_str(&right);
    if text.len() > width {
        text.truncate(width);
    }
    let style = if app.status_message.is_some() {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::Black).bg(Color::Gray)
    };
    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}
