mod generic;
mod html;
mod inilike;
mod markdown;
mod rules;

use std::path::Path;

use ratatui::style::{Color, Modifier, Style};

/// Catégorie sémantique d'un fragment de texte, indépendante du langage.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
    Plain,
    Keyword,
    Type,
    String,
    Number,
    Comment,
    Function,
    Attribute,
    Tag,
    AttrName,
    Heading,
    Bold,
    Italic,
    InlineCode,
    Link,
    Section,
    Key,
    Variable,
    Boolean,
}

pub fn style_for(kind: TokenKind) -> Style {
    match kind {
        TokenKind::Plain => Style::default(),
        TokenKind::Keyword => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        TokenKind::Type => Style::default().fg(Color::Yellow),
        TokenKind::String => Style::default().fg(Color::Green),
        TokenKind::Number => Style::default().fg(Color::LightCyan),
        TokenKind::Comment => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        TokenKind::Function => Style::default().fg(Color::Blue),
        TokenKind::Attribute => Style::default().fg(Color::LightYellow),
        TokenKind::Tag => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        TokenKind::AttrName => Style::default().fg(Color::Cyan),
        TokenKind::Heading => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        TokenKind::Bold => Style::default().add_modifier(Modifier::BOLD),
        TokenKind::Italic => Style::default().add_modifier(Modifier::ITALIC),
        TokenKind::InlineCode => Style::default().fg(Color::Green),
        TokenKind::Link => Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::UNDERLINED),
        TokenKind::Section => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        TokenKind::Key => Style::default().fg(Color::Cyan),
        TokenKind::Variable => Style::default().fg(Color::LightYellow),
        TokenKind::Boolean => Style::default().fg(Color::LightMagenta),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Java,
    Kotlin,
    JavaScript,
    TypeScript,
    Bash,
    Markdown,
    Html,
    Css,
    Properties,
    Ini,
    Json,
    Toml,
    Yaml,
    Python,
    C,
    Cpp,
    Go,
    Xml,
    PlainText,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Bash => "Bash",
            Language::Markdown => "Markdown",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Properties => "Properties",
            Language::Ini => "INI",
            Language::Json => "JSON",
            Language::Toml => "TOML",
            Language::Yaml => "YAML",
            Language::Python => "Python",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Xml => "XML",
            Language::PlainText => "Texte",
        }
    }
}

pub fn detect_language(path: &Path) -> Language {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "rs" => Language::Rust,
        "java" => Language::Java,
        "kt" | "kts" => Language::Kotlin,
        "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
        "ts" | "tsx" => Language::TypeScript,
        "sh" | "bash" | "zsh" => Language::Bash,
        "md" | "markdown" => Language::Markdown,
        "html" | "htm" => Language::Html,
        "css" => Language::Css,
        "properties" => Language::Properties,
        "ini" | "cfg" | "conf" => Language::Ini,
        "json" => Language::Json,
        "toml" => Language::Toml,
        "yml" | "yaml" => Language::Yaml,
        "py" | "pyw" => Language::Python,
        "c" | "h" => Language::C,
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Language::Cpp,
        "go" => Language::Go,
        "xml" | "xsd" | "svg" => Language::Xml,
        _ => match path.file_name().and_then(|f| f.to_str()).unwrap_or("") {
            "Makefile" | "makefile" | "GNUmakefile" => Language::Bash,
            ".bashrc" | ".zshrc" | ".profile" | ".bash_profile" => Language::Bash,
            "Dockerfile" => Language::Bash,
            _ => Language::PlainText,
        },
    }
}

/// État de coloration qui doit survivre d'une ligne à l'autre
/// (commentaires multi-lignes, blocs de code markdown, etc).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct LineHighlightState {
    pub in_block_comment: bool,
    pub in_html_comment: bool,
    pub in_code_fence: bool,
}

