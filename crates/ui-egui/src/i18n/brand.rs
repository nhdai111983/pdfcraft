//! NexusOffice: the product names the interface shows.
//!
//! Upstream's English labels (the catalog keys) and many translations (craft-i18n's
//! `pdfcraft.tsv`/`common.tsv`) say "PdfCraft" and "ArtCraft". Rather than rewrite keys or
//! translations, the catalog lookup brands what it hands out, once, in one place (`super::tr` and
//! friends):
//! - every translation a lookup finds is branded ([`branded`], kept once per string);
//! - an untranslated lookup (English, which has no catalog, or a string a catalog lacks) shows
//!   its key, so a key that names upstream is branded through [`branded_key`] — but only a key
//!   some embedded catalog has, that is, an interface string, never arbitrary text. A build
//!   without the catalogs (`CRAFT_I18N_DIR` unset: English only, never a release) brands every
//!   lookup that names upstream.
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
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

/// Upstream name → the name NexusOffice shows, in the order they are replaced.
const NAMES: [(&str, &str); 2] = [("PdfCraft", "Nexus PDF"), ("ArtCraft", "NexusOffice")];

pub(super) fn names_upstream(s: &str) -> bool {
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

/// At most this many distinct branded strings are kept ([`branded`]). Interface strings naming
/// upstream number in the hundreds; the cap only bounds memory if something unexpected were ever
/// passed through the lookup, which then comes back unbranded.
const MAX_INTERNED: usize = 4096;

fn cache() -> std::sync::MutexGuard<'static, HashMap<String, &'static str>> {
    static CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    // A poisoned lock still holds a usable map (inserts are single statements).
    CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap_or_else(|e| e.into_inner())
}

/// [`brand`] for the lookups, which hand out `&str`: a string that names upstream is branded once
/// and kept for the life of the process; anything else comes back as it is.
pub(super) fn branded(s: &str) -> &str {
    if !names_upstream(s) {
        return s;
    }
    let mut map = cache();
    if let Some(&hit) = map.get(s) {
        return hit;
    }
    if map.len() >= MAX_INTERNED {
        return s;
    }
    let leaked: &'static str = Box::leak(brand(s).into_owned().into_boxed_str());
    map.insert(s.to_string(), leaked);
    leaked
}

/// `s` branded, for a translation found under a key that only lives for the lookup
/// ([`upstream_key`]): a catalog row, so the set is bounded by the catalogs. Kept once per string.
pub(super) fn interned(s: &str) -> &'static str {
    let mut map = cache();
    if let Some(&hit) = map.get(s) {
        return hit;
    }
    let leaked: &'static str = Box::leak(brand(s).into_owned().into_boxed_str());
    map.insert(s.to_string(), leaked);
    leaked
}

