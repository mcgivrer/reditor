use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

pub struct ExplorerEntry {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

pub struct Explorer {
    pub entries: Vec<ExplorerEntry>,
    pub selected: usize,
}

impl Explorer {
    pub fn new(root: PathBuf) -> Self {
        let mut explorer = Explorer {
            entries: vec![ExplorerEntry {
                path: root,
                depth: 0,
                is_dir: true,
                expanded: true,
            }],
            selected: 0,
        };
        explorer.load_children(0);
        explorer
    }

    fn load_children(&mut self, parent_idx: usize) {
        let depth = self.entries[parent_idx].depth + 1;
        let path = self.entries[parent_idx].path.clone();
        let mut children: Vec<(PathBuf, bool)> = match fs::read_dir(&path) {
            Ok(read_dir) => read_dir
                .filter_map(|entry| entry.ok())
                .map(|entry| {
                    let p = entry.path();
                    let is_dir = p.is_dir();
                    (p, is_dir)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        children.sort_by(|a, b| match (a.1, b.1) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.0.file_name().cmp(&b.0.file_name()),
        });
        let mut insert_at = parent_idx + 1;
        for (path, is_dir) in children {
            self.entries.insert(
                insert_at,
                ExplorerEntry {
                    path,
                    depth,
                    is_dir,
                    expanded: false,
                },
            );
            insert_at += 1;
        }
    }

    fn collapse(&mut self, idx: usize) {
        let depth = self.entries[idx].depth;
        let mut end = idx + 1;
        while end < self.entries.len() && self.entries[end].depth > depth {
            end += 1;
        }
        self.entries.drain(idx + 1..end);
        self.entries[idx].expanded = false;
    }

    /// Ouvre/ferme un dossier ou renvoie le chemin d'un fichier à ouvrir.
    pub fn activate_selected(&mut self) -> Option<PathBuf> {
        let idx = self.selected;
        if idx >= self.entries.len() {
            return None;
        }
        if self.entries[idx].is_dir {
            if self.entries[idx].expanded {
                self.collapse(idx);
            } else {
                self.entries[idx].expanded = true;
                self.load_children(idx);
            }
            None
        } else {
            Some(self.entries[idx].path.clone())
        }
    }

    pub fn collapse_selected(&mut self) {
        let idx = self.selected;
        if idx < self.entries.len() && self.entries[idx].is_dir && self.entries[idx].expanded {
            self.collapse(idx);
        } else if self.entries[idx].depth > 0 {
            // Remonte au dossier parent.
            let depth = self.entries[idx].depth;
            let mut i = idx;
            while i > 0 {
                i -= 1;
                if self.entries[i].depth < depth {
                    self.selected = i;
                    break;
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }
}
