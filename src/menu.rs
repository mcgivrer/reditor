#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    NewFile,
    OpenFile,
    OpenFolder,
    Save,
    SaveAs,
    CloseTab,
    Quit,
    CutLine,
    CopyLine,
    PasteLine,
    GoToLine,
    ToggleExplorer,
    ToggleOutline,
    Compile,
    ConfigureCompilation,
    About,
}

#[derive(Debug)]
pub struct MenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    /// `None` représente un séparateur (non sélectionnable).
    pub action: Option<Action>,
}

#[derive(Debug)]
pub struct MenuDef {
    pub title: &'static str,
    pub items: Vec<MenuItem>,
}

#[derive(Debug)]
pub struct MenuBar {
    pub menus: Vec<MenuDef>,
    pub active: bool,
    pub selected_menu: usize,
    pub selected_item: usize,
}

impl MenuBar {
    /// `compilable` indique si le projet ouvert propose au moins un module
    /// de compilation (voir `compile::detect_module`) : le menu « Compiler »
    /// n'est construit que dans ce cas.
    pub fn new(compilable: bool) -> Self {
        let mut menus = vec![
            MenuDef {
                title: "Fichier",
                items: vec![
                    item("Nouveau", "Ctrl+N", Action::NewFile),
                    item("Ouvrir...", "Ctrl+O", Action::OpenFile),
                    item("Ouvrir un dossier...", "Ctrl+Maj+O", Action::OpenFolder),
                    item("Enregistrer", "Ctrl+S", Action::Save),
                    item("Enregistrer sous...", "", Action::SaveAs),
                    item("Fermer l'onglet", "Ctrl+W", Action::CloseTab),
                    separator(),
                    item("Quitter", "Ctrl+Q", Action::Quit),
                ],
            },
            MenuDef {
                title: "Édition",
                items: vec![
                    item("Couper la ligne", "Ctrl+X", Action::CutLine),
                    item("Copier la ligne", "Ctrl+C", Action::CopyLine),
                    item("Coller", "Ctrl+V", Action::PasteLine),
                    separator(),
                    item("Aller à la ligne...", "Ctrl+G", Action::GoToLine),
                ],
            },
            MenuDef {
                title: "Affichage",
                items: vec![
                    item("Explorateur", "Ctrl+B", Action::ToggleExplorer),
                    item("Structure", "", Action::ToggleOutline),
                ],
            },
        ];
        if compilable {
            menus.push(MenuDef {
                title: "Compiler",
                items: vec![
                    item("Compiler le projet", "F5", Action::Compile),
                    item("Configurer les JDK...", "F6", Action::ConfigureCompilation),
                ],
            });
        }
        menus.push(MenuDef {
            title: "Aide",
            items: vec![item("À propos", "F1", Action::About)],
        });
        MenuBar {
            menus,
            active: false,
            selected_menu: 0,
            selected_item: 0,
        }
    }

    pub fn open(&mut self) {
        self.active = true;
        self.selected_menu = 0;
        self.selected_item = 0;
        self.skip_separator_forward();
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn current_items(&self) -> &[MenuItem] {
        &self.menus[self.selected_menu].items
    }

    pub fn move_left(&mut self) {
        self.selected_menu = (self.selected_menu + self.menus.len() - 1) % self.menus.len();
        self.selected_item = 0;
        self.skip_separator_forward();
    }

    pub fn move_right(&mut self) {
        self.selected_menu = (self.selected_menu + 1) % self.menus.len();
        self.selected_item = 0;
        self.skip_separator_forward();
    }

    pub fn move_up(&mut self) {
        let n = self.current_items().len();
        if n == 0 {
            return;
        }
        for _ in 0..n {
            self.selected_item = (self.selected_item + n - 1) % n;
            if self.current_items()[self.selected_item].action.is_some() {
                break;
            }
        }
    }

    pub fn move_down(&mut self) {
        let n = self.current_items().len();
        if n == 0 {
            return;
        }
        for _ in 0..n {
            self.selected_item = (self.selected_item + 1) % n;
            if self.current_items()[self.selected_item].action.is_some() {
                break;
            }
        }
    }

    pub fn selected_action(&self) -> Option<Action> {
        self.current_items().get(self.selected_item).and_then(|i| i.action)
    }

    fn skip_separator_forward(&mut self) {
        let n = self.current_items().len();
        for _ in 0..n {
            if self.current_items()[self.selected_item].action.is_some() {
                return;
            }
            self.selected_item = (self.selected_item + 1) % n.max(1);
        }
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new(false)
    }
}

fn item(label: &'static str, shortcut: &'static str, action: Action) -> MenuItem {
    MenuItem { label, shortcut, action: Some(action) }
}

fn separator() -> MenuItem {
    MenuItem { label: "", shortcut: "", action: None }
}
