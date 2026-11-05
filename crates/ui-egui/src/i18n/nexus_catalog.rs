//! NexusOffice: Vietnamese that Nexus PDF shows where craft-i18n's Vietnamese catalog has none,
//! or words it differently on purpose.
//!
//! Upstream moved every interface catalog to craft-i18n (`CRAFT_I18N_DIR`, at the commit in
//! `craft-i18n.rev`) and deleted the in-repo `i18n/*.tsv` files, including the `vi.tsv` the Nexus
//! branches had edited. The Nexus rows live here instead, one overlay over the Vietnamese catalog:
//! [`super::tr`] (and so `tl!`, [`super::tr_ctx`] without a context row of its own, menu labels
//! without an `@id` row) looks a key up here first when the interface is in Vietnamese, then in
//! craft-i18n. Keys are the English source strings, exactly as the code passes them; `{name}`
//! placeholders stay as they are. A row that craft-i18n's Vietnamese catalog gains with the same
//! wording can be removed from here.

use std::collections::HashMap;
use std::sync::LazyLock;

/// (English key, Vietnamese).
const ROWS: &[(&str, &str)] = &[
    // Nexus wording (nx `vi.tsv`, 2026-10): "Lưu thay đổi", not "Lưu các thay đổi" (the save
    // prompt reads shorter and as the other Nexus apps word it).
    ("Save changes?", "Lưu thay đổi?"),
    ("Save changes to “{name}” before closing?", "Lưu thay đổi của “{name}” trước khi đóng?"),
    // craft-i18n e45ddf7..575cfde: PdfCraft's own rows leave these in English, over common.tsv's
    // Vietnamese (the Nexus `vi.tsv` had it).
    ("Less", "Ít hơn"),
    ("Subscript", "Chỉ số dưới"),
    ("Superscript", "Chỉ số trên"),
    // Upstream sync 2026-11-05: interface text whose Vietnamese craft-i18n (575cfde) doesn't have
    // yet. Drop a row once craft-i18n has it.
    ("1 line", "1 dòng"),
    ("{n} lines", "{n} dòng"),
    ("Add Tools to Hotbar", "Thêm công cụ vào thanh công cụ nhanh"),
    ("Background removal", "Làm trắng nền"),
    (
        "Cleans up the scanned pictures in place: recognized text, comments and links on the pages stay as they are.",
        "Làm sạch ảnh quét ngay trên trang: chữ đã nhận dạng, bình luận và liên kết trên các trang được giữ nguyên.",
    ),
    ("Close menu", "Đóng menu"),
    ("Custom scale", "Tỷ lệ tùy chỉnh"),
    ("Despeckle", "Khử đốm"),
    ("Dock navigation rail on Left", "Neo thanh điều hướng bên trái"),
    ("Dock navigation rail on Right", "Neo thanh điều hướng bên phải"),
    ("Drag hotbar", "Kéo thanh công cụ nhanh"),
    ("Edge shadow removal", "Khử bóng mép"),
    ("Enhance", "Nâng cao chất lượng"),
    ("Enhance Scanned File", "Nâng cao chất lượng tệp quét"),
    ("Enhance scanned file…", "Nâng cao chất lượng tệp quét…"),
    ("Enhancing scans… image {d} of {t}", "Đang nâng cao chất lượng ảnh quét… ảnh {d} trên {t}"),
    (
        "Evens out the paper so a grey or yellowed background turns white, and darkens faint strokes. 0 is off.",
        "Làm đều màu giấy để nền xám hoặc ố vàng thành trắng, và làm đậm nét mờ. 0 là tắt.",
    ),
    (
        "Every line set in the style chosen for a level becomes a bookmark of that level, under a new first bookmark.",
        "Mỗi dòng có kiểu chữ đã chọn cho một cấp sẽ thành dấu trang ở cấp đó, nằm dưới một dấu trang đầu tiên mới.",
    ),
    (
        "Extend the docked toolbar along the entire side. Choose a side above to enable.",
        "Kéo dài thanh công cụ đã neo dọc hết cạnh. Chọn một cạnh ở trên để bật.",
    ),
    ("Float", "Thả nổi"),
    ("Freehand-Highlight", "Tô sáng tự do"),
    ("Full-side strip", "Dải hết cạnh"),
    ("General & Interface", "Chung và giao diện"),
    ("Hotbar settings", "Cài đặt thanh công cụ nhanh"),
    ("Hotbar settings / add tools", "Cài đặt thanh công cụ nhanh / thêm công cụ"),
    ("Hotbar tools", "Công cụ trên thanh công cụ nhanh"),
    ("Invert middle-button drag", "Đảo chiều kéo bằng nút giữa"),
    ("Layout & Sizing", "Bố cục và kích thước"),
    ("Level {n}", "Cấp {n}"),
    (
        "Lightens shadows along the edges of the scan, such as a book's gutter or a copier's dark border",
        "Làm sáng bóng tối dọc mép ảnh quét, như gáy sách hoặc viền tối của máy photo",
    ),
    ("Navigation rail side", "Cạnh đặt thanh điều hướng"),
    ("New bookmarks from text styles", "Dấu trang mới từ kiểu chữ"),
    ("New bookmarks from text styles…", "Dấu trang mới từ kiểu chữ…"),
    ("No scanned pages were cleaned up.", "Không có trang quét nào được làm sạch."),
    ("Page Properties", "Thuộc tính trang"),
    ("Page properties…", "Thuộc tính trang…"),
    (
        "Panning or auto-scrolling with the middle (wheel) button moves the page the other way.",
        "Kéo hoặc tự cuộn bằng nút giữa (con lăn) sẽ di chuyển trang theo chiều ngược lại.",
    ),
    ("Paste bookmarks", "Dán dấu trang"),
    ("Remove from hotbar", "Gỡ khỏi thanh công cụ nhanh"),
    (
        "Removes isolated dark specks such as dust and scanner noise. The only filter black and white scans take.",
        "Xóa các đốm tối rời rạc như bụi và nhiễu của máy quét. Đây là bộ lọc duy nhất áp dụng cho bản quét đen trắng.",
    ),
    ("Reset side panels", "Đặt lại các bảng bên"),
    ("Search all tools…", "Tìm trong mọi công cụ…"),
    ("Search settings…", "Tìm trong cài đặt…"),
    ("Show or hide the Tools panel", "Hiện hoặc ẩn bảng Công cụ"),
    ("Show or hide the side panel", "Hiện hoặc ẩn bảng bên"),
    ("Stick to a side:", "Gắn vào một cạnh:"),
    ("Tab Order", "Thứ tự tab"),
    ("This document has no text to make bookmarks from.", "Tài liệu này không có chữ để tạo dấu trang."),
    ("Toolbar size", "Cỡ thanh công cụ"),
    ("Unspecified", "Chưa chỉ định"),
    ("Use Column Order", "Theo thứ tự cột"),
    ("Use Document Structure", "Theo cấu trúc tài liệu"),
    ("Use Row Order", "Theo thứ tự hàng"),
    ("Vertical layout", "Bố cục dọc"),
    ("bold", "đậm"),
    ("italic", "nghiêng"),
];

static MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| ROWS.iter().copied().collect());

/// The Nexus Vietnamese for `key` when `lang` (a language code) is Vietnamese.
pub(super) fn get(lang: &str, key: &str) -> Option<&'static str> {
    if lang == "vi" { MAP.get(key).copied() } else { None }
}

#[cfg(test)]
mod tests {
    use super::ROWS;
    use crate::i18n::{Lang, current, has, menu_label, set_current, t, tr, tr_ctx};

    /// With the Vietnamese catalog embedded, every Nexus row shows in Vietnamese through the
    /// interface lookups (`tl!`, contexts, `has`), English stays English, and craft-i18n's own
    /// rows still come from there.
    #[test]
    fn nexus_rows_show_in_a_vietnamese_interface() {
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let Some(vi) = Lang::from_code("vi") else { panic!("the Vietnamese catalog is embedded") };
        let before = current();
        set_current(vi);
        for (en, want) in ROWS {
            assert_eq!(tr(vi, en), *want, "{en:?}");
            assert_eq!(t(en), *want, "{en:?} through tl!");
            assert_eq!(tr_ctx(vi, "no such context", en), *want, "{en:?} with a context without its own row");
            assert!(has(vi, en), "{en:?} counts as translated");
        }
        // A command without an `@id` row of its own: its menu label is the overlay's.
        assert_eq!(menu_label("no.such.command", "Paste bookmarks"), "Dán dấu trang");
        set_current(Lang::EN);
        for (en, _) in ROWS {
            assert_eq!(t(en), *en, "English keeps {en:?}");
        }
        set_current(before);
        assert_eq!(tr(vi, "Close"), "Đóng", "a row of craft-i18n's own catalog");
    }

    /// Nexus PDF's own words in a Vietnamese interface, with the catalogs embedded: the save
    /// prompt as Nexus words it, the product named Nexus PDF (never PdfCraft) in craft-i18n's
    /// translations and in English keys, and the export labels naming the format.
    #[test]
    fn nexus_strings_show_in_vietnamese_with_the_nexus_brand() {
        if !crate::i18n::needs_catalogs() {
            return;
        }
        let Some(vi) = Lang::from_code("vi") else { panic!("the Vietnamese catalog is embedded") };
        assert_eq!(tr(vi, "Save changes?"), "Lưu thay đổi?");
        assert_eq!(tr(vi, "Save changes to “{name}” before closing?"), "Lưu thay đổi của “{name}” trước khi đóng?");
        let about = tr(vi, "About PdfCraft");
        assert!(about.starts_with("Giới thiệu") && about.contains("Nexus PDF") && !about.contains("PdfCraft"), "{about}");
        let welcome = tr(vi, "Welcome to PdfCraft");
        assert!(welcome.contains("Nexus PDF") && !welcome.contains("PdfCraft"), "{welcome}");
        assert_eq!(tr(vi, "Export to Word…"), "Xuất ra DOCX…");
        assert_eq!(tr(Lang::EN, "About PdfCraft"), "About Nexus PDF");
    }

    /// Each key once, each translation non-empty, with the key's `{placeholders}` and trailing `…`.
    #[test]
    fn rows_are_unique_and_keep_their_placeholders() {
        let mut seen = std::collections::HashSet::new();
        for (en, vi) in ROWS {
            assert!(seen.insert(*en), "{en:?} is listed twice");
            assert!(!vi.trim().is_empty(), "{en:?} has no Vietnamese");
            let holes = |s: &str| {
                let mut v: Vec<String> = s.split('{').skip(1).filter_map(|p| p.split_once('}').map(|(h, _)| h.to_string())).collect();
                v.sort();
                v
            };
            assert_eq!(holes(en), holes(vi), "{en:?}: placeholders differ");
            assert_eq!(en.ends_with('…'), vi.ends_with('…'), "{en:?}: a trailing … in one needs one in the other");
        }
    }
}
