use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ExplorerEntry {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

#[derive(Debug)]
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

    pub fn root(&self) -> &Path {
        &self.entries[0].path
    }

    /// Change la racine de l'explorateur et recharge l'arborescence depuis
    /// ce nouveau dossier (perd les dossiers dépliés et la sélection
    /// précédents, contrairement à `refresh`).
    pub fn set_root(&mut self, root: PathBuf) {
        *self = Explorer::new(root);
    }

    /// Remonte la racine au dossier parent du dossier actuellement affiché,
    /// même au-delà de la racine initiale de l'explorateur (utilisé par le
    /// dialogue « Ouvrir un dossier... » pour atteindre n'importe quel
    /// répertoire). Sélectionne l'ancien dossier racine dans la nouvelle
    /// arborescence pour conserver le contexte. Renvoie `false` si le
    /// dossier actuel n'a pas de parent (racine du système de fichiers).
    pub fn go_up(&mut self) -> bool {
        let current_root = self.entries[0].path.clone();
        let Some(parent) = current_root.parent().map(Path::to_path_buf) else {
            return false;
        };
        self.set_root(parent);
        if let Some(idx) = self.entries.iter().position(|e| e.path == current_root) {
            self.selected = idx;
        }
        true
    }

    /// Relit l'arborescence depuis le disque (fichier ajouté, modifié,
    /// renommé ou supprimé), en conservant autant que possible les dossiers
    /// dépliés et l'entrée sélectionnée.
    pub fn refresh(&mut self) {
        let expanded: HashSet<PathBuf> = self
            .entries
            .iter()
            .filter(|e| e.is_dir && e.expanded)
            .map(|e| e.path.clone())
            .collect();
        let selected_path = self.entries.get(self.selected).map(|e| e.path.clone());
        let root = self.entries[0].path.clone();

        self.entries = vec![ExplorerEntry {
            path: root,
            depth: 0,
            is_dir: true,
            expanded: true,
        }];
        self.load_children(0);

        let mut i = 0;
        while i < self.entries.len() {
            if self.entries[i].is_dir
                && !self.entries[i].expanded
                && expanded.contains(&self.entries[i].path)
            {
                self.entries[i].expanded = true;
                self.load_children(i);
            }
            i += 1;
        }

        self.selected = selected_path
            .and_then(|path| self.entries.iter().position(|e| e.path == path))
            .unwrap_or(0)
            .min(self.entries.len().saturating_sub(1));
    }
}
