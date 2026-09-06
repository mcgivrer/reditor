use crate::syntax::Language;

#[derive(Clone, Debug)]
pub struct OutlineItem {
    pub title: String,
    /// Numéro de ligne (0-indexé) sur laquelle sauter.
    pub line: usize,
    pub depth: usize,
}

pub fn extract_outline(lines: &[String], lang: Language) -> Vec<OutlineItem> {
    match lang {
        Language::Rust => extract_rust(lines),
        Language::Java | Language::Kotlin => extract_java_like(lines),
        Language::JavaScript | Language::TypeScript => extract_js(lines),
        Language::Python => extract_python(lines),
        Language::Go | Language::C | Language::Cpp => extract_brace_lang(lines),
        Language::Markdown => extract_markdown(lines),
        Language::Bash => extract_bash(lines),
        Language::Css => extract_css(lines),
        Language::Html | Language::Xml => extract_html(lines),
        Language::Ini | Language::Toml => extract_sections(lines),
        Language::Properties => extract_properties(lines),
        Language::Yaml => extract_yaml(lines),
        Language::Json | Language::PlainText => Vec::new(),
    }
}

fn word_after<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let idx = line.find(keyword)?;
    let after = &line[idx + keyword.len()..];
    let after = after.trim_start();
    if after.is_empty() {
        return None;
    }
    let end = after
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(after.len());
    if end == 0 {
        None
    } else {
        Some(&after[..end])
    }
}

fn extract_rust(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let indent = (raw.len() - line.len()) / 2;
        let stripped = line.trim_start_matches("pub(crate) ").trim_start_matches("pub ");
        for (kw, label) in [
            ("fn ", "fn"),
            ("struct ", "struct"),
            ("enum ", "enum"),
            ("trait ", "trait"),
            ("impl ", "impl"),
            ("mod ", "mod"),
        ] {
            if stripped.starts_with(kw) || stripped.starts_with(&format!("async {kw}")) {
                if let Some(name) = word_after(stripped, kw) {
                    items.push(OutlineItem {
                        title: format!("{label} {name}"),
                        line: i,
                        depth: indent,
                    });
                }
                break;
            }
        }
    }
    items
}

fn extract_java_like(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let indent = (raw.len() - line.len()) / 2;
        for (kw, label) in [
            ("class ", "class"),
            ("interface ", "interface"),
            ("enum ", "enum"),
            ("object ", "object"),
            ("fun ", "fun"),
        ] {
            if let Some(idx) = line.find(kw) {
                // évite de matcher au milieu d'un identifiant (ex: "className ")
                let before_ok = idx == 0 || !line.as_bytes()[idx - 1].is_ascii_alphanumeric();
                if before_ok
                    && let Some(name) = word_after(line, kw) {
                        items.push(OutlineItem {
                            title: format!("{label} {name}"),
                            line: i,
                            depth: indent,
                        });
                        break;
                    }
            }
        }
    }
    items
}

fn extract_js(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let indent = (raw.len() - line.len()) / 2;
        if let Some(name) = word_after(line, "class ") {
            items.push(OutlineItem { title: format!("class {name}"), line: i, depth: indent });
            continue;
        }
        if let Some(name) = word_after(line, "function ") {
            items.push(OutlineItem { title: format!("function {name}"), line: i, depth: indent });
            continue;
        }
        // const foo = (...) => { ... }  /  const foo = function
        if (line.starts_with("const ") || line.starts_with("let ") || line.starts_with("export const "))
            && let Some(eq) = line.find('=')
                && (line[eq + 1..].contains("=>") || line[eq + 1..].trim_start().starts_with("function")) {
                    let name_part = &line[..eq];
                    if let Some(name) = name_part.rsplit(' ').find(|s| !s.is_empty()) {
                        items.push(OutlineItem {
                            title: format!("const {name}"),
                            line: i,
                            depth: indent,
                        });
                    }
                }
    }
    items
}

fn extract_python(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let indent = (raw.len() - line.len()) / 4;
        if let Some(name) = word_after(line, "def ") {
            items.push(OutlineItem { title: format!("def {name}"), line: i, depth: indent });
        } else if let Some(name) = word_after(line, "class ") {
            items.push(OutlineItem { title: format!("class {name}"), line: i, depth: indent });
        }
    }
    items
}

