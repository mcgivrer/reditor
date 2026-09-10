use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::buffer::Buffer;
use crate::dialog::{DialogMode, FileDialog};
use crate::explorer::Explorer;
use crate::hitbox::{EditorHitbox, Hitboxes};
use crate::menu::{Action, MenuBar};
use crate::outline::{extract_outline, OutlineItem};
use crate::syntax::Language;

/// Nom d'onglet affiché pour le manuel utilisateur embarqué.
const HELP_TAB_NAME: &str = "HELP.md";
/// Contenu du manuel utilisateur, embarqué dans le binaire à la compilation
/// pour rester accessible sans dépendre d'un fichier présent sur disque.
const HELP_CONTENT: &str = include_str!("../docs/HELP.md");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Explorer,
    Editor,
    Outline,
}

#[derive(Debug)]
pub enum PromptKind {
    GoToLine,
    ConfirmQuit,
}

#[derive(Debug)]
pub struct Prompt {
    pub kind: PromptKind,
    pub label: String,
    pub input: String,
}

/// Contenu du presse-papiers : une ligne entière (Couper/Copier la ligne
/// sans sélection) ou un fragment de texte arbitraire (sélection), pour que
/// Coller sache s'il doit insérer une nouvelle ligne ou du texte en place.
#[derive(Debug, Clone)]
pub enum ClipboardContent {
    Line(String),
    Fragment(String),
}

