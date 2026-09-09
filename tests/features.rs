//! Tests de comportement (BDD) pilotés par les fichiers Gherkin de `docs/features`.
//!
//! Les étapes manipulent directement une instance de `reditor::app::App`
//! (sans terminal réel), ce qui rend les scénarios rapides et déterministes.

use std::fs;
use std::path::PathBuf;

use cucumber::{given, then, when, World};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::TempDir;

use reditor::app::App;
use reditor::dialog::DialogMode;

#[derive(Debug, World)]
#[world(init = Self::fresh)]
pub struct ReditorWorld {
    app: App,
    workdir: TempDir,
}

impl ReditorWorld {
    fn fresh() -> Self {
        let workdir = tempfile::tempdir().expect("création du dossier temporaire");
        let app = App::new(workdir.path().to_path_buf(), None, false)
            .expect("création de l'application");
        ReditorWorld { app, workdir }
    }

    fn path_for(&self, name: &str) -> PathBuf {
        self.workdir.path().join(name)
    }
}

/// Traduit une combinaison textuelle ("Ctrl+S", "F10", "Entrée", "a", ...)
/// en événement clavier crossterm.
fn parse_key(combo: &str) -> KeyEvent {
    let mut modifiers = KeyModifiers::NONE;
    let mut rest = combo;
    loop {
        if let Some(r) = rest.strip_prefix("Ctrl+") {
            modifiers |= KeyModifiers::CONTROL;
            rest = r;
        } else if let Some(r) = rest.strip_prefix("Alt+") {
            modifiers |= KeyModifiers::ALT;
            rest = r;
        } else if let Some(r) = rest.strip_prefix("Maj+") {
            modifiers |= KeyModifiers::SHIFT;
            rest = r;
        } else {
            break;
        }
    }
    let code = match rest {
        "Entrée" | "Enter" => KeyCode::Enter,
        "Échap" | "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Bas" | "Down" => KeyCode::Down,
        "Haut" | "Up" => KeyCode::Up,
        "Gauche" | "Left" => KeyCode::Left,
        "Droite" | "Right" => KeyCode::Right,
        "Suppr" | "Delete" => KeyCode::Delete,
        "RetourArriere" | "Backspace" => KeyCode::Backspace,
        "Origine" | "Home" => KeyCode::Home,
        "Fin" | "End" => KeyCode::End,
        "F1" => KeyCode::F(1),
        "F2" => KeyCode::F(2),
        "F3" => KeyCode::F(3),
        "F4" => KeyCode::F(4),
        "F5" => KeyCode::F(5),
        "F6" => KeyCode::F(6),
        "F10" => KeyCode::F(10),
        other if other.chars().count() == 1 => {
            let c = other.chars().next().unwrap();
            // Un vrai terminal envoie la lettre en minuscule pour Ctrl+<lettre>,
            // quelle que soit la casse écrite dans le scénario ("Ctrl+S" ou "Ctrl+s").
            let c = if modifiers.contains(KeyModifiers::CONTROL) {
                c.to_ascii_lowercase()
            } else {
                c
            };
            KeyCode::Char(c)
        }
        other => panic!("combinaison de touches inconnue : {other:?}"),
    };
    KeyEvent::new(code, modifiers)
}

fn unescape(text: &str) -> String {
    text.replace("\\n", "\n")
}

// ---------------------------------------------------------------------
// Étapes génériques : fichiers, ouverture, frappe au clavier
// ---------------------------------------------------------------------

#[given(regex = r#"^un nouvel onglet vide$"#)]
fn given_new_empty_tab(world: &mut ReditorWorld) {
    // `ReditorWorld::fresh()` démarre déjà avec un unique onglet "sans titre"
    // vide et non modifié : on se contente de vérifier cette précondition
    // plutôt que d'en créer un second.
    assert_eq!(world.app.tabs.len(), 1);
    assert!(!world.app.current_buffer().modified);
}

#[given(regex = r#"^un fichier "([^"]+)" contenant "([^"]*)"$"#)]
fn given_file_with_content(world: &mut ReditorWorld, name: String, content: String) {
    let path = world.path_for(&name);
    fs::write(path, unescape(&content)).expect("écriture du fichier de test");
}

