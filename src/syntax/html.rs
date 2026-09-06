use super::TokenKind;

pub fn tokenize(line: &str, in_comment: &mut bool) -> Vec<(TokenKind, String)> {
    let mut tokens = Vec::new();
    let len = line.len();
    let mut pos = 0usize;

    if *in_comment {
        if let Some(rel) = line.find("-->") {
            let end = rel + 3;
            tokens.push((TokenKind::Comment, line[..end].to_string()));
            pos = end;
            *in_comment = false;
        } else {
            tokens.push((TokenKind::Comment, line.to_string()));
            return tokens;
        }
    }

    while pos < len {
        let rest = &line[pos..];

        if let Some(after_open) = rest.strip_prefix("<!--") {
            if let Some(rel) = after_open.find("-->") {
                let end = pos + 4 + rel + 3;
                tokens.push((TokenKind::Comment, line[pos..end].to_string()));
                pos = end;
                continue;
            } else {
                tokens.push((TokenKind::Comment, line[pos..].to_string()));
                *in_comment = true;
                break;
            }
        }

        if rest.starts_with('<') {
            if let Some(rel) = rest.find('>') {
                let end = pos + rel + 1;
                tokens.extend(tokenize_tag(&line[pos..end]));
                pos = end;
                continue;
            } else {
                tokens.push((TokenKind::Tag, line[pos..].to_string()));
                break;
            }
        }

        let mut adv = 0usize;
        for c in rest.chars() {
            if c == '<' {
                break;
            }
            adv += c.len_utf8();
        }
        if adv == 0 {
            adv = rest.chars().next().unwrap().len_utf8();
        }
        tokens.push((TokenKind::Plain, line[pos..pos + adv].to_string()));
        pos += adv;
    }

    tokens
}

fn tokenize_tag(tag: &str) -> Vec<(TokenKind, String)> {
    let mut tokens = Vec::new();
    let len = tag.len();
    let mut pos = 0usize;

    if let Some(after_lt) = tag.strip_prefix('<') {
        let mut adv = 1usize;
        if after_lt.starts_with('/') {
            adv += 1;
        }
        tokens.push((TokenKind::Tag, tag[..adv].to_string()));
        pos = adv;
    }

    let name_start = pos;
    let mut adv = 0usize;
    for c in tag[pos..].chars() {
        if c.is_alphanumeric() || c == '-' || c == ':' {
            adv += c.len_utf8();
        } else {
            break;
        }
    }
    if adv > 0 {
        tokens.push((TokenKind::Tag, tag[name_start..name_start + adv].to_string()));
        pos = name_start + adv;
    }

    while pos < len {
        let rest = &tag[pos..];
        let c0 = rest.chars().next().unwrap();

        if c0.is_whitespace() {
            let mut a = 0usize;
            for c in rest.chars() {
                if c.is_whitespace() {
                    a += c.len_utf8();
                } else {
                    break;
                }
            }
            tokens.push((TokenKind::Plain, tag[pos..pos + a].to_string()));
            pos += a;
            continue;
        }

        if c0 == '/' || c0 == '>' {
            tokens.push((TokenKind::Tag, c0.to_string()));
            pos += c0.len_utf8();
            continue;
        }

        if c0 == '"' || c0 == '\'' {
            let quote = c0;
            let mut end = pos + c0.len_utf8();
            for c in tag[end..].chars() {
                end += c.len_utf8();
                if c == quote {
                    break;
                }
            }
            tokens.push((TokenKind::String, tag[pos..end].to_string()));
            pos = end;
            continue;
        }

        if c0 == '=' {
            tokens.push((TokenKind::Plain, "=".to_string()));
            pos += 1;
            continue;
        }

        let mut a = 0usize;
        for c in rest.chars() {
            if c.is_whitespace() || c == '=' || c == '>' {
                break;
            }
            a += c.len_utf8();
        }
        if a == 0 {
            a = c0.len_utf8();
        }
        tokens.push((TokenKind::AttrName, tag[pos..pos + a].to_string()));
        pos += a;
    }

    tokens
}