#[derive(Debug)]
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
    pub clipboard: Option<ClipboardContent>,
    pub prompt: Option<Prompt>,
    pub file_dialog: Option<FileDialog>,
    pub about_open: bool,
    pub hitboxes: Hitboxes,
    /// Position et instant du dernier clic gauche relâché, pour détecter un double-clic.
    pub last_click: Option<(Instant, u16, u16)>,
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
            file_dialog: None,
            about_open: false,
            hitboxes: Hitboxes::default(),
            last_click: None,
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
            self.push_tab(buf);
        }
        self.focus = Focus::Editor;
        self.outline_selected = 0;
        Ok(())
    }

    /// Ouvre (ou active l'onglet déjà ouvert pour) le manuel utilisateur
    /// embarqué dans le binaire.
    pub fn open_help(&mut self) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|b| b.virtual_name.as_deref() == Some(HELP_TAB_NAME))
        {
            self.active_tab = idx;
        } else {
            self.push_tab(Buffer::from_content(
                HELP_TAB_NAME,
                HELP_CONTENT,
                Language::Markdown,
            ));
        }
        self.focus = Focus::Editor;
        self.outline_selected = 0;
    }

    /// Insère `buf` dans un nouvel onglet, sauf s'il n'y a qu'un unique
    /// onglet vide et sans nom, auquel cas il le remplace.
    fn push_tab(&mut self, buf: Buffer) {
        let replace_empty = self.tabs.len() == 1
            && self.tabs[0].path.is_none()
            && self.tabs[0].virtual_name.is_none()
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
            Ok(()) => {
                self.status_message = Some(format!("{name} enregistré"));
                self.explorer.refresh();
            }
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
                self.file_dialog = Some(FileDialog::new(
                    DialogMode::Open,
                    self.explorer.root().to_path_buf(),
                    String::new(),
                ));
            }
            Action::Save => self.save_current(),
            Action::SaveAs => {
                let default_name = self
                    .current_buffer()
                    .path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| "sans-titre.txt".to_string());
                self.file_dialog = Some(FileDialog::new(
                    DialogMode::SaveAs,
                    self.explorer.root().to_path_buf(),
                    default_name,
                ));
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
                let buf = self.current_buffer_mut();
                if buf.has_selection() {
                    let text = buf.selected_text().unwrap_or_default();
                    buf.delete_selection();
                    self.clipboard = Some(ClipboardContent::Fragment(text));
                } else {
                    let text = buf.remove_current_line();
                    self.clipboard = Some(ClipboardContent::Line(text));
                }
            }
            Action::CopyLine => {
                let buf = self.current_buffer();
                let content = match buf.selected_text() {
                    Some(text) => ClipboardContent::Fragment(text),
                    None => ClipboardContent::Line(buf.current_line().to_string()),
                };
                self.clipboard = Some(content);
            }
            Action::PasteLine => {
                if let Some(content) = self.clipboard.clone() {
                    let buf = self.current_buffer_mut();
                    if buf.has_selection() {
                        buf.delete_selection();
                    }
                    match content {
                        ClipboardContent::Line(text) => buf.insert_line_below(text),
                        ClipboardContent::Fragment(text) => buf.insert_text_at_cursor(&text),
                    }
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
            Action::Help => self.open_help(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.about_open {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::F(1)) {
                self.about_open = false;
            }
            return;
        }

        if self.file_dialog.is_some() {
            self.handle_file_dialog_key(key);
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
                KeyCode::Char('e') => {
                    self.explorer.refresh();
                    self.focus = Focus::Explorer;
                }
                KeyCode::Char('l') => self.focus = Focus::Outline,
                _ => {}
            }
            // On ignore toute autre combinaison Ctrl+X pour éviter
            // d'insérer un caractère de contrôle dans le texte.
            return;
        }

        match key.code {
            KeyCode::F(2) => {
                self.explorer.refresh();
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
            PromptKind::GoToLine => match prompt.input.trim().parse::<usize>() {
                Ok(n) if n >= 1 => self.current_buffer_mut().goto_line(n - 1),
                _ => self.status_message = Some("Numéro de ligne invalide".to_string()),
            },
            PromptKind::ConfirmQuit => {}
        }
    }

    fn handle_file_dialog_key(&mut self, key: KeyEvent) {
        let editing_filename = self
            .file_dialog
            .as_ref()
            .is_some_and(|d| d.editing_filename);
        match key.code {
            KeyCode::Esc => self.file_dialog = None,
            KeyCode::Tab => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.toggle_filename_focus();
                }
            }
            KeyCode::Up | KeyCode::Char('k') if !editing_filename => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.browser.move_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') if !editing_filename => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.browser.move_down();
                }
            }
            KeyCode::Left | KeyCode::Char('h') if !editing_filename => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.browser.collapse_selected();
                }
            }
            KeyCode::Right | KeyCode::Char('l') if !editing_filename => {
                self.activate_dialog_selection();
            }
            KeyCode::Enter => self.confirm_file_dialog(),
            KeyCode::Backspace if editing_filename => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.filename.pop();
                }
            }
            KeyCode::Char(c) if editing_filename => {
                if let Some(d) = self.file_dialog.as_mut() {
                    d.filename.push(c);
                }
            }
            _ => {}
        }
    }

    /// Active l'entrée sélectionnée du dialogue : déplie/replie un dossier,
    /// ouvre un fichier (mode Ouvrir), ou pré-remplit le nom (mode
    /// Enregistrer sous).
    fn activate_dialog_selection(&mut self) {
        let Some(dialog) = self.file_dialog.as_mut() else {
            return;
        };
        let Some(path) = dialog.browser.activate_selected() else {
            // C'était un dossier : déjà déplié/replié, rien de plus à faire.
            return;
        };
        match dialog.mode {
            DialogMode::Open => {
                self.file_dialog = None;
                if let Err(e) = self.open_file(path) {
                    self.status_message = Some(format!("Erreur : {e}"));
                }
            }
            DialogMode::SaveAs => {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    dialog.filename = name.to_string();
                }
            }
        }
    }

    fn confirm_file_dialog(&mut self) {
        let Some(dialog) = self.file_dialog.as_ref() else {
            return;
        };
        if dialog.mode == DialogMode::SaveAs && dialog.editing_filename {
            if dialog.filename.trim().is_empty() {
                self.status_message = Some("Nom de fichier vide, enregistrement annulé".to_string());
                return;
            }
            let path = dialog.save_path();
            self.file_dialog = None;
            match self.current_buffer_mut().save_as(path) {
                Ok(()) => {
                    self.status_message = Some("Fichier enregistré".to_string());
                    self.explorer.refresh();
                }
                Err(e) => self.status_message = Some(format!("Erreur : {e}")),
            }
            return;
        }
        self.activate_dialog_selection();
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
        if buf.has_selection()
            && matches!(
                key.code,
                KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab | KeyCode::Backspace | KeyCode::Delete
            )
        {
            buf.delete_selection();
            if matches!(key.code, KeyCode::Backspace | KeyCode::Delete) {
                return;
            }
        }
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

    /// Point d'entrée souris, en miroir de `handle_key`. Même ordre de
    /// priorité que le clavier : À propos > dialogue fichier > invite >
    /// titre de menu cliqué > menu actif > zone cliquée.
    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        let pos = Position::new(mouse.column, mouse.row);
        let left_down = mouse.kind == MouseEventKind::Down(MouseButton::Left);

        if self.about_open {
            if left_down {
                self.about_open = false;
            }
            return;
        }

        if self.file_dialog.is_some() {
            self.handle_file_dialog_mouse(mouse, pos);
            return;
        }

        if self.prompt.is_some() {
            if left_down && !self.hitboxes.prompt.is_some_and(|r| r.contains(pos)) {
                self.prompt = None;
            }
            return;
        }

        let clicked_title = self.hitboxes.menu_titles.iter().position(|r| r.contains(pos));
        if left_down
            && let Some(idx) = clicked_title
        {
            if self.menu.active && self.menu.selected_menu == idx {
                self.menu.close();
            } else {
                self.menu.open_at(idx);
            }
            return;
        }

        if self.menu.active {
            self.handle_menu_mouse(mouse, pos);
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.handle_left_click(pos),
            MouseEventKind::Drag(MouseButton::Left) => self.handle_left_drag(pos),
            MouseEventKind::ScrollUp => self.handle_scroll(-3),
            MouseEventKind::ScrollDown => self.handle_scroll(3),
            _ => {}
        }
    }

    fn handle_menu_mouse(&mut self, mouse: MouseEvent, pos: Position) {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        let Some(dropdown) = self.hitboxes.menu_dropdown.clone() else {
            self.menu.close();
            return;
        };
        if let Some(idx) =
            dropdown.items.iter().position(|r| r.is_some_and(|r| r.contains(pos)))
        {
            self.menu.selected_item = idx;
            if let Some(action) = self.menu.selected_action() {
                self.execute_action(action);
            } else {
                self.menu.close();
            }
        } else if !dropdown.area.contains(pos) {
            self.menu.close();
        }
    }

    fn handle_left_click(&mut self, pos: Position) {
        if let Some(idx) = self.hitboxes.tabs.iter().position(|r| r.contains(pos)) {
            if idx < self.tabs.len() {
                self.active_tab = idx;
            }
            return;
        }

        if let Some(panel) = self.hitboxes.explorer
            && panel.inner.contains(pos)
        {
            self.focus = Focus::Explorer;
            let idx = panel.offset + (pos.y - panel.inner.y) as usize;
            if idx < self.explorer.entries.len() {
                self.explorer.selected = idx;
                if let Some(path) = self.explorer.activate_selected()
                    && let Err(e) = self.open_file(path)
                {
                    self.status_message = Some(format!("Erreur : {e}"));
                }
            }
            return;
        }

        if let Some(panel) = self.hitboxes.outline
            && panel.inner.contains(pos)
        {
            let idx = panel.offset + (pos.y - panel.inner.y) as usize;
            let items = self.current_outline();
            if idx < items.len() {
                let line = items[idx].line;
                self.outline_selected = idx;
                self.current_buffer_mut().goto_line(line);
                self.focus = Focus::Editor;
            } else {
                self.focus = Focus::Outline;
            }
            return;
        }

        if let Some(editor) = self.hitboxes.editor
            && editor.inner.contains(pos)
        {
            self.focus = Focus::Editor;
            let is_double = self.register_click(pos);
            let (line, col) = self.editor_position_at(editor, pos);
            let buf = self.current_buffer_mut();
            if is_double {
                buf.select_word_at(line, col);
            } else {
                buf.set_cursor_at(line, col);
                buf.start_selection();
            }
        }
    }

    fn handle_left_drag(&mut self, pos: Position) {
        let Some(editor) = self.hitboxes.editor else {
            return;
        };
        if !editor.inner.contains(pos) {
            return;
        }
        let (line, col) = self.editor_position_at(editor, pos);
        self.current_buffer_mut().set_cursor_at(line, col);
    }

    fn editor_position_at(&self, editor: EditorHitbox, pos: Position) -> (usize, usize) {
        let buf = self.current_buffer();
        let row_in_editor = (pos.y - editor.inner.y) as usize;
        let line = buf.scroll_row + row_in_editor;
        let text_x = editor.inner.x + editor.gutter_width as u16 + 1;
        let col = if pos.x >= text_x {
            buf.scroll_col + (pos.x - text_x) as usize
        } else {
            0
        };
        (line, col)
    }

    /// Détecte un double-clic : même cellule qu'un clic gauche précédent, à
    /// moins de 400ms d'écart. La détection reconnue consomme la mémorisation
    /// pour qu'un 3ᵉ clic rapproché ne soit pas à son tour pris pour un
    /// second double-clic.
    fn register_click(&mut self, pos: Position) -> bool {
        let now = Instant::now();
        let is_double = self.last_click.is_some_and(|(t, x, y)| {
            x == pos.x && y == pos.y && now.duration_since(t) < Duration::from_millis(400)
        });
        self.last_click = if is_double { None } else { Some((now, pos.x, pos.y)) };
        is_double
    }

    fn handle_scroll(&mut self, delta: isize) {
        match self.focus {
            Focus::Editor => {
                let buf = self.current_buffer_mut();
                let max_row = buf.lines.len().saturating_sub(1) as isize;
                buf.scroll_row = (buf.scroll_row as isize + delta).clamp(0, max_row) as usize;
            }
            Focus::Explorer => {
                let n = self.explorer.entries.len();
                if n > 0 {
                    self.explorer.selected =
                        (self.explorer.selected as isize + delta).clamp(0, n as isize - 1) as usize;
                }
            }
            Focus::Outline => {
                let n = self.current_outline().len();
                if n > 0 {
                    self.outline_selected =
                        (self.outline_selected as isize + delta).clamp(0, n as isize - 1) as usize;
                }
            }
        }
    }

    fn handle_file_dialog_mouse(&mut self, mouse: MouseEvent, pos: Position) {
        let Some(hitbox) = self.hitboxes.file_dialog else {
            return;
        };
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if hitbox.tree.inner.contains(pos) {
                    let idx = hitbox.tree.offset + (pos.y - hitbox.tree.inner.y) as usize;
                    if let Some(d) = self.file_dialog.as_mut() {
                        d.editing_filename = false;
                        if idx < d.browser.entries.len() {
                            d.browser.selected = idx;
                        }
                    }
                    self.activate_dialog_selection();
                } else if hitbox.filename_area.is_some_and(|r| r.contains(pos)) {
                    if let Some(d) = self.file_dialog.as_mut() {
                        d.editing_filename = true;
                    }
                } else if !hitbox.area.contains(pos) {
                    self.file_dialog = None;
                }
            }
            MouseEventKind::ScrollUp => self.scroll_dialog(-3),
            MouseEventKind::ScrollDown => self.scroll_dialog(3),
            _ => {}
        }
    }

    fn scroll_dialog(&mut self, delta: isize) {
        if let Some(d) = self.file_dialog.as_mut() {
            let n = d.browser.entries.len();
            if n > 0 {
                d.browser.selected =
                    (d.browser.selected as isize + delta).clamp(0, n as isize - 1) as usize;
            }
        }
    }
}
