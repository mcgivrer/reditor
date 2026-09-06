use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::syntax::{self, Language, LineHighlightState};

pub struct Buffer {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub modified: bool,
    pub language: Language,
    pub highlight_states: Vec<LineHighlightState>,
}

impl Buffer {
    pub fn empty() -> Self {
        Buffer {
            path: None,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            modified: false,
            language: Language::PlainText,
            highlight_states: vec![LineHighlightState::default()],
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
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            modified: false,
            language,
            highlight_states: Vec::new(),
        };
        buffer.recompute_highlight_states();
        Ok(buffer)
    }

    pub fn display_name(&self) -> String {
        match &self.path {
            Some(p) => p
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("?")
                .to_string(),
            None => "sans titre".to_string(),
        }
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

    pub fn recompute_highlight_states(&mut self) {
        let mut states = Vec::with_capacity(self.lines.len());
        let mut state = LineHighlightState::default();
        for line in &self.lines {
            states.push(state);
            let _ = syntax::highlight_line(line, self.language, &mut state);
        }
        self.highlight_states = states;
    }

    fn current_line_char_len(&self) -> usize {
        self.lines[self.cursor_line].chars().count()
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
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.current_line_char_len();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_col < self.current_line_char_len() {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.current_line_char_len());
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.current_line_char_len());
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.current_line_char_len();
    }

    pub fn move_page(&mut self, delta: isize) {
        let new_line = (self.cursor_line as isize + delta)
            .clamp(0, self.lines.len() as isize - 1) as usize;
        self.cursor_line = new_line;
        self.cursor_col = self.cursor_col.min(self.current_line_char_len());
    }

    pub fn goto_line(&mut self, line: usize) {
        self.cursor_line = line.min(self.lines.len().saturating_sub(1));
        self.cursor_col = 0;
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
}
