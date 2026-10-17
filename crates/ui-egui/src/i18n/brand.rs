//! NexusOffices: the product names the interface shows.
//!
//! Upstream's English labels (the catalog keys) and many translations say "PdfCraft" and
//! "ArtCraft", 545 times over 22 catalogs. Rather than rewrite keys or translations, the catalog
//! lookup brands what it hands out, once, in one place (`super::tr` and friends):
//! - every translation is branded when its catalog is loaded ([`brand`] over the catalog's
//!   values, `LangInfo::catalog`);
//! - an untranslated lookup (English, which has no catalog, or a string a catalog lacks) shows
//!   its key, so a key that names upstream is branded through [`branded_key`] — but only a key
//!   some bundled catalog has, that is, an interface string, never arbitrary text.
//!
//! So every language, English included, shows Nexus PDF / NexusOffices, while keys, command ids,
//! file keys and `PDFCRAFT_*` variables keep upstream's spelling: only `PdfCraft` and `ArtCraft`
//! spelled exactly so are replaced, and only in what the lookup returns. **Never apply this to
//! document or user text** (file names, field values, comment text, bookmark titles): those are
//! not looked up through the catalog, and the lookup functions say so in their doc comments.
//!
//! Accepted edge case: user text that is *exactly* a catalog key naming upstream and is shown
//! through the lookup anyway (a bookmark titled "About PdfCraft" passed to `tr`) reads "About
//! Nexus PDF". Text that only contains a name ("PdfCraft notes.pdf") is never rewritten; dialogs
//! that show document-provided names use the unbranded lookup.
//!
//! Text that must keep the upstream names (the attribution in About) is not looked up through the
//! catalog (`crate::nexus`, `pdfcraft_engine::nexus`).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Upstream name → the name NexusOffices shows, in the order they are replaced.
const NAMES: [(&str, &str); 2] = [("PdfCraft", "Nexus PDF"), ("ArtCraft", "NexusOffices")];

fn names_upstream(s: &str) -> bool {
    NAMES.iter().any(|(from, _)| s.contains(from))
}

/// `s` with the upstream product names replaced (`PdfCraft` → `Nexus PDF`, `ArtCraft` →
/// `NexusOffices`). Strings without them come back borrowed, without allocating.
pub fn brand(s: &str) -> Cow<'_, str> {
    if !names_upstream(s) {
        return Cow::Borrowed(s);
    }
    let mut out = s.to_string();
    for (from, to) in NAMES {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    Cow::Owned(out)
}

/// The text shown for an untranslated lookup of `key`: the key itself, branded when it is an
/// interface string that names upstream (a key of some bundled catalog). Anything else, including
/// text that merely contains `PdfCraft`, comes back unchanged.
pub(super) fn branded_key(key: &str) -> &str {
    if !names_upstream(key) {
        return key;
    }
    keys().get(key).map_or(key, String::as_str)
}

/// The catalog key of an engine message that Nexus PDF words with its own name at the source
/// (so the command line and MCP say Nexus PDF too) while upstream's catalogs still key it with
/// "PdfCraft": `Some` key to look up when `s` names Nexus PDF, `None` otherwise.
pub(super) fn upstream_key(s: &str) -> Option<String> {
    s.contains("Nexus PDF").then(|| s.replace("Nexus PDF", "PdfCraft"))
}

/// Every catalog key (plain, contextual and each plural form) that names upstream, with its
/// branded form. Built once, from the catalog lines that mention a name.
fn keys() -> &'static HashMap<String, String> {
    static KEYS: OnceLock<HashMap<String, String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys = HashMap::new();
        for lang in super::LANGUAGES.iter() {
            let lines: String = lang.source.lines().filter(|line| names_upstream(line)).flat_map(|line| [line, "\n"]).collect();
            let (entries, _) = super::catalog::parse_entries(&lines, lang.plural_forms());
            for entry in entries.into_iter().filter(|e| e.context != "@id") {
                for source in entry.source.split('|').filter(|s| names_upstream(s)) {
                    keys.entry(source.to_string()).or_insert_with(|| brand(source).into_owned());
                }
            }
        }
        keys
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Lang, tr, tr_ctx, trn};

    #[test]
    fn brand_substitutes_product_names() {
        assert_eq!(brand("About PdfCraft"), "About Nexus PDF");
        assert_eq!(brand("Join the ArtCraft community"), "Join the NexusOffices community");
        assert_eq!(brand("Giới thiệu về PdfCraft"), "Giới thiệu về Nexus PDF");
        assert!(matches!(brand("Giới thiệu"), Cow::Borrowed("Giới thiệu")));
        // Identifiers and variables keep upstream's spelling.
        assert!(matches!(brand("pdfcraft:open PDFCRAFT_MODELS pdfcraft-cli"), Cow::Borrowed(_)));
    }

    /// Lookups brand in every language, English included, and still fall back to the label.
    #[test]
    fn interface_lookups_show_nexus_names_in_english_and_vietnamese() {
        let (en, vi) = (Lang::from_code("en").unwrap(), Lang::from_code("vi").unwrap());
        assert_eq!(tr(en, "About PdfCraft"), "About Nexus PDF");
        assert_eq!(tr(en, "Welcome to PdfCraft"), "Welcome to Nexus PDF");
        assert_eq!(tr(en, "Join the ArtCraft community"), "Join the NexusOffices community");
        for lang in [en, vi] {
            for key in ["About PdfCraft", "Welcome to PdfCraft", "PdfCraft {v} is available.", "PdfCraft on GitHub"] {
                let shown = tr(lang, key);
                assert!(shown.contains("Nexus PDF") && !shown.contains("PdfCraft"), "{}: {shown}", lang.code());
            }
        }
        assert!(tr(vi, "About PdfCraft").starts_with("Giới thiệu"), "still the Vietnamese translation: {}", tr(vi, "About PdfCraft"));
        assert_eq!(tr_ctx(en, "no such context", "About PdfCraft"), "About Nexus PDF");
        assert!(!trn(en, 2, "{n} file", "{n} files").contains("Nexus"));
    }

    /// Engine messages worded with "Nexus PDF" at the source still find upstream's translation.
    #[test]
    fn engine_messages_renamed_at_the_source_keep_their_translation() {
        let vi = Lang::from_code("vi").unwrap();
        let ocr = "the text recognition models are not installed (reinstall Nexus PDF, or set PDFCRAFT_MODELS to the folder that holds them)";
        let shown = tr(vi, ocr);
        assert!(shown.starts_with("các mô hình") && shown.contains("Nexus PDF") && !shown.contains("PdfCraft"), "{shown}");
        assert_eq!(tr(Lang::from_code("en").unwrap(), ocr), ocr);
        assert_eq!(upstream_key("Close"), None);
    }

    /// Only interface strings are branded: text that is not a catalog key comes back as it is,
    /// so a file or field named after upstream keeps its name.
    #[test]
    fn untranslated_text_that_is_not_a_catalog_key_is_never_rewritten() {
        let en = Lang::from_code("en").unwrap();
        let text = String::from("My PdfCraft notes.pdf");
        assert!(std::ptr::eq(tr(en, &text), text.as_str()));
        assert!(std::ptr::eq(branded_key(&text), text.as_str()));
        let plain = String::from("Close");
        assert!(std::ptr::eq(branded_key(&plain), plain.as_str()));
    }
}
