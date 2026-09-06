use super::rules::GenericLangConfig;
use super::TokenKind;

/// Tokenizer générique pour les langages de type C (mots-clés, chaînes,
/// nombres, commentaires ligne/bloc, sigil de variable façon shell).
pub fn tokenize(
    line: &str,
    cfg: &GenericLangConfig,
    in_block_comment: &mut bool,
) -> Vec<(TokenKind, String)> {
    let mut tokens: Vec<(TokenKind, String)> = Vec::new();
    let len = line.len();
    let mut pos = 0usize;

    if *in_block_comment
        && let Some((_, close)) = cfg.block_comment {
            if let Some(rel) = line[pos..].find(close) {
                let end = pos + rel + close.len();
                tokens.push((TokenKind::Comment, line[pos..end].to_string()));
                pos = end;
                *in_block_comment = false;
            } else {
                tokens.push((TokenKind::Comment, line[pos..].to_string()));
                return tokens;
            }
        }

    while pos < len {
        let rest = &line[pos..];
        let ch = rest.chars().next().unwrap();

        // Commentaire de ligne.
        if cfg.line_comment.iter().any(|c| rest.starts_with(*c)) {
            tokens.push((TokenKind::Comment, line[pos..].to_string()));
            break;
        }

        // Commentaire de bloc.
        if let Some((open, close)) = cfg.block_comment
            && rest.starts_with(open) {
                if let Some(rel) = rest[open.len()..].find(close) {
                    let end = pos + open.len() + rel + close.len();
                    tokens.push((TokenKind::Comment, line[pos..end].to_string()));
                    pos = end;
                    continue;
                } else {
                    tokens.push((TokenKind::Comment, line[pos..].to_string()));
                    *in_block_comment = true;
                    break;
                }
            }

        // Chaîne de caractères.
        if cfg.string_quotes.contains(&ch) {
            let quote = ch;
            let content_start = pos + quote.len_utf8();
            let mut end = len;
            let mut cursor = content_start;
            let mut escaped = false;
            for c in line[content_start..].chars() {
                let clen = c.len_utf8();
                if escaped {
                    escaped = false;
                    cursor += clen;
                    continue;
                }
                if c == '\\' {
                    escaped = true;
                    cursor += clen;
                    continue;
                }
                if c == quote {
                    cursor += clen;
                    end = cursor;
                    break;
                }
                cursor += clen;
            }
            tokens.push((TokenKind::String, line[pos..end].to_string()));
            pos = end;
            continue;
        }

        // Nombre.
        if ch.is_ascii_digit() {
            let start = pos;
            let mut adv = 0usize;
            for c in rest.chars() {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
                    adv += c.len_utf8();
                } else {
                    break;
                }
            }
            let end = start + adv;
            tokens.push((TokenKind::Number, line[start..end].to_string()));
            pos = end;
            continue;
        }

        // Variable façon shell ($VAR, ${VAR}).
        if let Some(sigil) = cfg.variable_sigil
            && ch == sigil {
                let start = pos;
                let mut end = pos + ch.len_utf8();
                if line[end..].starts_with('{') {
                    if let Some(rel) = line[end..].find('}') {
                        end += rel + 1;
                    } else {
                        end = len;
                    }
                } else {
                    let mut adv = 0usize;
                    for c in line[end..].chars() {
                        if c.is_alphanumeric() || c == '_' {
                            adv += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    end += adv;
                }
                tokens.push((TokenKind::Variable, line[start..end].to_string()));
                pos = end;
                continue;
            }

        // Identifiant / mot-clé / type / appel de fonction.
        if ch.is_alphabetic() || ch == '_' {
            let start = pos;
            let mut adv = 0usize;
            for c in rest.chars() {
                if c.is_alphanumeric() || c == '_' {
                    adv += c.len_utf8();
                } else {
                    break;
                }
            }
            let end = start + adv;
            let word = &line[start..end];
            let kind = if cfg.keywords.contains(&word) {
                TokenKind::Keyword
            } else if cfg.booleans.contains(&word) {
                TokenKind::Boolean
            } else if cfg.types_or_builtins.contains(&word) {
                TokenKind::Type
            } else if line[end..].trim_start().starts_with('(') {
                TokenKind::Function
            } else {
                TokenKind::Plain
            };
            tokens.push((kind, word.to_string()));
            pos = end;
            continue;
        }

        // Attribut / annotation (#[...] en Rust, @Annotation en Java/Kotlin).
        if ch == '@' || (ch == '#' && line[pos + 1..].starts_with('[')) {
            let start = pos;
            let mut adv = ch.len_utf8();
            for c in line[start + adv..].chars() {
                if c.is_alphanumeric() || c == '_' || c == '[' || c == ']' || c == '(' || c == ')'
                {
                    adv += c.len_utf8();
                } else {
                    break;
                }
            }
            let end = start + adv;
            tokens.push((TokenKind::Attribute, line[start..end].to_string()));
            pos = end;
            continue;
        }

        // Reste : ponctuation / espace, un caractère à la fois.
        let start = pos;
        let end = pos + ch.len_utf8();
        tokens.push((TokenKind::Plain, line[start..end].to_string()));
        pos = end;
    }

    tokens
}

/// Repasse les chaînes suivies de ':' en `Key` (utile pour JSON/CSS).
pub fn recolor_keys(tokens: &mut [(TokenKind, String)]) {
    for i in 0..tokens.len() {
        if tokens[i].0 != TokenKind::String {
            continue;
        }
        let mut j = i + 1;
        while j < tokens.len() && tokens[j].1.trim().is_empty() {
            j += 1;
        }
        if let Some((kind, text)) = tokens.get(j)
            && *kind == TokenKind::Plain && text.trim_start().starts_with(':') {
                tokens[i].0 = TokenKind::Key;
            }
    }
}
