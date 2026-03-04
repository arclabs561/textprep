//! HTML entity decoding.
//!
//! Decodes the most common HTML entities (`&amp;`, `&#x2019;`, `&#169;`, etc.)
//! without pulling in a full HTML parser. Useful for cleaning NER inputs that
//! originate from web scraping or rich-text pipelines.

/// Decode HTML entities in `text`.
///
/// Handles:
/// - Named entities: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`
/// - Decimal numeric: `&#123;`
/// - Hexadecimal numeric: `&#x1F4A9;`, `&#X1f4a9;`
///
/// Unrecognized named entities and malformed sequences are passed through
/// unchanged.
///
/// # Examples
///
/// ```
/// use textprep::html::decode_entities;
///
/// assert_eq!(decode_entities("fish &amp; chips"), "fish & chips");
/// assert_eq!(decode_entities("&#169;"), "\u{00A9}");
/// assert_eq!(decode_entities("&#x2019;"), "\u{2019}");
/// ```
pub fn decode_entities(text: &str) -> String {
    // Fast path: no ampersand means no entities.
    if !text.contains('&') {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(amp_pos) = rest.find('&') {
        // Copy everything before the '&'.
        out.push_str(&rest[..amp_pos]);
        rest = &rest[amp_pos..]; // rest now starts with '&'

        // Look for the closing ';' within a reasonable window (max 12 chars for entity).
        let search_end = rest.len().min(14); // &xxxxxxxxxxxx;
        if let Some(semi_offset) = rest[..search_end].find(';') {
            let entity = &rest[1..semi_offset]; // between '&' and ';'

            if let Some(decoded) = decode_one(entity) {
                out.push(decoded);
                rest = &rest[semi_offset + 1..];
                continue;
            }
        }

        // Not a valid entity -- emit the '&' literally and advance.
        out.push('&');
        rest = &rest[1..];
    }

    // Remaining text after the last entity.
    out.push_str(rest);
    out
}

/// Try to decode a single entity body (the part between `&` and `;`).
fn decode_one(entity: &str) -> Option<char> {
    // Numeric: &#NNN; or &#xHH;
    if let Some(stripped) = entity.strip_prefix('#') {
        return if let Some(hex) = stripped
            .strip_prefix('x')
            .or_else(|| stripped.strip_prefix('X'))
        {
            u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
        } else {
            stripped.parse::<u32>().ok().and_then(char::from_u32)
        };
    }

    // Named entities.
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00A0}'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_entities() {
        assert_eq!(decode_entities("&amp;"), "&");
        assert_eq!(decode_entities("&lt;"), "<");
        assert_eq!(decode_entities("&gt;"), ">");
        assert_eq!(decode_entities("&quot;"), "\"");
        assert_eq!(decode_entities("&apos;"), "'");
        assert_eq!(decode_entities("&nbsp;"), "\u{00A0}");
    }

    #[test]
    fn numeric_decimal() {
        assert_eq!(decode_entities("&#169;"), "\u{00A9}"); // copyright
        assert_eq!(decode_entities("&#8217;"), "\u{2019}"); // right single quote
        assert_eq!(decode_entities("&#65;"), "A");
    }

    #[test]
    fn numeric_hex() {
        assert_eq!(decode_entities("&#x2019;"), "\u{2019}");
        assert_eq!(decode_entities("&#x41;"), "A");
        assert_eq!(decode_entities("&#X41;"), "A"); // uppercase X
    }

    #[test]
    fn mixed() {
        assert_eq!(
            decode_entities("fish &amp; chips &#x2014; good"),
            "fish & chips \u{2014} good"
        );
    }

    #[test]
    fn already_clean() {
        assert_eq!(decode_entities("no entities here"), "no entities here");
    }

    #[test]
    fn malformed_unclosed() {
        // Unclosed ampersand passes through.
        assert_eq!(decode_entities("AT&T"), "AT&T");
    }

    #[test]
    fn malformed_unknown_named() {
        // Unknown named entity passes through.
        assert_eq!(decode_entities("&foo;"), "&foo;");
    }

    #[test]
    fn empty_input() {
        assert_eq!(decode_entities(""), "");
    }

    #[test]
    fn consecutive_entities() {
        assert_eq!(decode_entities("&amp;&lt;&gt;"), "&<>");
    }

    #[test]
    fn entity_at_end() {
        assert_eq!(decode_entities("end&amp;"), "end&");
    }

    #[test]
    fn bare_ampersand_mid_text() {
        assert_eq!(decode_entities("a & b"), "a & b");
    }

    #[test]
    fn invalid_numeric() {
        // Invalid code point passes through.
        assert_eq!(decode_entities("&#xFFFFFF;"), "&#xFFFFFF;");
    }
}
