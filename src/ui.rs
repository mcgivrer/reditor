use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, Focus, PromptKind};
use crate::syntax::{self, style_for};

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

    draw_menu_bar(frame, app, menu_area);
    if let Some(area) = explorer_area {
        draw_explorer(frame, app, area);
    }
    draw_center(frame, app, center_area);
    if let Some(area) = outline_area {
        draw_outline(frame, app, area);
    }
    draw_status(frame, app, status_area);

    if app.menu.active {
        draw_menu_dropdown(frame, app, menu_area, size);
    }
    if let Some(prompt) = &app.prompt {
        draw_prompt(frame, prompt, size);
    }
    if app.about_open {
        draw_about(frame, size);
    }
}

fn draw_menu_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::with_capacity(app.menu.menus.len());
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
        spans.push(Span::styled(format!(" {} ", menu_def.title), style));
    }
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}

fn menu_x_offset(app: &App, index: usize) -> u16 {
    app.menu.menus[..index]
        .iter()
        .map(|m| m.title.chars().count() as u16 + 2)
        .sum()
}

fn draw_menu_dropdown(frame: &mut Frame, app: &App, menu_area: Rect, screen: Rect) {
    let menu_def = &app.menu.menus[app.menu.selected_menu];
    let width = menu_def
        .items
        .iter()
        .map(|it| it.label.chars().count() + it.shortcut.chars().count() + 4)
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

    let items: Vec<ListItem> = menu_def
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            if it.action.is_none() {
                return ListItem::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            let selected = i == app.menu.selected_item;
            let style = if selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let pad = (inner.width as usize)
                .saturating_sub(it.label.chars().count() + it.shortcut.chars().count() + 1);
            ListItem::new(Line::from(Span::styled(
                format!("{}{}{}", it.label, " ".repeat(pad.max(1)), it.shortcut),
                style,
            )))
        })
        .collect();
    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

fn draw_prompt(frame: &mut Frame, prompt: &crate::app::Prompt, screen: Rect) {
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
}

fn draw_about(frame: &mut Frame, screen: Rect) {
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
}

fn draw_explorer(frame: &mut Frame, app: &App, area: Rect) {
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
}

fn draw_outline(frame: &mut Frame, app: &mut App, area: Rect) {
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
        return;
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
}

fn draw_center(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    let tabs_area = rows[0];
    let editor_area = rows[1];

    let titles: Vec<Line> = app
        .tabs
        .iter()
        .map(|b| {
            let marker = if b.modified { "*" } else { "" };
            Line::from(format!(" {}{} ", b.display_name(), marker))
        })
        .collect();
    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");
    frame.render_widget(tabs, tabs_area);

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

    let mut lines: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for i in start..end {
        let mut state = buf.highlight_states[i];
        let tokens = syntax::highlight_line(&buf.lines[i], buf.language, &mut state);
        let mut spans: Vec<Span> = Vec::with_capacity(tokens.len() + 1);
        spans.push(Span::styled(
            format!("{:>width$} ", i + 1, width = gutter_width),
            Style::default().fg(Color::DarkGray),
        ));
        for (kind, text) in tokens {
            spans.push(Span::styled(text, style_for(kind)));
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