/// The text shown for an untranslated lookup of `key`: the key itself, branded when it is an
/// interface string that names upstream (a key of some embedded catalog, or any key in a build
/// without catalogs). Anything else, including text that merely contains `PdfCraft`, comes back
/// unchanged.
pub(super) fn branded_key(key: &str) -> &str {
    if !names_upstream(key) {
        return key;
    }
    if super::catalogs_embedded() && !keys().contains(key) && !UNCATALOGUED_KEYS.contains(&key) {
        return key;
    }
    branded(key)
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
const VENDOR_FREE: [(&str, &str, &str); 7] = [
    ("Export to Word…", "Export to DOCX…", "Xuất ra DOCX…"),
    ("Export to PowerPoint…", "Export to PPTX…", "Xuất ra PPTX…"),
    ("Microsoft Word (.docx)", "Document (.docx)", "Văn bản (.docx)"),
    // Rows of craft-i18n's PdfCraft catalog (sync 2026-11-05), for the spreadsheet export.
    ("Export to Excel", "Export to XLSX", "Xuất ra XLSX"),
    ("Export to Excel…", "Export to XLSX…", "Xuất ra XLSX…"),
    (
        "Tables become cells; numbers, amounts, percentages and dates become values Excel can compute with, shown as on the page.",
        "Tables become cells; numbers, amounts, percentages and dates become values a spreadsheet can compute with, shown as on the page.",
        "Bảng trở thành các ô; số, số tiền, tỷ lệ phần trăm và ngày tháng trở thành giá trị mà bảng tính có thể tính toán, hiển thị như trên trang.",
    ),
    ("Word, PowerPoint and Excel (converted to PDF)", "DOCX, PPTX and XLSX (converted to PDF)", "DOCX, PPTX và XLSX (chuyển đổi sang PDF)"),
];

/// Upstream interface strings that name PdfCraft and that no embedded catalog has yet: they are
/// still interface strings, so [`branded_key`] brands them like a catalog key (sync 2026-11-05).
const UNCATALOGUED_KEYS: [&str; 1] = ["Check for updates when PdfCraft starts"];

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

/// Every catalog key (plain, contextual and each plural form) of the embedded catalogs that names
/// upstream. Built once, from the catalog lines that mention a name.
fn keys() -> &'static HashSet<String> {
    static KEYS: OnceLock<HashSet<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let unescape = |s: &str| s.replace("\\n", "\n").replace("\\t", "\t").replace("\\\\", "\\");
        let mut keys = HashSet::new();
        for (_, shared, app) in super::CRAFT_I18N.iter() {
            for line in shared.lines().chain(app.lines()).filter(|l| !l.starts_with('#') && names_upstream(l)) {
                let mut columns = line.split('\t');
                let (Some(context), Some(source)) = (columns.next(), columns.next()) else { continue };
                if context == "@id" {
                    continue;
                }
                for source in source.split('|').filter(|s| names_upstream(s)) {
                    keys.insert(unescape(source));
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
        let en = Lang::EN;
        assert_eq!(tr(en, "About PdfCraft"), "About Nexus PDF");
        assert_eq!(tr(en, "Welcome to PdfCraft"), "Welcome to Nexus PDF");
        assert_eq!(tr(en, "Join the ArtCraft community"), "Join the NexusOffice community");
        assert_eq!(tr_ctx(en, "no such context", "About PdfCraft"), "About Nexus PDF");
        assert!(!trn(en, 2, "{n} file", "{n} files").contains("Nexus"));
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let Some(vi) = Lang::from_code("vi") else { panic!("the Vietnamese catalog is embedded") };
        for lang in [en, vi] {
            for key in ["About PdfCraft", "Welcome to PdfCraft", "PdfCraft {v} is available.", "PdfCraft on GitHub"] {
                let shown = tr(lang, key);
                assert!(shown.contains("Nexus PDF") && !shown.contains("PdfCraft"), "{}: {shown}", lang.code());
            }
        }
        assert!(tr(vi, "About PdfCraft").starts_with("Giới thiệu"), "still the Vietnamese translation: {}", tr(vi, "About PdfCraft"));
    }

    /// Engine messages worded with "Nexus PDF" at the source still find upstream's translation.
    #[test]
    fn engine_messages_renamed_at_the_source_keep_their_translation() {
        assert_eq!(upstream_key("Close"), None);
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let Some(vi) = Lang::from_code("vi") else { panic!("the Vietnamese catalog is embedded") };
        let ocr = "the text recognition models are not installed (reinstall Nexus PDF, or set PDFCRAFT_MODELS to the folder that holds them)";
        let shown = tr(vi, ocr);
        assert!(shown.starts_with("các mô hình") && shown.contains("Nexus PDF") && !shown.contains("PdfCraft"), "{shown}");
        assert_eq!(tr(Lang::EN, ocr), ocr);
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
        // English always; Vietnamese and French when the craft-i18n catalogs are embedded.
        let vi = Lang::from_code("vi");
        for lang in std::iter::once(Lang::EN).chain(vi) {
            crate::i18n::set_current(lang);
            let commands = pdfcraft_engine::commands::COMMANDS.iter().map(|c| crate::i18n::menu_label(c.id, c.label));
            let tools = pdfcraft_engine::catalog::TOOL_GROUPS
                .iter()
                .flat_map(|g| g.sections.iter())
                .flat_map(|s| s.items.iter())
                .map(|item| crate::i18n::t(item.label).to_string());
            let keys = vi
                .into_iter()
                .flat_map(Lang::catalog_texts)
                .flat_map(str::lines)
                .filter(|line| !line.starts_with('#'))
                .filter_map(|line| line.split('\t').nth(1))
                .map(|key| crate::i18n::t(key).to_string());
            let layout = crate::nexus_wps::NEXUS_VI.iter().map(|(en, _)| crate::nexus_wps::tx(en).to_string());
            for shown in commands.chain(tools).chain(keys).chain(layout) {
                if names_microsoft_product(&shown) {
                    offenders.push(format!("{}: {shown}", lang.code()));
                }
            }
        }
        crate::i18n::set_current(Lang::EN);
        assert!(offenders.is_empty(), "{offenders:#?}");
        // The export commands still say what they make.
        assert_eq!(tr(Lang::EN, "Export to PowerPoint…"), "Export to PPTX…");
        if let (Some(vi), Some(fr)) = (vi, Lang::from_code("fr")) {
            assert_eq!(tr(vi, "Export to Word…"), "Xuất ra DOCX…");
            assert_eq!(tr(fr, "Export to Word…"), "Export to DOCX…");
        }
        assert_eq!(vendor_free("vi", "To Word"), Some("Sang DOCX"));
        // Exact keys only: document text that merely mentions Word is never rewritten.
        assert_eq!(vendor_free("en", "Export to Word… notes"), None);
    }

    /// Only interface strings are branded: text that is not a catalog key comes back as it is,
    /// so a file or field named after upstream keeps its name.
    #[test]
    fn untranslated_text_that_is_not_a_catalog_key_is_never_rewritten() {
        let plain = String::from("Close");
        assert!(std::ptr::eq(branded_key(&plain), plain.as_str()));
        // Which keys are interface strings is known from the catalogs (craft-i18n).
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let en = Lang::EN;
        let text = String::from("My PdfCraft notes.pdf");
        assert!(std::ptr::eq(tr(en, &text), text.as_str()));
        assert!(std::ptr::eq(branded_key(&text), text.as_str()));
    }
}
