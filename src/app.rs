use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::buffer::Buffer;
use crate::explorer::Explorer;
use crate::menu::{Action, MenuBar};
use crate::outline::{extract_outline, OutlineItem};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Explorer,
    Editor,
    Outline,
}

pub enum PromptKind {
    SaveAs,
    OpenFile,
    GoToLine,
    ConfirmQuit,
}

pub struct Prompt {
    pub kind: PromptKind,
    pub label: String,
    pub input: String,
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
    pub menu: MenuBar,
    pub clipboard: Option<String>,
    pub prompt: Option<Prompt>,
    pub about_open: bool,
}

impl App {
    pub fn new(
        root: PathBuf,
        initial_file: Option<PathBuf>,
        directory_opened: bool,
    ) -> anyhow::Result<Self> {
        let explorer = Explorer::new(root);
        let mut app = App {
            tabs: Vec::new(),
            active_tab: 0,
            explorer,
            focus: Focus::Editor,
            show_explorer: directory_opened,
            show_outline: true,
            outline_selected: 0,
            status_message: None,
            should_quit: false,
            menu: MenuBar::new(),
            clipboard: None,
            prompt: None,
            about_open: false,
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

    /// Exécute une action déclenchée depuis le menu ou un raccourci clavier.
    pub fn execute_action(&mut self, action: Action) {
        self.menu.close();
        match action {
            Action::NewFile => {
                self.tabs.push(Buffer::empty());
                self.active_tab = self.tabs.len() - 1;
                self.focus = Focus::Editor;
            }
            Action::OpenFile => {
                self.prompt = Some(Prompt {
                    kind: PromptKind::OpenFile,
                    label: "Ouvrir un fichier :".to_string(),
                    input: String::new(),
                });
            }
            Action::Save => self.save_current(),
            Action::SaveAs => {
                let default = self
                    .current_buffer()
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                self.prompt = Some(Prompt {
                    kind: PromptKind::SaveAs,
                    label: "Enregistrer sous :".to_string(),
                    input: default,
                });
            }
            Action::CloseTab => self.close_tab(self.active_tab),
            Action::Quit => {
                if self.tabs.iter().any(|b| b.modified) {
                    self.prompt = Some(Prompt {
                        kind: PromptKind::ConfirmQuit,
                        label: "Modifications non enregistrées. Quitter quand même ? (o/n)"
                            .to_string(),
                        input: String::new(),
                    });
                } else {
                    self.should_quit = true;
                }
            }
            Action::CutLine => {
                let text = self.current_buffer_mut().remove_current_line();
                self.clipboard = Some(text);
            }
            Action::CopyLine => {
                let text = self.current_buffer().current_line().to_string();
                self.clipboard = Some(text);
            }
            Action::PasteLine => {
                if let Some(text) = self.clipboard.clone() {
                    self.current_buffer_mut().insert_line_below(text);
                }
            }
            Action::GoToLine => {
                self.prompt = Some(Prompt {
                    kind: PromptKind::GoToLine,
                    label: "Aller à la ligne :".to_string(),
                    input: String::new(),
                });
            }
            Action::ToggleExplorer => self.show_explorer = !self.show_explorer,
            Action::ToggleOutline => self.show_outline = !self.show_outline,
            Action::About => self.about_open = true,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.about_open {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::F(1)) {
                self.about_open = false;
            }
            return;
        }

        if self.prompt.is_some() {
            self.handle_prompt_key(key);
            return;
        }

        if self.menu.active {
            self.handle_menu_key(key);
            return;
        }

        self.status_message = None;

        if key.code == KeyCode::F(10) {
            self.menu.open();
            return;
        }
        if key.code == KeyCode::F(1) {
            self.about_open = true;
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => self.execute_action(Action::Quit),
                KeyCode::Char('n') => self.execute_action(Action::NewFile),
                KeyCode::Char('o') => self.execute_action(Action::OpenFile),
                KeyCode::Char('s') => self.execute_action(Action::Save),
                KeyCode::Char('w') => self.execute_action(Action::CloseTab),
                KeyCode::Char('b') => self.execute_action(Action::ToggleExplorer),
                KeyCode::Char('g') => self.execute_action(Action::GoToLine),
                KeyCode::Char('x') => self.execute_action(Action::CutLine),
                KeyCode::Char('c') => self.execute_action(Action::CopyLine),
                KeyCode::Char('v') => self.execute_action(Action::PasteLine),
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

    fn handle_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::F(10) => self.menu.close(),
            KeyCode::Left => self.menu.move_left(),
            KeyCode::Right => self.menu.move_right(),
            KeyCode::Up => self.menu.move_up(),
            KeyCode::Down => self.menu.move_down(),
            KeyCode::Enter => {
                if let Some(action) = self.menu.selected_action() {
                    self.execute_action(action);
                } else {
                    self.menu.close();
                }
            }
            _ => {}
        }
    }

    fn handle_prompt_key(&mut self, key: KeyEvent) {
        let is_confirm_quit =
            matches!(self.prompt.as_ref(), Some(Prompt { kind: PromptKind::ConfirmQuit, .. }));
        if is_confirm_quit {
            match key.code {
                KeyCode::Enter | KeyCode::Char('o' | 'O' | 'y' | 'Y') => self.should_quit = true,
                _ => self.prompt = None,
            }
            return;
        }
        match key.code {
            KeyCode::Esc => self.prompt = None,
            KeyCode::Enter => self.confirm_prompt(),
            KeyCode::Backspace => {
                if let Some(p) = self.prompt.as_mut() {
                    p.input.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(p) = self.prompt.as_mut() {
                    p.input.push(c);
                }
            }
            _ => {}
        }
    }

    fn confirm_prompt(&mut self) {
        let Some(prompt) = self.prompt.take() else {
            return;
        };
        match prompt.kind {
            PromptKind::SaveAs => {
                let trimmed = prompt.input.trim();
                if trimmed.is_empty() {
                    self.status_message = Some("Chemin vide, enregistrement annulé".to_string());
                    return;
                }
                let path = PathBuf::from(trimmed);
                match self.current_buffer_mut().save_as(path) {
                    Ok(()) => self.status_message = Some("Fichier enregistré".to_string()),
                    Err(e) => self.status_message = Some(format!("Erreur : {e}")),
                }
            }
            PromptKind::OpenFile => {
                let trimmed = prompt.input.trim();
                if trimmed.is_empty() {
                    return;
                }
                let mut path = PathBuf::from(trimmed);
                if path.is_relative() {
                    path = self.explorer.root().join(path);
                }
                if let Err(e) = self.open_file(path) {
                    self.status_message = Some(format!("Erreur : {e}"));
                }
            }
            PromptKind::GoToLine => match prompt.input.trim().parse::<usize>() {
                Ok(n) if n >= 1 => self.current_buffer_mut().goto_line(n - 1),
                _ => self.status_message = Some("Numéro de ligne invalide".to_string()),
            },
            PromptKind::ConfirmQuit => {}
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
