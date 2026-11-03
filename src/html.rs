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
/// Numeric references follow HTML5: `&#128;`..`&#159;` map through the
/// windows-1252 table (`&#146;` is `’`), and `&#0;`, surrogates and values
/// above U+10FFFF decode to U+FFFD.
///
/// Unrecognized named entities and malformed sequences (including numeric
/// references with no digits or with a sign) are passed through unchanged.
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
        // Search bytes, not a `str` slice: 14 bytes can end inside a multi-byte
        // character, and slicing there panicked. `;` is ASCII, so its byte
        // index is always a char boundary.
        let search_end = rest.len().min(14); // &xxxxxxxxxxxx;
        let window = &rest.as_bytes()[..search_end];
        if let Some(semi_offset) = window.iter().position(|&b| b == b';') {
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
            numeric_reference(hex, 16)
        } else {
            numeric_reference(stripped, 10)
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

/// Decode the digits of a numeric character reference as HTML5 does
/// (WHATWG "numeric character reference end state").
fn numeric_reference(digits: &str, radix: u32) -> Option<char> {
    // Digits only: `u32::from_str_radix` would also accept a leading `+`.
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    // The only remaining parse error is overflow, which is out of range.
    let code = u32::from_str_radix(digits, radix).unwrap_or(u32::MAX);
    let c = match code {
        0x80..=0x9F => windows_1252(code as u8),
        _ => char::from_u32(code),
    };
    // NUL, surrogates and values above U+10FFFF.
    Some(c.filter(|&c| c != '\0').unwrap_or('\u{FFFD}'))
}

/// HTML5 maps references to C1 controls through windows-1252; the five bytes
/// that windows-1252 leaves undefined keep their own code point.
fn windows_1252(b: u8) -> Option<char> {
    Some(match b {
        0x80 => '\u{20AC}',
        0x82 => '\u{201A}',
        0x83 => '\u{0192}',
        0x84 => '\u{201E}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02C6}',
        0x89 => '\u{2030}',
        0x8A => '\u{0160}',
        0x8B => '\u{2039}',
        0x8C => '\u{0152}',
        0x8E => '\u{017D}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201C}',
        0x94 => '\u{201D}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02DC}',
        0x99 => '\u{2122}',
        0x9A => '\u{0161}',
        0x9B => '\u{203A}',
        0x9C => '\u{0153}',
        0x9E => '\u{017E}',
        0x9F => '\u{0178}',
        _ => char::from(b),
    })
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
    fn multibyte_text_after_ampersand_does_not_panic() {
        // The 14-byte search window used to end inside a 2-byte character.
        assert_eq!(decode_entities("&ééééééé;"), "&ééééééé;");
        // Every window offset, with 2-, 3- and 4-byte characters.
        for filler in ["é", "€", "😀"] {
            for prefix in 0..8 {
                let text = format!("&{}{}&amp;", "a".repeat(prefix), filler.repeat(6));
                assert!(decode_entities(&text).ends_with('&'));
            }
        }
        // An entity right after multi-byte text still decodes.
        assert_eq!(decode_entities("café&amp;crème"), "café&crème");
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
        // Out-of-range code points decode to U+FFFD, as in HTML5.
        assert_eq!(decode_entities("&#xFFFFFF;"), "\u{FFFD}");
        // Digits that overflow u32 are also out of range.
        assert_eq!(decode_entities("&#99999999999;"), "\u{FFFD}");
        // No digits: not a character reference.
        assert_eq!(decode_entities("&#;"), "&#;");
        assert_eq!(decode_entities("&#x;"), "&#x;");
    }

    #[test]
    fn decode_entities_follows_html5_numeric_rules() {
        // 0x80..=0x9F map through the windows-1252 table.
        assert_eq!(decode_entities("don&#146;t"), "don\u{2019}t");
        assert_eq!(decode_entities("&#150;"), "\u{2013}");
        assert_eq!(decode_entities("&#x80;"), "\u{20AC}");
        assert_eq!(decode_entities("&#x9F;"), "\u{0178}");
        // Bytes windows-1252 leaves undefined keep their own code point.
        assert_eq!(decode_entities("&#x81;"), "\u{0081}");
        // NUL, surrogates and values above U+10FFFF become U+FFFD.
        assert_eq!(decode_entities("&#0;"), "\u{FFFD}");
        assert_eq!(decode_entities("&#xD800;"), "\u{FFFD}");
        assert_eq!(decode_entities("&#xDFFF;"), "\u{FFFD}");
        assert_eq!(decode_entities("&#x110000;"), "\u{FFFD}");
        // Control: ordinary code points around those ranges are unchanged.
        assert_eq!(
            decode_entities("&#x7F;&#xA0;&#xD7FF;&#xE000;&#x10FFFF;"),
            "\u{7F}\u{A0}\u{D7FF}\u{E000}\u{10FFFF}"
        );
    }

    #[test]
    fn decode_entities_rejects_plus_sign() {
        assert_eq!(decode_entities("&#+65;"), "&#+65;");
        assert_eq!(decode_entities("&#x+41;"), "&#x+41;");
        assert_eq!(decode_entities("&#-65;"), "&#-65;");
        // Control: unsigned digits still decode.
        assert_eq!(decode_entities("&#65;&#x41;"), "AA");
    }
}
