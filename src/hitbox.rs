use ratatui::layout::Rect;

/// Géométrie d'une liste défilante (explorateur, structure, arbre du
/// dialogue fichier) : zone intérieure du bloc et offset de défilement
/// courant de la `List`, pour retrouver l'index cliqué.
#[derive(Debug, Default, Clone, Copy)]
pub struct PanelHitbox {
    pub inner: Rect,
    pub offset: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EditorHitbox {
    pub inner: Rect,
    pub gutter_width: usize,
}

/// `items[i] == None` marque un séparateur (non cliquable).
#[derive(Debug, Default, Clone)]
pub struct DropdownHitbox {
    pub area: Rect,
    pub items: Vec<Option<Rect>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FileDialogHitbox {
    pub area: Rect,
    pub tree: PanelHitbox,
    pub filename_area: Option<Rect>,
}

/// Zones cliquables recalculées à chaque rendu par `ui::draw`, pour que
/// `App::handle_mouse` puisse déterminer sur quel composant/ligne un clic
/// est tombé sans dupliquer la logique de layout.
#[derive(Debug, Default, Clone)]
pub struct Hitboxes {
    pub menu_titles: Vec<Rect>,
    pub menu_dropdown: Option<DropdownHitbox>,
    pub tabs: Vec<Rect>,
    pub explorer: Option<PanelHitbox>,
    pub outline: Option<PanelHitbox>,
    pub editor: Option<EditorHitbox>,
    pub file_dialog: Option<FileDialogHitbox>,
    pub prompt: Option<Rect>,
    pub about: Option<Rect>,
}
