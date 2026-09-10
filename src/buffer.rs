use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::syntax::{self, Language, LineHighlightState};

#[derive(Debug)]
pub struct Buffer {
    pub path: Option<PathBuf>,
    /// Nom affiché pour un onglet sans chemin de fichier réel (ex. le
    /// manuel utilisateur embarqué), à défaut de `path`.
    pub virtual_name: Option<String>,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub modified: bool,
    pub language: Language,
    pub highlight_states: Vec<LineHighlightState>,
    /// Point d'ancrage d'une sélection en cours (ligne, colonne). `None` si
    /// aucune sélection n'est active.
    pub selection_anchor: Option<(usize, usize)>,
}

impl Buffer {
    pub fn empty() -> Self {
        Buffer {
            path: None,
            virtual_name: None,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            modified: false,
            language: Language::PlainText,
            highlight_states: vec![LineHighlightState::default()],
            selection_anchor: None,
        }
    }

    pub fn from_path(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Impossible de lire {}", path.display()))?;
        let mut lines: Vec<String> = content
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
            .collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let language = syntax::detect_language(&path);
        let mut buffer = Buffer {
            path: Some(path),
            virtual_name: None,
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            modified: false,
            language,
            highlight_states: Vec::new(),
            selection_anchor: None,
        };
        buffer.recompute_highlight_states();
        Ok(buffer)
    }

    /// Construit un onglet en lecture depuis un contenu embarqué dans le
    /// binaire (ex. le manuel utilisateur), sans fichier associé sur disque.
    pub fn from_content(name: &str, content: &str, language: Language) -> Self {
        let mut lines: Vec<String> = content
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
            .collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let mut buffer = Buffer {
            path: None,
            virtual_name: Some(name.to_string()),
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            modified: false,
            language,
            highlight_states: Vec::new(),
            selection_anchor: None,
        };
        buffer.recompute_highlight_states();
        buffer
    }

    pub fn display_name(&self) -> String {
        if let Some(p) = &self.path {
            return p
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("?")
                .to_string();
        }
        if let Some(name) = &self.virtual_name {
            return name.clone();
        }
        "sans titre".to_string()
    }

    pub fn save(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .context("Ce buffer n'a pas encore de chemin de fichier")?;
        let content = self.lines.join("\n");
        fs::write(&path, content)
            .with_context(|| format!("Impossible d'écrire {}", path.display()))?;
        self.modified = false;
        Ok(())
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<()> {
        self.language = syntax::detect_language(&path);
        self.path = Some(path);
        self.recompute_highlight_states();
        self.save()
    }

    pub fn recompute_highlight_states(&mut self) {
        let mut states = Vec::with_capacity(self.lines.len());
        let mut state = LineHighlightState::default();
        for line in &self.lines {
            states.push(state);
            let _ = syntax::highlight_line(line, self.language, &mut state);
        }
        self.highlight_states = states;
    }

    pub fn line_char_len(&self, line: usize) -> usize {
        self.lines[line].chars().count()
    }

    fn current_line_char_len(&self) -> usize {
        self.line_char_len(self.cursor_line)
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_idx = char_to_byte(&self.lines[self.cursor_line], self.cursor_col);
        self.lines[self.cursor_line].insert(byte_idx, c);
        self.cursor_col += 1;
        self.modified = true;
        self.recompute_highlight_states();
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(' ');
        }
    }

