use super::TokenKind;

/// Tokenizer pour les formats clé=valeur simples : `.properties`, `.ini`,
/// `.toml` (approximatif, sans tables imbriquées ni tableaux).
pub fn tokenize(line: &str, has_sections: bool) -> Vec<(TokenKind, String)> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let mut tokens = Vec::new();
    if indent_len > 0 {
        tokens.push((TokenKind::Plain, line[..indent_len].to_string()));
    }
    if trimmed.is_empty() {
        return tokens;
    }
    if trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with('!') {
        tokens.push((TokenKind::Comment, trimmed.to_string()));
        return tokens;
    }
    if has_sections && trimmed.starts_with('[') {
        tokens.push((TokenKind::Section, trimmed.to_string()));
        return tokens;
    }
    if let Some(idx) = trimmed.find(['=', ':']) {
        let key = &trimmed[..idx];
        let sep = &trimmed[idx..idx + 1];
        let val = &trimmed[idx + 1..];
        tokens.push((TokenKind::Key, key.to_string()));
        tokens.push((TokenKind::Plain, sep.to_string()));
        if !val.is_empty() {
            tokens.push((TokenKind::String, val.to_string()));
        }
    } else {
        tokens.push((TokenKind::Plain, trimmed.to_string()));
    }
    tokens
}

/// Tokenizer approximatif pour YAML : commentaires, listes `- item`,
/// paires `clé: valeur`.
pub fn tokenize_yaml(line: &str) -> Vec<(TokenKind, String)> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let mut tokens = Vec::new();
    if indent_len > 0 {
        tokens.push((TokenKind::Plain, line[..indent_len].to_string()));
    }
    if trimmed.is_empty() {
        return tokens;
    }
    if trimmed.starts_with('#') {
        tokens.push((TokenKind::Comment, trimmed.to_string()));
        return tokens;
    }
    let mut rest = trimmed;
    if rest.starts_with("- ") || rest == "-" {
        tokens.push((TokenKind::Plain, "-".to_string()));
        rest = rest[1..].trim_start();
        let consumed = trimmed.len() - rest.len() - 1;
        if consumed > 0 {
            tokens.push((TokenKind::Plain, " ".repeat(consumed)));
        }
    }
    if rest.starts_with("---") {
        tokens.push((TokenKind::Section, rest.to_string()));
        return tokens;
    }
    if let Some(idx) = rest.find(':') {
        let next_char = rest[idx + 1..].chars().next();
        if next_char.is_none() || next_char == Some(' ') {
            let key = &rest[..idx];
            let val = &rest[idx + 1..];
            tokens.push((TokenKind::Key, key.to_string()));
            tokens.push((TokenKind::Plain, ":".to_string()));
            if !val.is_empty() {
                tokens.push((TokenKind::String, val.to_string()));
            }
            return tokens;
        }
    }
    tokens.push((TokenKind::Plain, rest.to_string()));
    tokens
}
