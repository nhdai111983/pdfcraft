//! NexusOffice: the product names the interface shows.
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
//! So every language, English included, shows Nexus PDF / NexusOffice, while keys, command ids,
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

/// Upstream name → the name NexusOffice shows, in the order they are replaced.
const NAMES: [(&str, &str); 2] = [("PdfCraft", "Nexus PDF"), ("ArtCraft", "NexusOffice")];

fn names_upstream(s: &str) -> bool {
    NAMES.iter().any(|(from, _)| s.contains(from))
}

/// `s` with the upstream product names replaced (`PdfCraft` → `Nexus PDF`, `ArtCraft` →
/// `NexusOffice`). Strings without them come back borrowed, without allocating.
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

/// Upstream labels that name a Microsoft product (NexusOffice's interface names the file format
/// instead; a format's extension is fine): English key, what English shows, what Vietnamese
/// shows. Other languages show the English: NexusOffice ships English and Vietnamese, and their
/// catalogs would otherwise keep the product's name ("Exporter vers Word…").
const VENDOR_FREE: [(&str, &str, &str); 3] = [
    ("Export to Word…", "Export to DOCX…", "Xuất ra DOCX…"),
    ("Export to PowerPoint…", "Export to PPTX…", "Xuất ra PPTX…"),
    ("Microsoft Word (.docx)", "Document (.docx)", "Văn bản (.docx)"),
];

/// The NexusOffice layout's own labels that name a Microsoft product (`nexus_wps.rs`,
/// `nexus_backstage.rs`): English key and what English shows. Vietnamese comes from
/// `crate::nexus_wps::NEXUS_VI`, the one table of the layout's Vietnamese.
const VENDOR_FREE_NEXUS: [(&str, &str); 5] = [
    ("To Word", "To DOCX"),
    ("To PowerPoint", "To PPTX"),
    ("Word document…", "Document (.docx)…"),
    ("PowerPoint presentation…", "Presentation (.pptx)…"),
    ("Edit the text in Nexus Docs or Word", "Edit the text in Nexus Docs or another word processor"),
];

/// What `lang` shows for a label `key` that names a Microsoft product, or `None` for any other
/// key. Exact keys only, so document and user text are never rewritten.
pub(super) fn vendor_free(lang: &str, key: &str) -> Option<&'static str> {
    if let Some((_, en, vi)) = VENDOR_FREE.iter().find(|(k, ..)| *k == key) {
        return Some(if lang == "vi" { vi } else { en });
    }
    let (_, en) = VENDOR_FREE_NEXUS.iter().find(|(k, _)| *k == key)?;
    if lang == "vi"
        && let Some((_, vi)) = crate::nexus_wps::NEXUS_VI.iter().find(|(k, _)| *k == key)
    {
        return Some(vi);
    }
    Some(en)
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
        assert_eq!(brand("Join the ArtCraft community"), "Join the NexusOffice community");
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
        assert_eq!(tr(en, "Join the ArtCraft community"), "Join the NexusOffice community");
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

    /// A whole word of `shown` that is a Microsoft product's name. "Word" is also the common noun
    /// of Word count, so those labels are let through by name.
    fn names_microsoft_product(shown: &str) -> bool {
        const PRODUCTS: [&str; 7] = ["Word", "PowerPoint", "Excel", "Outlook", "Microsoft", "OneNote", "Visio"];
        const COMMON_NOUN: [&str; 3] = ["Word count", "Word count…", "Word counting stopped unexpectedly."];
        !COMMON_NOUN.contains(&shown) && shown.split(|c: char| !c.is_alphanumeric()).any(|w| PRODUCTS.contains(&w))
    }

    /// NexusOffice's interface never names a Microsoft product (QC 02/11 and review 05/11: the
    /// Vietnamese catalog's "Xuất ra Word…", English "Export to Word…", "To Word", "Microsoft Word
    /// (.docx)"), in English or Vietnamese, wherever a label is shown: the menus (every registered
    /// command, as `menu_label` shows it), the tool panels (every catalog tool item), every
    /// catalog key, and the NexusOffice layout's own labels.
    #[test]
    fn labels_shown_in_english_and_vietnamese_name_no_microsoft_product() {
        let mut offenders = Vec::new();
        for lang in [Lang::from_code("en").unwrap(), Lang::from_code("vi").unwrap()] {
            crate::i18n::set_current(lang);
            let commands = pdfcraft_engine::commands::COMMANDS.iter().map(|c| crate::i18n::menu_label(c.id, c.label));
            let tools = pdfcraft_engine::catalog::TOOL_GROUPS
                .iter()
                .flat_map(|g| g.sections.iter())
                .flat_map(|s| s.items.iter())
                .map(|item| crate::i18n::t(item.label).to_string());
            let keys = super::super::LANGUAGES
                .iter()
                .filter(|l| l.code == "vi")
                .flat_map(|l| l.source.lines())
                .filter_map(|line| line.split('\t').nth(1))
                .map(|key| crate::i18n::t(key).to_string());
            let layout = crate::nexus_wps::NEXUS_VI.iter().map(|(en, _)| crate::nexus_wps::tx(en).to_string());
            for shown in commands.chain(tools).chain(keys).chain(layout) {
                if names_microsoft_product(&shown) {
                    offenders.push(format!("{}: {shown}", lang.code()));
                }
            }
        }
        crate::i18n::set_current(Lang::from_code("en").unwrap());
        assert!(offenders.is_empty(), "{offenders:#?}");
        // The export commands still say what they make.
        let vi = Lang::from_code("vi").unwrap();
        assert_eq!(tr(vi, "Export to Word…"), "Xuất ra DOCX…");
        assert_eq!(tr(Lang::from_code("en").unwrap(), "Export to PowerPoint…"), "Export to PPTX…");
        assert_eq!(tr(Lang::from_code("fr").unwrap(), "Export to Word…"), "Export to DOCX…");
        assert_eq!(vendor_free("vi", "To Word"), Some("Sang DOCX"));
        // Exact keys only: document text that merely mentions Word is never rewritten.
        assert_eq!(vendor_free("en", "Export to Word… notes"), None);
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
