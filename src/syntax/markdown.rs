use super::TokenKind;

pub fn tokenize(line: &str, in_code_fence: &mut bool) -> Vec<(TokenKind, String)> {
    let trimmed = line.trim_start();

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        *in_code_fence = !*in_code_fence;
        return vec![(TokenKind::InlineCode, line.to_string())];
    }
    if *in_code_fence {
        return vec![(TokenKind::InlineCode, line.to_string())];
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count > 0 && hash_count <= 6 {
        let after = &trimmed[hash_count..];
        if after.is_empty() || after.starts_with(' ') {
            return vec![(TokenKind::Heading, line.to_string())];
        }
    }

    if trimmed.starts_with('>') {
        return vec![(TokenKind::Italic, line.to_string())];
    }

    tokenize_inline(line)
}

fn tokenize_inline(line: &str) -> Vec<(TokenKind, String)> {
    let mut tokens = Vec::new();
    let len = line.len();
    let mut pos = 0usize;

    while pos < len {
        let rest = &line[pos..];
        let first = rest.chars().next().unwrap();

        if first == '*' || first == '_' || first == '`' || first == '[' {
            if let Some(end) = try_wrapped(rest, pos, "**") {
                tokens.push((TokenKind::Bold, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            if let Some(end) = try_wrapped(rest, pos, "__") {
                tokens.push((TokenKind::Bold, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            if let Some(end) = try_wrapped(rest, pos, "`") {
                tokens.push((TokenKind::InlineCode, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            if let Some(end) = try_link(rest, pos) {
                tokens.push((TokenKind::Link, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            if let Some(end) = try_wrapped(rest, pos, "*") {
                tokens.push((TokenKind::Italic, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            if let Some(end) = try_wrapped(rest, pos, "_") {
                tokens.push((TokenKind::Italic, line[pos..end].to_string()));
                pos = end;
                continue;
            }
            tokens.push((TokenKind::Plain, first.to_string()));
            pos += first.len_utf8();
            continue;
        }

        let mut adv = 0usize;
        for c in rest.chars() {
            if "*_`[".contains(c) {
                break;
            }
            adv += c.len_utf8();
        }
        if adv == 0 {
            adv = first.len_utf8();
        }
        tokens.push((TokenKind::Plain, line[pos..pos + adv].to_string()));
        pos += adv;
    }

    tokens
}

fn try_wrapped(rest: &str, pos: usize, marker: &str) -> Option<usize> {
    if !rest.starts_with(marker) {
        return None;
    }
    let after = &rest[marker.len()..];
    if after.is_empty() {
        return None;
    }
    let rel = after.find(marker)?;
    if rel == 0 {
        return None;
    }
    Some(pos + marker.len() + rel + marker.len())
}

fn try_link(rest: &str, pos: usize) -> Option<usize> {
    if !rest.starts_with('[') {
        return None;
    }
    let close_bracket = rest.find(']')?;
    let after_bracket = &rest[close_bracket + 1..];
    if !after_bracket.starts_with('(') {
        return None;
    }
    let close_paren_rel = after_bracket.find(')')?;
    Some(pos + close_bracket + 1 + close_paren_rel + 1)
}
