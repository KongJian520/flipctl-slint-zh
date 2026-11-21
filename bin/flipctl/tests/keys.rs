//! Every id the panel asks for has text, in the source language and in each locale.
//!
//! `build.rs` can check that the locale files agree with each other; it cannot see
//! which ids the code uses, and an id the catalogue has never heard of draws as
//! itself — `menu.network.wifi` on a row, which is how this test came to exist. The
//! source is compiled in with `include_str!`, so the check runs on the same text the
//! compiler did.

const SOURCE: &str = include_str!("../src/main.rs");

/// The leading segment of every id the catalogue carries: `menu`, `detail`, `action`
/// and so on. A dotted string that does not start with one of these is a filename or
/// a version, not an id.
fn groups() -> Vec<&'static str> {
    let mut groups: Vec<&'static str> = flipper_ui::strings::ids()
        .iter()
        .filter_map(|(id, _)| id.split('.').next())
        .collect();
    groups.sort_unstable();
    groups.dedup();
    groups
}

/// An id rather than a value: one of the catalogue's groups, then lowercase words
/// joined by dots, e.g. `menu.main.apps`.
fn looks_like_a_key_with(text: &str, groups: &[&str]) -> bool {
    let mut parts = text.split('.');
    let Some(first) = parts.next() else { return false };
    if !groups.contains(&first) || parts.clone().count() == 0 {
        return false;
    }
    parts.all(|w| {
        !w.is_empty()
            && w.starts_with(|c: char| c.is_ascii_lowercase())
            && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    })
}

fn keys_in<'a>(source: &'a str, groups: &[&str]) -> Vec<&'a str> {
    let mut found: Vec<&str> = Vec::new();
    let bytes = source.as_bytes();
    let mut at = 0;
    while let Some(start) = source[at..].find('"') {
        let open = at + start + 1;
        let Some(len) = source[open..].find('"') else { break };
        let text = &source[open..open + len];
        if looks_like_a_key_with(text, groups) {
            found.push(text);
        }
        at = open + len + 1;
        let _ = bytes;
    }
    found.sort_unstable();
    found.dedup();
    found
}

#[test]
fn every_key_the_panel_uses_has_text() {
    let groups = groups();
    let missing: Vec<&str> = keys_in(SOURCE, &groups)
        .into_iter()
        .filter(|key| !flipper_ui::strings::has(key))
        .collect();
    assert!(missing.is_empty(), "no text for these ids: {missing:?}");
}

#[test]
fn the_shape_test_tells_ids_from_values() {
    let groups = groups();
    assert!(groups.contains(&"menu"), "the catalogue's groups: {groups:?}");
    assert!(looks_like_a_key_with("menu.main.apps", &groups));
    assert!(looks_like_a_key_with("detail.usb_c1", &groups));
    assert!(!looks_like_a_key_with("172.17.0.1", &groups));
    assert!(!looks_like_a_key_with("Airplane mode", &groups));
    assert!(!looks_like_a_key_with("icon.png", &groups));
    assert!(!looks_like_a_key_with("b9f8f8c", &groups));
    assert!(!looks_like_a_key_with("menu.", &groups));
}
