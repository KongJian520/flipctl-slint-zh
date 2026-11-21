//! The panel's copy, in every language the build carries.
//!
//! The source of truth is `locales/*.toml` — one file per language, `"id" = "text"`,
//! the file's `locale` naming itself. `build.rs` reads them, refuses to build when one
//! is missing an id another has, and writes the tables in below. A language is
//! therefore added by adding a file: no code changes, no list to keep in step.
//!
//! Keys are stable ids (`menu.network.airplane`), not the English text. The text moves
//! when somebody improves the wording; the id must not, because it is what every
//! translation and every comparison is written against.
//!
//! Lookups happen where text is drawn, not where it is stored: a menu's rows and
//! breadcrumb, a detail row's label and value, a screen's breadcrumb, the soft keys,
//! the status words. Five places, so the tables reach ten screens without two hundred
//! call sites to keep in step.
//!
//! [`t`] answers for anything: a key finds its text, a string that is not a key (an IP
//! address, an SSID, a byte count) comes back as itself. A key the current locale
//! lacks falls back to `en`, and a key nothing has falls back to the key — visible,
//! which is the point, rather than an empty row.

use std::sync::atomic::{AtomicUsize, Ordering};

include!(concat!(env!("OUT_DIR"), "/i18n_generated.rs"));

/// No locale chosen yet, so `locale()` answers with the default one.
const UNSET: usize = usize::MAX;

static CURRENT: AtomicUsize = AtomicUsize::new(UNSET);

/// The library's default is the source language. Which language a *panel* starts in
/// is the binary's decision, not a crate's: flipctl asks for zh-Hans in main, and a
/// golden that drew Chinese would be a golden nobody could read the diff of.
const DEFAULT: &str = "en";

/// Every language in the image, `en` first.
pub fn locales() -> &'static [Locale] {
    LOCALES
}

/// The language being drawn.
pub fn locale() -> &'static Locale {
    &LOCALES[index()]
}

fn index() -> usize {
    let at = CURRENT.load(Ordering::Relaxed);
    if at == UNSET {
        LOCALES.iter().position(|l| l.id == DEFAULT).unwrap_or(0)
    } else {
        at
    }
}

fn texts(at: usize) -> &'static [(&'static str, &'static str)] {
    TEXTS[at]
}

fn find<'a>(table: &'static [(&'static str, &'static str)], key: &'a str) -> Option<&'a str> {
    table.binary_search_by(|(id, _)| (*id).cmp(key)).ok().map(|at| table[at].1)
}

/// Switch the language. False when no such locale is in the build; the caller is
/// expected to redraw either way.
pub fn set_locale(id: &str) -> bool {
    match LOCALES.iter().position(|l| l.id == id) {
        Some(at) => {
            CURRENT.store(at, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

/// The text for `key` in the current language.
///
/// Takes any `&str` rather than a `&'static str` so a detail row's value (an IP
/// address, an SSID) can pass through the same door: it is not a key, so it comes
/// back as itself. A key from the tables is `'static` and narrows to the caller's
/// lifetime for free.
pub fn t<'a>(key: &'a str) -> &'a str {
    if let Some(text) = find(texts(index()), key) {
        return text;
    }
    // `en` is the source language, so it is what a half-translated locale falls back to.
    find(texts(0), key).unwrap_or(key)
}

/// The source language's table: every id the build carries, sorted.
pub fn ids() -> &'static [(&'static str, &'static str)] {
    texts(0)
}

/// Whether the source language has text for `key`. For tests that check the keys
/// the code uses are the keys the catalogue carries.
pub fn has(key: &str) -> bool {
    find(texts(0), key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_names_itself() {
        for locale in locales() {
            assert!(!locale.id.is_empty());
            assert!(!locale.name.is_empty(), "{} has no name", locale.id);
        }
    }

    #[test]
    fn the_tables_are_sorted_so_the_lookup_can_binary_search() {
        for (at, table) in TEXTS.iter().enumerate() {
            for pair in table.windows(2) {
                assert!(pair[0].0 < pair[1].0, "{}: {} out of order", LOCALES[at].id, pair[1].0);
            }
        }
    }

    #[test]
    fn a_key_finds_its_text_and_a_value_comes_back_as_itself() {
        set_locale("en");
        assert_eq!(t("menu.main.network"), "Network");
        set_locale("zh-Hans");
        assert_eq!(t("menu.main.network"), "网络");
        assert_eq!(t("172.17.0.1"), "172.17.0.1");
        assert_eq!(t("no.such.key"), "no.such.key");
    }

    #[test]
    fn an_unknown_locale_is_refused_rather_than_silently_ignored() {
        assert!(!set_locale("kl-GL"));
        assert!(set_locale("zh-Hans"));
    }
}
