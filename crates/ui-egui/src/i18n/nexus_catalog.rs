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
    // Upstream sync 2026-11-14: OCR Verify, the print dialog and other new interface text whose
    // Vietnamese craft-i18n (575cfde) doesn't have yet. Drop a row once craft-i18n has it.
    ("Check for updates when PdfCraft starts", "Kiểm tra cập nhật khi Nexus PDF khởi động"),
    ("Open web and email addresses written in the page text", "Mở địa chỉ web và email viết trong chữ của trang"),
    (
        "Only web (http, https) and email addresses open, and PdfCraft asks first, as for other links.",
        "Chỉ mở địa chỉ web (http, https) và email, và Nexus PDF hỏi trước, như với các liên kết khác.",
    ),
    (
        "PdfCraft's checks now find no problems for {level}. This is not a validator result. Save to keep the changes.",
        "Bước kiểm tra của Nexus PDF nay không thấy lỗi nào cho {level}. Đây không phải kết quả của trình thẩm định. Hãy lưu để giữ các thay đổi.",
    ),
    (
        "PdfCraft's checks found no problems for {level}. This is not a validator result.",
        "Bước kiểm tra của Nexus PDF không thấy lỗi nào cho {level}. Đây không phải kết quả của trình thẩm định.",
    ),
    ("Edit description", "Sửa mô tả"),
    ("Show tab numbers", "Hiện số thứ tự tab"),
    (
        "The document changed. Reopen Document Properties to edit the current version.",
        "Tài liệu đã thay đổi. Mở lại Thuộc tính tài liệu để sửa bản hiện tại.",
    ),
    ("This document's script is asking", "Mã lệnh của tài liệu này đang hỏi"),
    (
        "This request comes from the document, not PdfCraft. Only enter a password if you trust this document.",
        "Yêu cầu này đến từ tài liệu, không phải từ Nexus PDF. Chỉ nhập mật khẩu nếu bạn tin tài liệu này.",
    ),
    ("Inherit Zoom", "Giữ mức thu phóng hiện tại"),
    ("Reading the drawing for snapping…", "Đang đọc bản vẽ để bắt điểm…"),
    (
        "Field names with + - * / and parentheses; put \\ before spaces in names.",
        "Tên trường kèm + - * / và dấu ngoặc; đặt \\ trước dấu cách trong tên.",
    ),
    ("Custom calculation script:", "Mã lệnh tính tùy chỉnh:"),
    (
        "This script is too long to edit here ({kib} KiB); its start is shown. It is kept as it is.",
        "Mã lệnh này quá dài để sửa ở đây ({kib} KiB); chỉ hiện phần đầu. Mã lệnh được giữ nguyên.",
    ),
    (
        "A script edited here can be at most 256 KiB long; that change wasn't made.",
        "Mã lệnh sửa ở đây dài tối đa 256 KiB; thay đổi đó chưa được thực hiện.",
    ),
    ("Engine", "Bộ nhận dạng"),
    ("Built-in", "Có sẵn"),
    ("Tesseract", "Tesseract"),
    ("Combine engines", "Kết hợp các bộ nhận dạng"),
    ("Straighten pages scanned sideways or upside down", "Xoay thẳng trang bị quét nằm ngang hoặc lộn ngược"),
    ("Straighten skewed text lines", "Nắn thẳng dòng chữ bị nghiêng"),
    ("Remove scan speckle", "Xoá đốm bẩn khi quét"),
    ("Black and white (binarize)", "Đen trắng (nhị phân hoá)"),
    ("Skip pages that already have text", "Bỏ qua trang đã có chữ"),
    ("Pages with renderable text are left alone and reported as skipped", "Trang đã có chữ hiển thị được thì để nguyên và báo là đã bỏ qua"),
    ("Skip files that already have text", "Bỏ qua tệp đã có chữ"),
    (
        "In multiple files: a file with any text is its own skipped bucket, and nothing is written for it",
        "Khi chạy nhiều tệp: tệp nào đã có chữ thì xếp vào nhóm bỏ qua và không ghi gì cho tệp đó",
    ),
    ("Force OCR (override skip options)", "Bắt buộc OCR (bỏ qua các tùy chọn bỏ qua)"),
    ("Reads pages and files even where the skip options would leave them alone", "Đọc cả trang và tệp mà các tùy chọn bỏ qua lẽ ra để nguyên"),
    ("The searchable copies are written here under the source names", "Bản tìm kiếm được ghi vào đây, giữ tên tệp gốc"),
    ("A folder is asked for when a multiple-file run starts", "Khi bắt đầu chạy nhiều tệp sẽ hỏi chọn thư mục"),
    (
        "Several files have the same name, so their results would overwrite each other; nothing was written: {list}",
        "Có nhiều tệp trùng tên nên kết quả sẽ ghi đè lên nhau; chưa ghi gì: {list}",
    ),
    (
        "The output folder already holds files of these names; nothing was written: {list}",
        "Thư mục đầu ra đã có tệp mang các tên này; chưa ghi gì: {list}",
    ),
    ("{name}: {n} low-confidence word(s) on pages {pages}", "{name}: {n} từ độ tin cậy thấp ở các trang {pages}"),
    ("; {n} files already had text", "; {n} tệp đã có chữ sẵn"),
    ("; cancelled — {n} files were not run", "; đã huỷ — {n} tệp chưa chạy"),
    ("Re-reading the region…", "Đang đọc lại vùng…"),
    ("Re-reading the page…", "Đang đọc lại trang…"),
    ("Downloading the text recognition models…", "Đang tải mô hình nhận dạng chữ…"),
    ("Couldn't download the text recognition models: {e}", "Không tải được mô hình nhận dạng chữ: {e}"),
    ("The re-read was cancelled; the words are unchanged", "Đã huỷ đọc lại; các từ giữ nguyên"),
    (
        "The region was read again; its words were replaced (undo in the panel)",
        "Đã đọc lại vùng; các từ trong vùng đã được thay (hoàn tác trong bảng)",
    ),
    (
        "The page was read again; its words were replaced (undo in the panel)",
        "Đã đọc lại trang; các từ trên trang đã được thay (hoàn tác trong bảng)",
    ),
    (
        "Cancelled — {n} words were read on {m} of {t} pages; the rest were not read",
        "Đã huỷ — đọc được {n} từ trên {m}/{t} trang; phần còn lại chưa đọc",
    ),
    ("OCR complete — no text recognized on this page.", "OCR xong — không nhận dạng được chữ nào trên trang này."),
    ("Read {n} words on {m} of {t} pages", "Đã đọc {n} từ trên {m}/{t} trang"),
    (
        "Accept or reject the OCR review of the other document before running OCR here",
        "Hãy chấp nhận hoặc từ chối phần soát OCR của tài liệu kia trước khi chạy OCR ở đây",
    ),
    ("Recognized text", "Chữ đã nhận dạng"),
    ("Word", "Từ"),
    ("Source page", "Trang gốc"),
    ("Auto-rotate", "Tự xoay"),
    ("Turns pages scanned sideways or upside down upright before reading", "Xoay thẳng trang bị quét nằm ngang hoặc lộn ngược trước khi đọc"),
    ("Deskew", "Chống nghiêng"),
    ("Straightens skewed text lines before reading", "Nắn thẳng dòng chữ bị nghiêng trước khi đọc"),
    ("Denoise", "Khử nhiễu"),
    ("A 3×3 median filter against scan speckle", "Bộ lọc trung vị 3×3 chống đốm bẩn khi quét"),
    ("Binarize", "Nhị phân hoá"),
    ("Sauvola binarization: ink to black, paper to white", "Nhị phân hoá Sauvola: mực thành đen, giấy thành trắng"),
    ("Run OCR", "Chạy OCR"),
    ("Cancel re-read", "Huỷ đọc lại"),
    ("Accept", "Chấp nhận"),
    ("Reject", "Từ chối"),
    ("Page verified", "Đã soát trang"),
    ("Mark this page as reviewed (Ctrl+T)", "Đánh dấu trang này đã soát (Ctrl+T)"),
    ("Next uncertain", "Từ chưa chắc tiếp theo"),
    ("F8: next word to review; Shift+F8 goes back", "F8: từ cần soát tiếp theo; Shift+F8 để quay lại"),
    ("Previous uncertain", "Từ chưa chắc trước đó"),
    ("Shift+F8: the previous word to review", "Shift+F8: từ cần soát trước đó"),
    ("Primary only", "Chỉ bộ chính"),
    ("Confidence weighted", "Theo độ tin cậy"),
    ("ROVER vote", "Bỏ phiếu ROVER"),
    ("Both engines, combined as the Strategy says", "Cả hai bộ nhận dạng, kết hợp theo Chiến lược"),
    ("The built-in engine (pure Rust)", "Bộ nhận dạng có sẵn (viết bằng Rust)"),
    ("The tesseract program, run as an external process", "Chương trình tesseract, chạy thành tiến trình riêng"),
    ("Use the chosen engine's reading alone", "Chỉ dùng kết quả của bộ nhận dạng đã chọn"),
    ("Add a second engine's reading when the first is unsure", "Thêm kết quả của bộ thứ hai khi bộ thứ nhất chưa chắc"),
    ("Always read with both engines and merge the words", "Luôn đọc bằng cả hai bộ và gộp các từ"),
    ("Editable text", "Chữ sửa được"),
    ("Writes a new document of visible text; the scan is not changed", "Ghi ra tài liệu mới gồm chữ thấy được; bản quét không thay đổi"),
    ("PAGE", "TRANG"),
    ("AVG CONFIDENCE", "ĐỘ TIN CẬY TB"),
    ("LOW-CONFIDENCE WORDS", "TỪ ĐỘ TIN CẬY THẤP"),
    ("VERIFIED %", "% ĐÃ SOÁT"),
    ("ENGINE · STRATEGY", "BỘ NHẬN DẠNG · CHIẾN LƯỢC"),
    ("skipped", "đã bỏ qua"),
    ("verified", "đã soát"),
    ("{n} word(s) to review", "{n} từ cần soát"),
    ("Re-OCR this region", "OCR lại vùng này"),
    ("Re-OCR entire page", "OCR lại cả trang"),
    ("Accept entire page", "Chấp nhận cả trang"),
    ("Reject entire page", "Từ chối cả trang"),
    ("Click a word to inspect it; drag a rectangle to re-read part of the page", "Bấm vào một từ để xem; kéo một khung để đọc lại một phần trang"),
    ("CONFIDENCE", "ĐỘ TIN CẬY"),
    ("HIGH (>= 90%)", "CAO (>= 90%)"),
    ("MEDIUM (70-89%)", "TRUNG BÌNH (70-89%)"),
    ("LOW (< 70%)", "THẤP (< 70%)"),
    ("Select a word on the page to inspect it", "Chọn một từ trên trang để xem"),
    ("The selected word is gone; choose another", "Từ đã chọn không còn; hãy chọn từ khác"),
    ("removed", "đã xoá"),
    ("corrected", "đã sửa"),
    ("unreviewed", "chưa soát"),
    ("Corrected text (Enter commits)", "Chữ đã sửa (Enter để ghi)"),
    ("+ Dictionary", "+ Từ điển"),
    ("Verified", "Đã soát"),
    ("Undo the last review change (Ctrl+Z)", "Hoàn tác thay đổi soát gần nhất (Ctrl+Z)"),
    ("Suggestions", "Gợi ý"),
    ("The page's words were applied as one undoable step ({n} words)", "Đã áp dụng các từ của trang trong một bước hoàn tác được ({n} từ)"),
    ("Recognized text applied ({n} words, one undoable step)", "Đã áp dụng chữ nhận dạng ({n} từ, một bước hoàn tác được)"),
    ("Wrote {name}", "Đã ghi {name}"),
    ("Optimizing… reading the pages", "Đang tối ưu… đọc các trang"),
    ("Optimizing… reading page {d} of {t}", "Đang tối ưu… đọc trang {d}/{t}"),
    ("Discard embedded search index", "Bỏ chỉ mục tìm kiếm nhúng"),
    ("Discard external cross-references", "Bỏ tham chiếu chéo ra ngoài"),
    ("Detecting paper…", "Đang dò khổ giấy…"),
    ("Detecting…", "Đang dò…"),
    (
        "Selects paper size and orientation from the printer driver. For paper tray, print quality or other driver defaults, use Properties…",
        "Chọn khổ giấy và hướng giấy từ trình điều khiển máy in. Muốn chọn khay giấy, chất lượng in hay mặc định khác của trình điều khiển, dùng Thuộc tính…",
    ),
    ("Page setup dialog is not available on this system.", "Máy này không có hộp thoại Thiết lập trang."),
    ("Page Setup…", "Thiết lập trang…"),
    ("by", "×"),
    ("Creep:", "Bù gáy:"),
    (
        "Moves each sheet's pages toward the fold by this much more than the sheet outside it, so the margins match once the booklet is folded and trimmed.",
        "Dời trang của mỗi tờ vào phía nếp gấp nhiều hơn tờ bên ngoài đúng bằng mức này, để lề đều nhau khi gấp và xén cuốn sách nhỏ.",
    ),
    (
        "The creep moves the innermost sheet's pages too far for this many pages. Reduce Creep.",
        "Với số trang này, mức bù gáy đẩy trang của tờ trong cùng đi quá xa. Hãy giảm Bù gáy.",
    ),
    ("Columns and rows must each be 1 to 99.", "Số cột và số hàng mỗi loại phải từ 1 đến 99."),
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