    pub fn insert_newline(&mut self) {
        let byte_idx = char_to_byte(&self.lines[self.cursor_line], self.cursor_col);
        let rest = self.lines[self.cursor_line].split_off(byte_idx);
        self.lines.insert(self.cursor_line + 1, rest);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.modified = true;
        self.recompute_highlight_states();
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            let start = char_to_byte(line, self.cursor_col - 1);
            let end = char_to_byte(line, self.cursor_col);
            line.replace_range(start..end, "");
            self.cursor_col -= 1;
            self.modified = true;
            self.recompute_highlight_states();
        } else if self.cursor_line > 0 {
            let current = self.lines.remove(self.cursor_line);
            let prev_len = self.lines[self.cursor_line - 1].chars().count();
            self.lines[self.cursor_line - 1].push_str(&current);
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
            self.modified = true;
            self.recompute_highlight_states();
        }
    }

    pub fn delete_forward(&mut self) {
        let len = self.current_line_char_len();
        if self.cursor_col < len {
            let line = &mut self.lines[self.cursor_line];
            let start = char_to_byte(line, self.cursor_col);
            let end = char_to_byte(line, self.cursor_col + 1);
            line.replace_range(start..end, "");
            self.modified = true;
            self.recompute_highlight_states();
        } else if self.cursor_line + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
            self.modified = true;
            self.recompute_highlight_states();
        }
    }

    pub fn move_left(&mut self) {
        self.clear_selection();
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.current_line_char_len();
        }
    }

    pub fn move_right(&mut self) {
        self.clear_selection();
        if self.cursor_col < self.current_line_char_len() {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        self.clear_selection();
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.current_line_char_len());
        }
    }

    pub fn move_down(&mut self) {
        self.clear_selection();
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.current_line_char_len());
        }
    }

    pub fn move_home(&mut self) {
        self.clear_selection();
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.clear_selection();
        self.cursor_col = self.current_line_char_len();
    }

    pub fn move_page(&mut self, delta: isize) {
        self.clear_selection();
        let new_line = (self.cursor_line as isize + delta)
            .clamp(0, self.lines.len() as isize - 1) as usize;
        self.cursor_line = new_line;
        self.cursor_col = self.cursor_col.min(self.current_line_char_len());
    }

    pub fn goto_line(&mut self, line: usize) {
        self.clear_selection();
        self.cursor_line = line.min(self.lines.len().saturating_sub(1));
        self.cursor_col = 0;
    }

    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_line]
    }

    /// Positionne le curseur à `(line, col)`, en clampant sur les bornes du
    /// buffer et de la ligne visée (utilisé par un clic souris).
    pub fn set_cursor_at(&mut self, line: usize, col: usize) {
        self.cursor_line = line.min(self.lines.len().saturating_sub(1));
        self.cursor_col = col.min(self.line_char_len(self.cursor_line));
    }

    pub fn start_selection(&mut self) {
        self.selection_anchor = Some((self.cursor_line, self.cursor_col));
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Un point d'ancrage identique au curseur (clic simple, sans glisser)
    /// ne compte pas comme une sélection.
    pub fn has_selection(&self) -> bool {
        self.selection_anchor
            .is_some_and(|anchor| anchor != (self.cursor_line, self.cursor_col))
    }

    /// Bornes normalisées `(début, fin)` de la sélection, ou `None`.
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if !self.has_selection() {
            return None;
        }
        let anchor = self.selection_anchor?;
        let cursor = (self.cursor_line, self.cursor_col);
        Some(if anchor <= cursor { (anchor, cursor) } else { (cursor, anchor) })
    }

    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        Some(self.text_in_range(start, end))
    }

    fn text_in_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        let (start_line, start_col) = start;
        let (end_line, end_col) = end;
        if start_line == end_line {
            let line = &self.lines[start_line];
            let from = char_to_byte(line, start_col);
            let to = char_to_byte(line, end_col);
            return line[from..to].to_string();
        }
        let mut result = String::new();
        let first = &self.lines[start_line];
        result.push_str(&first[char_to_byte(first, start_col)..]);
        for line in &self.lines[start_line + 1..end_line] {
            result.push('\n');
            result.push_str(line);
        }
        result.push('\n');
        let last = &self.lines[end_line];
        result.push_str(&last[..char_to_byte(last, end_col)]);
        result
    }

    /// Supprime le texte sélectionné, replace le curseur au point de départ
    /// de la sélection et referme celle-ci.
    pub fn delete_selection(&mut self) {
        let Some((start, end)) = self.selection_range() else {
            return;
        };
        let (start_line, start_col) = start;
        let (end_line, end_col) = end;
        let first = &self.lines[start_line];
        let prefix = first[..char_to_byte(first, start_col)].to_string();
        let last = &self.lines[end_line];
        let tail = last[char_to_byte(last, end_col)..].to_string();
        self.lines.drain(start_line..=end_line);
        self.lines.insert(start_line, prefix + &tail);
        self.cursor_line = start_line;
        self.cursor_col = start_col;
        self.clear_selection();
        self.modified = true;
        self.recompute_highlight_states();
    }

    /// Sélectionne le mot alphanumérique/`_` sous `(line, col)`. Ne crée
    /// aucune sélection si le caractère visé n'appartient pas à un mot.
    pub fn select_word_at(&mut self, line: usize, col: usize) {
        self.clear_selection();
        let line = line.min(self.lines.len().saturating_sub(1));
        let chars: Vec<char> = self.lines[line].chars().collect();
        let col = col.min(chars.len());
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        // Un clic juste après le dernier caractère du mot doit encore le sélectionner.
        let probe = if col < chars.len() {
            Some(col)
        } else if col > 0 && is_word(chars[col - 1]) {
            Some(col - 1)
        } else {
            None
        };
        let Some(probe) = probe.filter(|&i| is_word(chars[i])) else {
            self.set_cursor_at(line, col);
            return;
        };
        let mut start = probe;
        while start > 0 && is_word(chars[start - 1]) {
            start -= 1;
        }
        let mut end = probe + 1;
        while end < chars.len() && is_word(chars[end]) {
            end += 1;
        }
        self.cursor_line = line;
        self.selection_anchor = Some((line, start));
        self.cursor_col = end;
    }

    /// Insère `text` (potentiellement multi-lignes) à la position du
    /// curseur, en coupant la ligne courante comme le fait `insert_newline`.
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        let byte_idx = char_to_byte(&self.lines[self.cursor_line], self.cursor_col);
        let tail = self.lines[self.cursor_line].split_off(byte_idx);
        let mut fragments = text.split('\n');
        let first = fragments.next().unwrap_or("");
        self.lines[self.cursor_line].push_str(first);
        let mut insert_at = self.cursor_line + 1;
        let mut last_fragment = first;
        for fragment in fragments {
            self.lines.insert(insert_at, fragment.to_string());
            insert_at += 1;
            last_fragment = fragment;
        }
        let last_line = insert_at - 1;
        self.cursor_col = if last_line == self.cursor_line {
            self.cursor_col + last_fragment.chars().count()
        } else {
            last_fragment.chars().count()
        };
        self.lines[last_line].push_str(&tail);
        self.cursor_line = last_line;
        self.modified = true;
        self.recompute_highlight_states();
    }

    /// Retire la ligne courante du buffer et la renvoie.
    pub fn remove_current_line(&mut self) -> String {
        let content = if self.lines.len() == 1 {
            std::mem::take(&mut self.lines[0])
        } else {
            let removed = self.lines.remove(self.cursor_line);
            if self.cursor_line >= self.lines.len() {
                self.cursor_line = self.lines.len() - 1;
            }
            removed
        };
        self.cursor_col = 0;
        self.modified = true;
        self.recompute_highlight_states();
        content
    }

    /// Insère `text` comme nouvelle ligne juste après la ligne courante.
    pub fn insert_line_below(&mut self, text: String) {
        self.lines.insert(self.cursor_line + 1, text);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.modified = true;
        self.recompute_highlight_states();
    }

    pub fn ensure_cursor_visible(&mut self, viewport_height: usize, viewport_width: usize) {
        if viewport_height == 0 || viewport_width == 0 {
            return;
        }
        if self.cursor_line < self.scroll_row {
            self.scroll_row = self.cursor_line;
        } else if self.cursor_line >= self.scroll_row + viewport_height {
            self.scroll_row = self.cursor_line + 1 - viewport_height;
        }
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + viewport_width {
            self.scroll_col = self.cursor_col + 1 - viewport_width;
        }
    }
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_newline_splits_line_at_cursor() {
        let mut b = Buffer::empty();
        b.lines[0] = "// Point d'entrée".to_string();
        for c in "// ligne ajoutee par le test".chars() {
            b.insert_char(c);
        }
        assert_eq!(b.lines[0], "// ligne ajoutee par le test// Point d'entrée");
        assert_eq!(b.cursor_col, 28);
        b.insert_newline();
        assert_eq!(b.lines[0], "// ligne ajoutee par le test");
        assert_eq!(b.lines[1], "// Point d'entrée");
        assert_eq!(b.cursor_line, 1);
        assert_eq!(b.cursor_col, 0);
    }

    fn buffer_with(lines: &[&str]) -> Buffer {
        let mut b = Buffer::empty();
        b.lines = lines.iter().map(|l| l.to_string()).collect();
        b
    }

    #[test]
    fn selection_range_is_normalized_regardless_of_drag_direction() {
        let mut b = buffer_with(&["bonjour le monde"]);
        b.set_cursor_at(0, 11);
        b.start_selection();
        b.set_cursor_at(0, 3);
        assert_eq!(b.selection_range(), Some(((0, 3), (0, 11))));
        assert_eq!(b.selected_text(), Some("jour le ".to_string()));
    }

    #[test]
    fn simple_click_without_drag_is_not_a_selection() {
        let mut b = buffer_with(&["bonjour"]);
        b.set_cursor_at(0, 2);
        b.start_selection();
        assert!(!b.has_selection());
        assert_eq!(b.selected_text(), None);
    }

    #[test]
    fn select_word_at_picks_word_boundaries() {
        let mut b = buffer_with(&["let variable_name = 1;"]);
        b.select_word_at(0, 6);
        assert_eq!(b.selected_text(), Some("variable_name".to_string()));
    }

    #[test]
    fn select_word_at_on_whitespace_creates_no_selection() {
        let mut b = buffer_with(&["a  b"]);
        b.select_word_at(0, 2);
        assert!(!b.has_selection());
    }

    #[test]
    fn delete_selection_merges_multiline_fragments() {
        let mut b = buffer_with(&["premiere ligne", "seconde ligne", "troisieme ligne"]);
        b.set_cursor_at(0, 9);
        b.start_selection();
        b.set_cursor_at(2, 6);
        assert_eq!(b.selected_text(), Some("ligne\nseconde ligne\ntroisi".to_string()));
        b.delete_selection();
        assert_eq!(b.lines, vec!["premiere eme ligne".to_string()]);
        assert_eq!(b.cursor_line, 0);
        assert_eq!(b.cursor_col, 9);
        assert!(!b.has_selection());
    }

    #[test]
    fn insert_text_at_cursor_inserts_multiline_fragment() {
        let mut b = buffer_with(&["debutfin"]);
        b.set_cursor_at(0, 5);
        b.insert_text_at_cursor("a\nb");
        assert_eq!(b.lines, vec!["debuta".to_string(), "bfin".to_string()]);
        assert_eq!(b.cursor_line, 1);
        assert_eq!(b.cursor_col, 1);
    }
}
