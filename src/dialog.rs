use std::path::PathBuf;

use crate::explorer::Explorer;

/// Ce que le dialogue de fichier doit accomplir une fois validé.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogMode {
    Open,
    SaveAs,
}

/// Dialogue modal de sélection d'un fichier dans l'arborescence,
/// utilisé pour « Ouvrir... » et « Enregistrer sous... ».
///
/// La navigation dans l'arbre réutilise directement [`Explorer`]. En mode
/// [`DialogMode::SaveAs`], un champ de saisie additionnel (`filename`)
/// permet de choisir/corriger le nom du fichier à écrire ; `Tab` bascule le
/// focus clavier entre l'arborescence et ce champ.
#[derive(Debug)]
pub struct FileDialog {
    pub mode: DialogMode,
    pub browser: Explorer,
    pub filename: String,
    pub editing_filename: bool,
}

impl FileDialog {
    pub fn new(mode: DialogMode, root: PathBuf, default_filename: String) -> Self {
        FileDialog {
            mode,
            browser: Explorer::new(root),
            filename: default_filename,
            editing_filename: false,
        }
    }

    /// Dossier « actif » d'après l'entrée sélectionnée dans l'arborescence :
    /// l'entrée elle-même si c'est un dossier, sinon son dossier parent.
    pub fn target_directory(&self) -> PathBuf {
        match self.browser.entries.get(self.browser.selected) {
            Some(entry) if entry.is_dir => entry.path.clone(),
            Some(entry) => entry
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| entry.path.clone()),
            None => self.browser.root().to_path_buf(),
        }
    }

    /// Chemin complet proposé pour un enregistrement : dossier actif +
    /// nom de fichier saisi.
    pub fn save_path(&self) -> PathBuf {
        self.target_directory().join(&self.filename)
    }

    pub fn toggle_filename_focus(&mut self) {
        if self.mode == DialogMode::SaveAs {
            self.editing_filename = !self.editing_filename;
        }
    }
}
