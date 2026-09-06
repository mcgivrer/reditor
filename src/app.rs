use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::buffer::Buffer;
use crate::explorer::Explorer;
use crate::outline::{extract_outline, OutlineItem};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Explorer,
    Editor,
    Outline,
}

pub struct App {
    pub tabs: Vec<Buffer>,
    pub active_tab: usize,
    pub explorer: Explorer,
    pub focus: Focus,
    pub show_explorer: bool,
    pub show_outline: bool,
    pub outline_selected: usize,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(root: PathBuf, initial_file: Option<PathBuf>) -> anyhow::Result<Self> {
        let explorer = Explorer::new(root);
        let mut app = App {
            tabs: Vec::new(),
            active_tab: 0,
            explorer,
            focus: Focus::Editor,
            show_explorer: true,
            show_outline: true,
            outline_selected: 0,
            status_message: None,
            should_quit: false,
        };
        match initial_file {
            Some(path) if path.is_file() => app.open_file(path)?,
            _ => app.tabs.push(Buffer::empty()),
        }
        Ok(app)
    }

    pub fn current_buffer(&self) -> &Buffer {
        &self.tabs[self.active_tab]
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.tabs[self.active_tab]
    }

    pub fn current_outline(&self) -> Vec<OutlineItem> {
        let buf = self.current_buffer();
        extract_outline(&buf.lines, buf.language)
    }

    pub fn open_file(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|b| b.path.as_deref() == Some(path.as_path()))
        {
            self.active_tab = idx;
        } else {
            let buf = Buffer::from_path(path)?;
            let replace_empty = self.tabs.len() == 1
                && self.tabs[0].path.is_none()
                && !self.tabs[0].modified
                && self.tabs[0].lines.len() == 1
                && self.tabs[0].lines[0].is_empty();
            if replace_empty {
                self.tabs[0] = buf;
                self.active_tab = 0;
            } else {
                self.tabs.push(buf);
                self.active_tab = self.tabs.len() - 1;
            }
        }
        self.focus = Focus::Editor;
        self.outline_selected = 0;
        Ok(())
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        if self.tabs.len() == 1 {
            self.tabs[0] = Buffer::empty();
            self.active_tab = 0;
            return;
        }
        self.tabs.remove(idx);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
        }
    }

    pub fn save_current(&mut self) {
        let name = self.current_buffer().display_name();
        match self.current_buffer_mut().save() {
            Ok(()) => self.status_message = Some(format!("{name} enregistré")),
            Err(e) => self.status_message = Some(format!("Erreur : {e}")),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.status_message = None;

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('s') => self.save_current(),
                KeyCode::Char('w') => self.close_tab(self.active_tab),
                KeyCode::Char('b') => self.show_explorer = !self.show_explorer,
                KeyCode::Char('o') => self.show_outline = !self.show_outline,
                KeyCode::Right => self.next_tab(),
                KeyCode::Left => self.prev_tab(),
                KeyCode::Char('e') => self.focus = Focus::Explorer,
                KeyCode::Char('l') => self.focus = Focus::Outline,
                _ => {}
            }
            // On ignore toute autre combinaison Ctrl+X pour éviter
            // d'insérer un caractère de contrôle dans le texte.
            return;
        }

        match key.code {
            KeyCode::F(2) => {
                self.focus = Focus::Explorer;
                return;
            }
            KeyCode::F(3) => {
                self.focus = Focus::Editor;
                return;
            }
            KeyCode::F(4) => {
                self.focus = Focus::Outline;
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Explorer => self.handle_explorer_key(key),
            Focus::Outline => self.handle_outline_key(key),
            Focus::Editor => self.handle_editor_key(key),
        }
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.explorer.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.explorer.move_down(),
            KeyCode::Left | KeyCode::Char('h') => self.explorer.collapse_selected(),
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(path) = self.explorer.activate_selected()
                    && let Err(e) = self.open_file(path) {
                        self.status_message = Some(format!("Erreur : {e}"));
                    }
            }
            KeyCode::Esc => self.focus = Focus::Editor,
            _ => {}
        }
    }

    fn handle_outline_key(&mut self, key: KeyEvent) {
        let items = self.current_outline();
        if items.is_empty() {
            if key.code == KeyCode::Esc {
                self.focus = Focus::Editor;
            }
            return;
        }
        if self.outline_selected >= items.len() {
            self.outline_selected = items.len() - 1;
        }
        match key.code {
            KeyCode::Up => {
                if self.outline_selected > 0 {
                    self.outline_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.outline_selected + 1 < items.len() {
                    self.outline_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(item) = items.get(self.outline_selected) {
                    let line = item.line;
                    self.current_buffer_mut().goto_line(line);
                }
                self.focus = Focus::Editor;
            }
            KeyCode::Esc => self.focus = Focus::Editor,
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        let buf = self.current_buffer_mut();
        match key.code {
            KeyCode::Char(c) => buf.insert_char(c),
            KeyCode::Enter => buf.insert_newline(),
            KeyCode::Backspace => buf.backspace(),
            KeyCode::Delete => buf.delete_forward(),
            KeyCode::Tab => buf.insert_tab(),
            KeyCode::Left => buf.move_left(),
            KeyCode::Right => buf.move_right(),
            KeyCode::Up => buf.move_up(),
            KeyCode::Down => buf.move_down(),
            KeyCode::Home => buf.move_home(),
            KeyCode::End => buf.move_end(),
            KeyCode::PageUp => buf.move_page(-20),
            KeyCode::PageDown => buf.move_page(20),
            _ => {}
        }
    }
}
