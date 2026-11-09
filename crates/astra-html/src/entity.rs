//! HTML character reference and entity decoding.

/// Decodes common named HTML entities and numeric character references.
#[must_use]
pub fn decode_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }

    let mut output = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' {
            // Check for entity termination ';' within reasonable distance (max 32 chars)
            let mut j = i + 1;
            let mut found_semicolon = false;
            while j < chars.len() && j - i < 32 {
                if chars[j] == ';' {
                    found_semicolon = true;
                    break;
                }
                if chars[j].is_whitespace() || chars[j] == '<' || chars[j] == '&' {
                    break;
                }
                j += 1;
            }

            if found_semicolon {
                let entity_str: String = chars[i + 1..j].iter().collect();
                if let Some(decoded) = resolve_entity(&entity_str) {
                    output.push_str(&decoded);
                    i = j + 1;
                    continue;
                }
            }
        }

        output.push(chars[i]);
        i += 1;
    }

    output
}

fn resolve_entity(entity: &str) -> Option<String> {
    if let Some(stripped) = entity.strip_prefix('#') {
        // Numeric character reference
        let code = if let Some(hex) = stripped
            .strip_prefix('x')
            .or_else(|| stripped.strip_prefix('X'))
        {
            u32::from_str_radix(hex, 16).ok()
        } else {
            stripped.parse::<u32>().ok()
        };

        code.and_then(char::from_u32).map(|c| c.to_string())
    } else {
        // Named entities
        match entity {
            "amp" => Some("&".to_string()),
            "lt" => Some("<".to_string()),
            "gt" => Some(">".to_string()),
            "quot" => Some("\"".to_string()),
            "apos" => Some("'".to_string()),
            "nbsp" => Some("\u{00A0}".to_string()),
            "copy" => Some("\u{00A9}".to_string()),
            "reg" => Some("\u{00AE}".to_string()),
            _ => None,
        }
    }
}