/// Découpe une ligne en fragments (kind, texte) selon le langage donné.
/// `state` est mis à jour en place pour refléter l'état en fin de ligne.
pub fn highlight_line(
    line: &str,
    lang: Language,
    state: &mut LineHighlightState,
) -> Vec<(TokenKind, String)> {
    match lang {
        Language::Rust => generic::tokenize(line, &rules::rust_config(), &mut state.in_block_comment),
        Language::Java => generic::tokenize(line, &rules::java_config(), &mut state.in_block_comment),
        Language::Kotlin => {
            generic::tokenize(line, &rules::kotlin_config(), &mut state.in_block_comment)
        }
        Language::JavaScript | Language::TypeScript => {
            generic::tokenize(line, &rules::javascript_config(), &mut state.in_block_comment)
        }
        Language::C => generic::tokenize(line, &rules::c_config(), &mut state.in_block_comment),
        Language::Cpp => generic::tokenize(line, &rules::cpp_config(), &mut state.in_block_comment),
        Language::Go => generic::tokenize(line, &rules::go_config(), &mut state.in_block_comment),
        Language::Css => {
            let mut tokens =
                generic::tokenize(line, &rules::css_config(), &mut state.in_block_comment);
            generic::recolor_keys(&mut tokens);
            tokens
        }
        Language::Json => {
            let mut tokens =
                generic::tokenize(line, &rules::json_config(), &mut state.in_block_comment);
            generic::recolor_keys(&mut tokens);
            tokens
        }
        Language::Bash => generic::tokenize(line, &rules::bash_config(), &mut state.in_block_comment),
        Language::Python => {
            generic::tokenize(line, &rules::python_config(), &mut state.in_block_comment)
        }
        Language::Markdown => markdown::tokenize(line, &mut state.in_code_fence),
        Language::Html | Language::Xml => html::tokenize(line, &mut state.in_html_comment),
        Language::Properties => inilike::tokenize(line, false),
        Language::Ini | Language::Toml => inilike::tokenize(line, true),
        Language::Yaml => inilike::tokenize_yaml(line),
        Language::PlainText => vec![(TokenKind::Plain, line.to_string())],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_languages_by_extension() {
        assert_eq!(detect_language(Path::new("Main.java")), Language::Java);
        assert_eq!(detect_language(Path::new("app.kt")), Language::Kotlin);
        assert_eq!(detect_language(Path::new("lib.rs")), Language::Rust);
        assert_eq!(detect_language(Path::new("index.html")), Language::Html);
        assert_eq!(detect_language(Path::new("style.css")), Language::Css);
        assert_eq!(detect_language(Path::new("script.js")), Language::JavaScript);
        assert_eq!(detect_language(Path::new("README.md")), Language::Markdown);
        assert_eq!(detect_language(Path::new("deploy.sh")), Language::Bash);
        assert_eq!(
            detect_language(Path::new("app.properties")),
            Language::Properties
        );
        assert_eq!(detect_language(Path::new("config.ini")), Language::Ini);
        assert_eq!(detect_language(Path::new("Makefile")), Language::Bash);
        assert_eq!(detect_language(Path::new("notes.txt")), Language::PlainText);
    }

    fn kinds(line: &str, lang: Language) -> Vec<TokenKind> {
        let mut state = LineHighlightState::default();
        highlight_line(line, lang, &mut state)
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    #[test]
    fn rust_highlights_keywords_strings_and_comments() {
        let kinds = kinds(r#"fn main() { let s = "hi"; } // done"#, Language::Rust);
        assert!(kinds.contains(&TokenKind::Keyword));
        assert!(kinds.contains(&TokenKind::String));
        assert!(kinds.contains(&TokenKind::Comment));
        assert!(kinds.contains(&TokenKind::Function));
    }

    #[test]
    fn rust_block_comment_spans_lines() {
        let mut state = LineHighlightState::default();
        let first = highlight_line("/* début", Language::Rust, &mut state);
        assert!(state.in_block_comment);
        assert!(first.iter().all(|(k, _)| *k == TokenKind::Comment));
        let second = highlight_line("toujours en commentaire */ let x = 1;", Language::Rust, &mut state);
        assert!(!state.in_block_comment);
        assert!(second.iter().any(|(k, _)| *k == TokenKind::Keyword));
    }

    #[test]
    fn bash_highlights_variables() {
        let kinds = kinds("echo $HOME/bin # comment", Language::Bash);
        assert!(kinds.contains(&TokenKind::Variable));
        assert!(kinds.contains(&TokenKind::Comment));
    }

    #[test]
    fn markdown_detects_headings_and_bold() {
        let heading = kinds("# Titre", Language::Markdown);
        assert_eq!(heading, vec![TokenKind::Heading]);
        let bold = kinds("texte **gras** normal", Language::Markdown);
        assert!(bold.contains(&TokenKind::Bold));
    }

    #[test]
    fn ini_detects_sections_and_keys() {
        let section = kinds("[server]", Language::Ini);
        assert_eq!(section, vec![TokenKind::Section]);
        let kv = kinds("port=8080", Language::Ini);
        assert!(kv.contains(&TokenKind::Key));
    }

    #[test]
    fn html_tags_and_attributes() {
        let kinds = kinds(r#"<div id="app" class="main"></div>"#, Language::Html);
        assert!(kinds.contains(&TokenKind::Tag));
        assert!(kinds.contains(&TokenKind::AttrName));
        assert!(kinds.contains(&TokenKind::String));
    }
}