fn extract_brace_lang(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        let indent = (raw.len() - line.len()) / 2;
        for (kw, label) in [
            ("func ", "func"),
            ("struct ", "struct"),
            ("class ", "class"),
            ("interface ", "interface"),
            ("type ", "type"),
        ] {
            if line.starts_with(kw) {
                if let Some(name) = word_after(line, kw) {
                    items.push(OutlineItem {
                        title: format!("{label} {name}"),
                        line: i,
                        depth: indent,
                    });
                }
                break;
            }
        }
    }
    items
}

fn extract_markdown(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level > 0 && level <= 6 {
            let after = trimmed[level..].trim();
            if !after.is_empty() {
                items.push(OutlineItem {
                    title: after.to_string(),
                    line: i,
                    depth: level - 1,
                });
            }
        }
    }
    items
}

fn extract_bash(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        if let Some(name) = word_after(line, "function ") {
            items.push(OutlineItem { title: format!("{name}()"), line: i, depth: 0 });
            continue;
        }
        if let Some(paren) = line.find("()") {
            let name = line[..paren].trim();
            if !name.is_empty()
                && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                items.push(OutlineItem { title: format!("{name}()"), line: i, depth: 0 });
            }
        }
    }
    items
}

fn extract_css(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.ends_with('{') {
            let selector = line.trim_end_matches('{').trim();
            if !selector.is_empty() {
                items.push(OutlineItem { title: selector.to_string(), line: i, depth: 0 });
            }
        }
    }
    items
}

fn extract_html(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_start();
        if let Some(rest) = line.strip_prefix('<') {
            if rest.starts_with('/') || rest.starts_with('!') {
                continue;
            }
            let name_end = rest
                .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                .unwrap_or(rest.len());
            let tag = &rest[..name_end];
            if matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "script" | "style" | "body" | "head") {
                items.push(OutlineItem { title: format!("<{tag}>"), line: i, depth: 0 });
            } else if let Some(id_idx) = rest.find("id=") {
                let after = &rest[id_idx + 3..];
                let quote = after.chars().next();
                if let Some(q) = quote
                    && (q == '"' || q == '\'')
                    && let Some(end) = after[1..].find(q)
                {
                    items.push(OutlineItem {
                        title: format!("#{}", &after[1..1 + end]),
                        line: i,
                        depth: 1,
                    });
                }
            }
        }
    }
    items
}

fn extract_sections(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            items.push(OutlineItem { title: line.to_string(), line: i, depth: 0 });
        }
    }
    items
}

fn extract_properties(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some(idx) = line.find(['=', ':']) {
            items.push(OutlineItem { title: line[..idx].trim().to_string(), line: i, depth: 0 });
        }
    }
    items
}

fn extract_yaml(lines: &[String]) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();
        let indent = (raw.len() - trimmed.len()) / 2;
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        if let Some(idx) = trimmed.find(':') {
            let next_char = trimmed[idx + 1..].chars().next();
            if next_char.is_none() || next_char == Some(' ') {
                items.push(OutlineItem {
                    title: trimmed[..idx].to_string(),
                    line: i,
                    depth: indent,
                });
            }
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_lines(text: &str) -> Vec<String> {
        text.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn rust_outline_finds_fn_and_struct() {
        let src = to_lines("struct Point {\n}\n\nfn main() {\n}\n");
        let items = extract_outline(&src, Language::Rust);
        let titles: Vec<_> = items.iter().map(|i| i.title.as_str()).collect();
        assert!(titles.contains(&"struct Point"));
        assert!(titles.contains(&"fn main"));
    }

    #[test]
    fn java_outline_finds_class() {
        let src = to_lines("public class Main {\n    public void run() {\n    }\n}\n");
        let items = extract_outline(&src, Language::Java);
        assert!(items.iter().any(|i| i.title == "class Main"));
    }

    #[test]
    fn markdown_outline_orders_headings_by_depth() {
        let src = to_lines("# Titre\n## Sous-titre\ntexte\n");
        let items = extract_outline(&src, Language::Markdown);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].depth, 0);
        assert_eq!(items[1].depth, 1);
    }

    #[test]
    fn ini_outline_lists_sections() {
        let src = to_lines("[a]\nkey=1\n[b]\nkey=2\n");
        let items = extract_outline(&src, Language::Ini);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "[a]");
    }
}
