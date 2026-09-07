mod app;
mod buffer;
mod explorer;
mod menu;
mod outline;
mod syntax;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

/// reditor — un éditeur de texte façon IDE, dans le terminal.
#[derive(Parser)]
#[command(name = "reditor", version, about = "Un éditeur de texte façon IDE dans le terminal")]
struct Cli {
    /// Fichier ou dossier à ouvrir au démarrage
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // `directory_opened` distingue un dossier explicitement demandé (l'explorateur
    // s'affiche alors par défaut) d'un simple dossier de secours (fichier isolé ou
    // aucun argument), auquel cas l'explorateur démarre masqué.
    let (root, initial_file, directory_opened) = match cli.path {
        Some(p) if p.is_dir() => (p, None, true),
        Some(p) if p.is_file() => {
            let root = p
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            (root, Some(p), false)
        }
        Some(p) => {
            let root = p
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            (root, None, false)
        }
        None => (std::env::current_dir()?, None, false),
    };

    let mut app = App::new(root, initial_file, directory_opened)?;

    install_panic_hook();
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