#[given(regex = r#"^un fichier nommé "([^"]+)"$"#)]
fn given_named_file(world: &mut ReditorWorld, name: String) {
    let path = world.path_for(&name);
    fs::write(&path, "").expect("écriture du fichier de test");
    world.app.open_file(path).expect("ouverture du fichier");
}

#[when(regex = r#"^j'ouvre le fichier "([^"]+)"$"#)]
fn when_open_file(world: &mut ReditorWorld, name: String) {
    let path = world.path_for(&name);
    world.app.open_file(path).expect("ouverture du fichier");
}

#[when(regex = r#"^je tape "([^"]*)"$"#)]
fn when_type_text(world: &mut ReditorWorld, text: String) {
    for c in unescape(&text).chars() {
        if c == '\n' {
            world.app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        } else {
            world
                .app
                .handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }
}

#[when(regex = r#"^j'appuie sur "([^"]+)"$"#)]
fn when_press_key(world: &mut ReditorWorld, combo: String) {
    world.app.handle_key(parse_key(&combo));
}

#[when(regex = r#"^je valide l'invite avec "([^"]*)"$"#)]
fn when_confirm_prompt(world: &mut ReditorWorld, input: String) {
    for c in input.chars() {
        world
            .app
            .handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    world.app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

// ---------------------------------------------------------------------
// Dialogue de sélection de fichier (Ouvrir / Enregistrer sous)
// ---------------------------------------------------------------------

#[then(regex = r#"^une fenêtre de dialogue "([^"]+)" est affichée$"#)]
fn then_dialog_shown(world: &mut ReditorWorld, mode_label: String) {
    let dialog = world
        .app
        .file_dialog
        .as_ref()
        .expect("aucune fenêtre de dialogue affichée");
    let expected = match mode_label.as_str() {
        "Ouvrir" => DialogMode::Open,
        "Enregistrer sous" => DialogMode::SaveAs,
        other => panic!("mode de dialogue inconnu : {other}"),
    };
    assert_eq!(dialog.mode, expected);
}

#[then("aucune fenêtre de dialogue n'est affichée")]
fn then_no_dialog(world: &mut ReditorWorld) {
    assert!(world.app.file_dialog.is_none());
}

/// Descend dans l'arborescence du dialogue jusqu'à sélectionner l'entrée
/// nommée `name`, puis l'active (ouvre le fichier, ou déplie le dossier).
#[when(regex = r#"^je choisis "([^"]+)" dans le dialogue de fichier$"#)]
fn when_pick_in_dialog(world: &mut ReditorWorld, name: String) {
    let max = world
        .app
        .file_dialog
        .as_ref()
        .map(|d| d.browser.entries.len())
        .unwrap_or(0);
    for _ in 0..max {
        let current = world.app.file_dialog.as_ref().and_then(|d| {
            d.browser
                .entries
                .get(d.browser.selected)
                .and_then(|e| e.path.file_name())
                .and_then(|n| n.to_str())
                .map(str::to_string)
        });
        if current.as_deref() == Some(name.as_str()) {
            break;
        }
        world
            .app
            .handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }
    world.app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

#[when(regex = r#"^je saisis le nom de fichier "([^"]*)" dans le dialogue$"#)]
fn when_type_dialog_filename(world: &mut ReditorWorld, name: String) {
    world
        .app
        .handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    if let Some(d) = world.app.file_dialog.as_mut() {
        d.filename.clear();
    }
    for c in name.chars() {
        world
            .app
            .handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
}

#[when("je confirme le dialogue de fichier")]
fn when_confirm_file_dialog(world: &mut ReditorWorld) {
    world.app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

// ---------------------------------------------------------------------
// Vérifications : contenu de l'éditeur, fichiers sur disque, statut
// ---------------------------------------------------------------------

#[then(regex = r#"^la ligne (\d+) de l'éditeur contient "([^"]*)"$"#)]
fn then_line_contains(world: &mut ReditorWorld, line: usize, expected: String) {
    let buf = world.app.current_buffer();
    assert_eq!(
        buf.lines.get(line - 1).map(String::as_str),
        Some(expected.as_str())
    );
}

#[then(regex = r#"^le fichier "([^"]+)" contient sur le disque "([^"]*)"$"#)]
fn then_file_on_disk(world: &mut ReditorWorld, name: String, expected: String) {
    let path = world.path_for(&name);
    let actual = fs::read_to_string(path).expect("lecture du fichier");
    assert_eq!(actual, unescape(&expected));
}

#[then(regex = r#"^le fichier "([^"]+)" n'existe pas sur le disque$"#)]
fn then_file_absent(world: &mut ReditorWorld, name: String) {
    assert!(!world.path_for(&name).exists());
}

#[then("l'onglet n'est plus marqué comme modifié")]
fn then_tab_not_modified(world: &mut ReditorWorld) {
    assert!(!world.app.current_buffer().modified);
}

#[then("l'onglet est marqué comme modifié")]
fn then_tab_modified(world: &mut ReditorWorld) {
    assert!(world.app.current_buffer().modified);
}

#[then(regex = r#"^le nombre d'onglets ouverts est (\d+)$"#)]
fn then_tab_count(world: &mut ReditorWorld, count: usize) {
    assert_eq!(world.app.tabs.len(), count);
}

#[then(regex = r#"^le nom de l'onglet actif est "([^"]+)"$"#)]
fn then_active_tab_name(world: &mut ReditorWorld, name: String) {
    assert_eq!(world.app.current_buffer().display_name(), name);
}

#[then("l'application doit se terminer")]
fn then_should_quit(world: &mut ReditorWorld) {
    assert!(world.app.should_quit);
}

#[then("l'application ne doit pas se terminer")]
fn then_should_not_quit(world: &mut ReditorWorld) {
    assert!(!world.app.should_quit);
}

// ---------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------

#[then(regex = r#"^le menu "([^"]+)" est actif$"#)]
fn then_menu_active(world: &mut ReditorWorld, title: String) {
    assert!(world.app.menu.active, "le menu devrait être ouvert");
    assert_eq!(world.app.menu.menus[world.app.menu.selected_menu].title, title);
}

#[then("le menu n'est plus actif")]
fn then_menu_inactive(world: &mut ReditorWorld) {
    assert!(!world.app.menu.active);
}

#[then(regex = r#"^une invite "([^"]+)" est affichée$"#)]
fn then_prompt_shown(world: &mut ReditorWorld, label: String) {
    let prompt = world.app.prompt.as_ref().expect("aucune invite affichée");
    assert_eq!(prompt.label, label);
}

#[then("aucune invite n'est affichée")]
fn then_no_prompt(world: &mut ReditorWorld) {
    assert!(world.app.prompt.is_none());
}

// ---------------------------------------------------------------------
// Affichage des panneaux
// ---------------------------------------------------------------------

#[given(regex = r#"^aucun dossier n'a été ouvert explicitement$"#)]
fn given_no_explicit_directory(world: &mut ReditorWorld) {
    let path = world.path_for("solo.txt");
    fs::write(&path, "contenu").expect("écriture du fichier de test");
    world.app =
        App::new(world.workdir.path().to_path_buf(), Some(path), false).expect("création de l'app");
}

#[given(regex = r#"^un dossier a été ouvert explicitement$"#)]
fn given_explicit_directory(world: &mut ReditorWorld) {
    world.app = App::new(world.workdir.path().to_path_buf(), None, true)
        .expect("création de l'app");
}

#[then(regex = r#"^le panneau "([^"]+)" est (visible|caché)$"#)]
fn then_panel_visibility(world: &mut ReditorWorld, panel: String, state: String) {
    let visible = state == "visible";
    match panel.as_str() {
        "Explorateur" => assert_eq!(world.app.show_explorer, visible),
        "Structure" => assert_eq!(world.app.show_outline, visible),
        other => panic!("panneau inconnu : {other}"),
    }
}

// ---------------------------------------------------------------------
// Rafraîchissement de l'explorateur
// ---------------------------------------------------------------------

#[when(regex = r#"^j'ajoute le fichier "([^"]+)" directement sur le disque$"#)]
fn when_add_file_on_disk(world: &mut ReditorWorld, name: String) {
    fs::write(world.path_for(&name), "contenu ajouté en externe")
        .expect("écriture du fichier de test");
}

#[when(regex = r#"^je supprime le fichier "([^"]+)" directement sur le disque$"#)]
fn when_remove_file_on_disk(world: &mut ReditorWorld, name: String) {
    fs::remove_file(world.path_for(&name)).expect("suppression du fichier de test");
}

#[when(regex = r#"^je renomme le fichier "([^"]+)" en "([^"]+)" directement sur le disque$"#)]
fn when_rename_file_on_disk(world: &mut ReditorWorld, from: String, to: String) {
    fs::rename(world.path_for(&from), world.path_for(&to)).expect("renommage du fichier de test");
}

#[when("je rafraîchis l'explorateur")]
fn when_refresh_explorer(world: &mut ReditorWorld) {
    world.app.explorer.refresh();
}

#[then(regex = r#"^l'explorateur contient l'entrée "([^"]+)"$"#)]
fn then_explorer_contains(world: &mut ReditorWorld, name: String) {
    let found = world.app.explorer.entries.iter().any(|e| {
        e.path.file_name().and_then(|n| n.to_str()) == Some(name.as_str())
    });
    assert!(found, "entrée {name:?} absente de l'explorateur");
}

#[then(regex = r#"^l'explorateur ne contient pas l'entrée "([^"]+)"$"#)]
fn then_explorer_not_contains(world: &mut ReditorWorld, name: String) {
    let found = world.app.explorer.entries.iter().any(|e| {
        e.path.file_name().and_then(|n| n.to_str()) == Some(name.as_str())
    });
    assert!(!found, "entrée {name:?} présente alors qu'elle ne devrait pas l'être");
}

// ---------------------------------------------------------------------
// Coloration syntaxique / structure
// ---------------------------------------------------------------------

#[then(regex = r#"^le langage détecté est "([^"]+)"$"#)]
fn then_language_is(world: &mut ReditorWorld, label: String) {
    assert_eq!(world.app.current_buffer().language.label(), label);
}

#[then(regex = r#"^la structure du fichier contient "([^"]+)"$"#)]
fn then_outline_contains(world: &mut ReditorWorld, title: String) {
    let items = world.app.current_outline();
    assert!(
        items.iter().any(|item| item.title == title),
        "symbole {title:?} absent de la structure : {:?}",
        items.iter().map(|i| &i.title).collect::<Vec<_>>()
    );
}

#[then(regex = r#"^la structure du fichier ne contient pas "([^"]+)"$"#)]
fn then_outline_not_contains(world: &mut ReditorWorld, title: String) {
    let items = world.app.current_outline();
    assert!(!items.iter().any(|item| item.title == title));
}

// ---------------------------------------------------------------------
// Compilation
// ---------------------------------------------------------------------

#[then(regex = r#"^le menu "([^"]+)" est présent$"#)]
fn then_menu_present(world: &mut ReditorWorld, title: String) {
    assert!(
        world.app.menu.menus.iter().any(|m| m.title == title),
        "menu {title:?} absent"
    );
}

#[then(regex = r#"^le menu "([^"]+)" est absent$"#)]
fn then_menu_absent(world: &mut ReditorWorld, title: String) {
    assert!(
        !world.app.menu.menus.iter().any(|m| m.title == title),
        "menu {title:?} présent alors qu'il ne devrait pas l'être"
    );
}

#[then("une fenêtre de configuration de compilation est affichée")]
fn then_compile_dialog_shown(world: &mut ReditorWorld) {
    assert!(world.app.compile_dialog.is_some(), "aucune fenêtre de configuration affichée");
}

#[then("aucune fenêtre de configuration de compilation n'est affichée")]
fn then_no_compile_dialog(world: &mut ReditorWorld) {
    assert!(world.app.compile_dialog.is_none());
}

#[when("je sélectionne le premier JDK détecté dans le dialogue de compilation")]
fn when_select_first_jdk(world: &mut ReditorWorld) {
    world.app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

#[then(regex = r#"^le message de statut contient "([^"]+)"$"#)]
fn then_status_message_contains(world: &mut ReditorWorld, expected: String) {
    let message = world.app.status_message.clone().unwrap_or_default();
    assert!(
        message.contains(&expected),
        "le message de statut {message:?} ne contient pas {expected:?}"
    );
}

#[then(regex = r#"^une fenêtre de résultat de compilation "(réussie|échouée)" est affichée$"#)]
fn then_compile_result_shown(world: &mut ReditorWorld, expected: String) {
    let outcome = world
        .app
        .compile_result
        .as_ref()
        .expect("aucune fenêtre de résultat de compilation affichée");
    assert_eq!(outcome.success, expected == "réussie");
}

#[tokio::main]
async fn main() {
    ReditorWorld::cucumber()
        .run("docs/features")
        .await;
}
