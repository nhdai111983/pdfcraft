//! NexusOffice layout in the style of WPS Office (its "Universal UI"; Henry's approved mockup,
//! 2026-10-24), the same layout as Nexus Docs' and Nexus Sheet's so the suite's tabs feel like one
//! product. Only the arrangement is borrowed: every icon is a Lucide icon the app already ships
//! (`icons.rs`) or drawn in code here, the colours are the app's own (`theme.rs`), and the command
//! names come from the app's command registry.
//!
//! [`PdfCraftApp::wps`] turns it on. The window then has:
//!
//! - [`bar`] (36 pt): File (Tệp, opens the light File page, `nexus_backstage`), the Quick Access
//!   Toolbar (Save, Print, Undo, Redo), the tool tabs left aligned (accent, semibold, a 2 pt
//!   underline, » for the ones that don't fit); on the right the save state, the command search
//!   and a "⋯" menu (theme, Preferences, shortcuts, About).
//! - [`strip`] (56 pt): the current tab's groups as one flat row, two rows of small buttons high
//!   (a large button is a 20 px icon over one label line). Groups the window has no room for fold
//!   into a "…" panel at the end of the row; their buttons work there.
//! - On the left a rail of navigation panels (page thumbnails, bookmarks, attachments…) and the
//!   chosen one beside it; on the right the comments or search pane and the current tool's pane;
//!   the document in the middle; a status bar (page n/N, zoom) at the bottom.
//!
//! [`PdfCraftApp::embedded`] is for a host window with tabs of its own (the NexusOffice window):
//! the app leaves the window title alone, draws no document tab strip, and hands new and opened
//! documents to the host through [`Host`].

use std::sync::Arc;
use std::sync::mpsc::Receiver;

use egui::{Align2, Color32, Id, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

use crate::canvas::Fit;
use crate::theme::{self, Tokens};
use crate::{LeftPanel, Mode, PdfCraftApp, QuickTool, RightPanel, icons};

/// Height of the top row (File, Quick Access, tabs, save state, search).
pub const BAR_H: f32 = 36.0;
/// Height of the command strip: two rows of small buttons, or a large button.
pub const STRIP_H: f32 = 56.0;
/// Height of the status bar.
pub const STATUS_H: f32 = 28.0;
/// Height of the top row's buttons and search box.
const CONTROL_H: f32 = 28.0;
/// A small button's height: two of them and the gap fill the strip's content.
pub const SMALL_H: f32 = 24.0;
/// The strip's content height (two small rows, 2 pt apart).
const CONTENT_H: f32 = SMALL_H * 2.0 + 2.0;
// Two rows of small buttons fit the strip, with 3 pt above and below.
const _: () = assert!(CONTENT_H + 6.0 <= STRIP_H);
/// Width kept at the end of the strip for the "…" button.
const OVERFLOW_W: f32 = 30.0;
/// Width of the » button holding the tabs the top row has no room for.
const MORE_TABS_W: f32 = 26.0;
/// Below this window width the search box shows its magnifier alone.
const NARROW_W: f32 = 1200.0;
/// The navigation rail's width.
const RAIL_W: f32 = 40.0;

/// A file dialog the app asks the host to show (NexusOffice: the system's dialog, shown without
/// freezing the window). The answer comes later on the returned channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogRequest {
    /// Pick a file to open; `purpose` says what for (`pdf`).
    Open { purpose: String },
    /// Pick where to save, given a suggested file name and the types to offer (extensions; Save
    /// offers `pdf` only).
    Save { name: String, types: Vec<String> },
}

/// The host's answer: the picked path, `Ok(None)` when cancelled, or why no dialog could be shown.
pub type FileDialogAnswer = Result<Option<String>, String>;

/// [`Host::open_document`].
pub type OpenDocumentFn = Box<dyn Fn(&str)>;
/// [`Host::adopt_document`]: `Err(why)` when the host can't take the document now (it then stays
/// in this app, kept, and the user is told).
///
/// A host that takes it must answer with [`PdfCraftApp::adoption_answered`] **in the same frame**
/// it was handed the document, or a few frames later at most (NexusOffice opens the new tab when it
/// handles the tab's request, one request a frame). An answer that comes after
/// [`HANDOVER_FRAMES`] frames is late: the document was kept here meanwhile. A late `Ok` is logged
/// as a warning; the kept copy then closes only if it is unchanged since it was handed over, with
/// nothing being typed in it either (the new tab has it as it is), otherwise both stay and the kept one gets a recovery copy of its
/// own, so the two tabs never write over one copy.
pub type AdoptDocumentFn = Box<dyn Fn(AdoptedDocument) -> Result<(), String>>;

/// A document this app hands to the host for a tab of its own, with what makes it the same
/// document there: its file (Save writes back to it) and its crash-recovery copy (kept on disk
/// across the handoff, so a crash right after loses nothing).
#[derive(Clone)]
pub struct AdoptedDocument {
    pub name: String,
    pub path: Option<String>,
    pub bytes: Arc<Vec<u8>>,
    /// The recovery store entry holding this document's last snapshot, if it has one.
    pub recovery_key: Option<String>,
    /// The document in the app that handed it over, which keeps it until the host answers with
    /// [`PdfCraftApp::adoption_answered`] (`None` for one a host hands in: nothing to answer).
    pub source: Option<pdfcraft_engine::DocId>,
}

impl std::fmt::Debug for AdoptedDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdoptedDocument")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("bytes", &self.bytes.len())
            .field("recovery_key", &self.recovery_key)
            .field("source", &self.source)
            .finish()
    }
}
/// [`Host::file_dialog`].
pub type FileDialogFn = Box<dyn Fn(FileDialogRequest) -> Receiver<FileDialogAnswer>>;

/// What a host window with tabs (NexusOffice) does for the embedded app. Every hook is optional;
/// without one the app behaves as it does stand-alone.
#[derive(Default)]
pub struct Host {
    /// File › New (a blank PDF), Ctrl+N: the host opens a new tab; this document stays.
    pub new_document: Option<Box<dyn Fn()>>,
    /// A PDF picked in Open (or a recent file, or a dropped one) while this app shows a document:
    /// the host opens it in a tab of its own, or brings that tab forward.
    pub open_document: Option<OpenDocumentFn>,
    /// A document with unsaved changes this app made or recovered (from images, the clipboard,
    /// Combine, Extract, File › Recover…), for a tab of its own. The app keeps its copy, out of
    /// sight, until the host answers with [`PdfCraftApp::adoption_answered`]: closed once the new
    /// tab opened (its recovery copy stays on disk for it), kept and shown here if it didn't, or
    /// if the hook refused it, or if no answer came within [`HANDOVER_FRAMES`] frames.
    pub adopt_document: Option<AdoptDocumentFn>,
    /// Open and Save As dialogs answered later (wins over the app's own pickers for File › Open
    /// and Save).
    pub file_dialog: Option<FileDialogFn>,
}

/// A label of the NexusOffice layout in the interface language: the fork's own Vietnamese
/// ([`NEXUS_VI`]) in a Vietnamese interface, else the shared catalogs (labels upstream already
/// translates), else English. The fork's labels live here rather than in the catalogs: upstream's
/// tests require every catalog key in all 22 languages, and the fork doesn't edit the others.
pub(crate) fn tx(s: &str) -> &str {
    if crate::i18n::current().code() == "vi"
        && let Some((_, vi)) = NEXUS_VI.iter().find(|(en, _)| *en == s)
    {
        return vi;
    }
    crate::i18n::t(s)
}

/// Said once when a document can't move to a tab of its own (`{name}`, why: `{e}`).
const KEPT_NOTICE: &str = "“{name}” couldn't move to a tab of its own, so it stays in this one: {e}";

/// Why a stray document couldn't move, when it isn't its password: its bytes couldn't be written,
/// or they don't open again as they are (the engine's own reason, in English, goes to the log).
const STRAY_UNREADY: &str = "it couldn't be prepared for a new tab";

/// Why a handed-over document stayed: the host never said whether its new tab opened.
const NO_ANSWER: &str = "the window didn't open a tab for it";

/// Said when a document kept here (its handover went unanswered) opened in a tab of its own late,
/// unchanged: the copy here closes.
const LATE_MOVED: &str = "“{name}” opened in a tab of its own after all, so it was closed here.";

/// Said when a document kept here opened in a tab of its own late but was changed here since:
/// both stay, each with its own recovery copy.
const LATE_BOTH: &str = "“{name}” also opened in a tab of its own, but it was changed here since, so both are kept: check which one to save.";

/// [`LATE_BOTH`], when the kept one's own recovery copy couldn't be written (`{e}`: why).
const LATE_BOTH_NO_COPY: &str = "“{name}” also opened in a tab of its own, but it was changed here since, so both are kept. The copy here has no recovery copy of its own yet ({e}): save it soon.";

/// Frames a handed-over document waits for the host's answer before it counts as refused (a
/// host that never answers would otherwise keep the tab from closing for good).
pub const HANDOVER_FRAMES: u32 = 120;

/// Said when Save stops because the paragraph open in the text editor couldn't be applied.
pub(crate) const DRAFT_NOT_SAVED: &str =
    "The paragraph being edited couldn't be applied, so nothing was saved. It is still open: change it or press Esc to drop it, then save again.";

/// Vietnamese for the labels only the NexusOffice layout has.
pub(crate) const NEXUS_VI: &[(&str, &str)] = &[
    (
        "“{name}” couldn't move to a tab of its own, so it stays in this one: {e}",
        "Không chuyển được “{name}” sang tab riêng nên tài liệu ở lại tab này: {e}",
    ),
    ("it needs its password", "tài liệu cần mật khẩu"),
    ("it couldn't be prepared for a new tab", "không chuẩn bị được tài liệu để mở ở tab mới"),
    ("the window didn't open a tab for it", "cửa sổ không mở tab mới cho tài liệu"),
    ("“{name}” opened in a tab of its own after all, so it was closed here.", "“{name}” đã mở được ở tab riêng nên được đóng ở tab này."),
    (
        "“{name}” also opened in a tab of its own, but it was changed here since, so both are kept: check which one to save.",
        "“{name}” cũng đã mở ở tab riêng, nhưng bản ở tab này đã được sửa sau đó nên giữ cả hai: hãy xem lại nên lưu bản nào.",
    ),
    (
        "“{name}” also opened in a tab of its own, but it was changed here since, so both are kept. The copy here has no recovery copy of its own yet ({e}): save it soon.",
        "“{name}” cũng đã mở ở tab riêng, nhưng bản ở tab này đã được sửa sau đó nên giữ cả hai. Bản ở tab này chưa ghi được bản khôi phục riêng ({e}): hãy lưu sớm.",
    ),
    (
        "The paragraph being edited couldn't be applied, so nothing was saved. It is still open: change it or press Esc to drop it, then save again.",
        "Không áp dụng được đoạn văn đang sửa nên chưa lưu gì. Đoạn văn vẫn đang mở: hãy sửa lại hoặc nhấn Esc để bỏ, rồi lưu lại.",
    ),
    ("Also open in this tab:", "Cũng mở trong tab này:"),
    ("A PNG or JPEG for each page", "Mỗi trang một ảnh PNG hoặc JPEG"),
    ("A picture or a text file as a PDF", "Một ảnh hoặc tệp văn bản thành PDF"),
    ("A slide for each page", "Mỗi trang một trang chiếu"),
    ("All images", "Mọi hình ảnh"),
    ("Alt text", "Văn bản thay thế"),
    ("Ask for a password to open or change it", "Đặt mật khẩu để mở hoặc để sửa"),
    ("Auto tag", "Tự gắn thẻ"),
    ("Blank PDF", "PDF trống"),
    ("Bookmarks from structure", "Dấu trang theo cấu trúc"),
    ("Camera image", "Ảnh chụp"),
    ("Choose a PDF on this computer", "Chọn một tệp PDF trên máy này"),
    ("Clear marks", "Xóa dấu"),
    ("Continuous", "Cuộn liên tục"),
    ("Cover page", "Trang bìa"),
    ("Crop", "Cắt trang"),
    ("Detect fields", "Nhận diện trường"),
    ("Drop-down", "Danh sách thả xuống"),
    ("Edit the text in Nexus Docs or Word", "Sửa chữ trong Nexus Docs hoặc Word"),
    ("Fields panel", "Bảng trường"),
    ("Find", "Tìm"),
    ("Find & redact", "Tìm và che"),
    ("Fit page", "Vừa trang"),
    ("Fit width", "Vừa chiều rộng"),
    ("Flatten fields", "Làm phẳng trường"),
    ("From a file…", "Từ một tệp…"),
    ("From clipboard", "Từ bộ nhớ tạm"),
    ("From file", "Từ tệp"),
    ("From images", "Từ ảnh"),
    ("From images…", "Từ ảnh…"),
    ("From the clipboard", "Từ bộ nhớ tạm"),
    ("Full screen", "Toàn màn hình"),
    ("Go back to the last saved version", "Quay về bản đã lưu gần nhất"),
    ("HTML to open in a browser", "HTML để mở bằng trình duyệt"),
    ("Hand", "Bàn tay"),
    ("Hand tool", "Công cụ bàn tay"),
    ("Header & footer", "Đầu trang và chân trang"),
    ("Hide all", "Ẩn tất cả"),
    ("Images…", "Hình ảnh…"),
    ("JavaScript console", "Bảng điều khiển JavaScript"),
    ("Links from URLs", "Liên kết từ địa chỉ web"),
    ("Make the file smaller to send", "Làm tệp nhỏ hơn để gửi đi"),
    ("Merge data", "Gộp dữ liệu"),
    ("More commands", "Thêm lệnh"),
    ("More tabs", "Thêm thẻ"),
    ("New", "Mới"),
    ("No restrictions", "Không hạn chế"),
    ("Not saved", "Chưa lưu"),
    ("Not saved yet", "Chưa được lưu"),
    ("OCR", "Nhận dạng chữ"),
    ("OCR many files", "Nhận dạng nhiều tệp"),
    ("One empty page, in a new tab", "Một trang trống, trong thẻ mới"),
    ("One full page", "Trọn một trang"),
    ("Only the words", "Chỉ phần chữ"),
    ("Page boxes", "Khung trang"),
    ("Plain text…", "Văn bản thuần…"),
    ("PowerPoint presentation…", "Bản trình chiếu PowerPoint…"),
    ("Prepare form", "Soạn biểu mẫu"),
    ("Properties", "Thuộc tính"),
    ("RTF for older word processors", "RTF cho các trình soạn thảo cũ"),
    ("Reading options", "Tùy chọn đọc"),
    ("Recent files", "Tệp gần đây"),
    ("Reduce size", "Giảm dung lượng"),
    ("Remove hidden info", "Xóa thông tin ẩn"),
    ("Remove links", "Xóa liên kết"),
    ("Rich Text Format…", "Định dạng RTF…"),
    ("Rotate left", "Xoay trái"),
    ("Rotate right", "Xoay phải"),
    ("Rotate…", "Xoay…"),
    ("Saved", "Đã lưu"),
    ("Scale", "Tỉ lệ"),
    ("Select", "Chọn"),
    ("Select text and images", "Chọn chữ và hình ảnh"),
    ("Several files into one PDF", "Nhiều tệp thành một PDF"),
    ("Several pictures, a page each", "Nhiều ảnh, mỗi ảnh một trang"),
    ("Signature field", "Trường chữ ký"),
    ("Snapshot", "Chụp vùng"),
    ("Title, author, fonts, initial view", "Tiêu đề, tác giả, phông chữ, chế độ xem ban đầu"),
    ("To HTML", "Sang HTML"),
    ("To PowerPoint", "Sang PowerPoint"),
    ("To RTF", "Sang RTF"),
    ("To Word", "Sang Word"),
    ("To image", "Sang ảnh"),
    ("To text", "Sang văn bản"),
    ("Two pages", "Hai trang"),
    ("Web page…", "Trang web…"),
    ("What you copied, as a PDF", "Nội dung đã sao chép, thành PDF"),
    ("Word document…", "Tài liệu Word…"),
    ("No file dialog could be shown: {e}", "Không mở được hộp chọn tệp: {e}"),
];

/// The tool tabs of the strip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WpsTab {
    #[default]
    Home,
    Edit,
    Comment,
    Pages,
    Convert,
    Forms,
    Protect,
    Sign,
    View,
    Tools,
}

impl WpsTab {
    pub const ALL: [WpsTab; 10] = [
        WpsTab::Home,
        WpsTab::Edit,
        WpsTab::Comment,
        WpsTab::Pages,
        WpsTab::Convert,
        WpsTab::Forms,
        WpsTab::Protect,
        WpsTab::Sign,
        WpsTab::View,
        WpsTab::Tools,
    ];

    /// The tab's English name (the catalog key).
    pub fn label(self) -> &'static str {
        match self {
            WpsTab::Home => "Home",
            WpsTab::Edit => "Edit",
            WpsTab::Comment => "Comment",
            WpsTab::Pages => "Pages",
            WpsTab::Convert => "Convert",
            WpsTab::Forms => "Forms",
            WpsTab::Protect => "Protect",
            WpsTab::Sign => "Sign",
            WpsTab::View => "View",
            WpsTab::Tools => "Tools",
        }
    }
}

/// One button of the strip: a registry command id (or one of the `nx.*` ids below, handled here)
/// with a short label for the strip (the registry's label is the tooltip).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: &'static str,
    pub label: &'static str,
    /// A large button (icon over its label) instead of a small one (icon beside it).
    pub big: bool,
}

const fn big(id: &'static str, label: &'static str) -> Item {
    Item { id, label, big: true }
}

const fn small(id: &'static str, label: &'static str) -> Item {
    Item { id, label, big: false }
}

/// The ids the strip handles itself (no registry command does these).
const LOCAL: [(&str, &str, &str); 12] = [
    ("nx.select", "mouse-pointer-2", "Select text and images"),
    ("nx.hand", "hand", "Hand tool"),
    ("nx.zoom_in", "zoom-in", "Zoom in"),
    ("nx.zoom_out", "zoom-out", "Zoom out"),
    ("nx.actual_size", "scan", "Actual size"),
    ("nx.fit_width", "maximize", "Fit to width"),
    ("nx.fit_page", "maximize-2", "Zoom to page level"),
    ("nx.rotate_view", "rotate-cw", "Rotate view clockwise"),
    ("nx.panel.pages", "files", "Page thumbnails"),
    ("nx.panel.bookmarks", "bookmark", "Bookmarks"),
    ("nx.panel.attachments", "paperclip", "Attachments"),
    ("nx.panel.layers", "layers", "Layers"),
];

/// The groups of `tab`, in order. Each group is drawn as columns: a large button alone, small
/// ones two to a column.
pub fn groups(tab: WpsTab) -> &'static [&'static [Item]] {
    match tab {
        WpsTab::Home => {
            const G: &[&[Item]] = &[
                &[big("nx.hand", "Hand"), big("nx.select", "Select")],
                &[
                    small("nx.zoom_in", "Zoom in"),
                    small("nx.zoom_out", "Zoom out"),
                    small("nx.fit_width", "Fit width"),
                    small("nx.fit_page", "Fit page"),
                ],
                &[big("edit.edit_text", "Edit text"), big("comment.highlight", "Highlight"), big("comment.note", "Note")],
                &[big("page.organize", "Organize"), small("page.rotate", "Rotate"), small("page.delete", "Delete")],
                &[big("export.docx", "To Word"), big("ocr.recognize", "OCR")],
                &[big("edit.find", "Find"), big("sign.fill.signature", "Sign")],
            ];
            G
        }
        WpsTab::Edit => {
            const G: &[&[Item]] = &[
                &[big("edit.edit_text", "Edit text"), big("edit.text", "Add text"), big("edit.image", "Add image"), big("edit.link", "Link")],
                &[
                    small("edit.header_footer", "Header & footer"),
                    small("edit.watermark", "Watermark"),
                    small("edit.background", "Background"),
                    small("edit.bates", "Bates numbering"),
                ],
                &[small("edit.links_from_urls", "Links from URLs"), small("edit.remove_links", "Remove links")],
                &[big("page.crop", "Crop"), small("page.number", "Number pages"), small("page.boxes", "Page boxes")],
                &[small("file.properties", "Properties"), small("file.word_count", "Word count")],
            ];
            G
        }
        WpsTab::Comment => {
            const G: &[&[Item]] = &[
                &[big("comment.note", "Note"), big("comment.freetext", "Text box")],
                &[
                    small("comment.highlight", "Highlight"),
                    small("comment.underline", "Underline"),
                    small("comment.strikeout", "Strikethrough"),
                    small("comment.squiggly", "Squiggly"),
                    small("comment.caret", "Insert text"),
                    small("comment.replace", "Replace text"),
                ],
                &[
                    small("comment.ink", "Draw"),
                    small("comment.line", "Line"),
                    small("comment.arrow", "Arrow"),
                    small("comment.square", "Rectangle"),
                    small("comment.circle", "Oval"),
                    small("comment.cloud", "Cloud"),
                    small("comment.polygon", "Polygon"),
                    small("comment.callout", "Callout"),
                ],
                &[big("comment.stamp", "Stamp"), small("comment.attach", "Attach file"), small("comment.eraser", "Eraser")],
                &[
                    big("comment.list", "Comment list"),
                    small("comment.summarize", "Summarize"),
                    small("comment.hide_all", "Hide all"),
                    small("comment.import", "Import"),
                    small("comment.export", "Export"),
                    small("comment.flatten", "Flatten"),
                ],
            ];
            G
        }
        WpsTab::Pages => {
            const G: &[&[Item]] = &[
                &[
                    big("page.organize", "Organize"),
                    big("page.insert", "Insert"),
                    small("page.insert_blank", "Blank page"),
                    small("page.duplicate", "Duplicate"),
                    small("page.delete", "Delete"),
                ],
                &[small("page.rotate", "Rotate right"), small("page.rotate_ccw", "Rotate left"), small("page.rotate_dialog", "Rotate…")],
                &[big("page.extract", "Extract"), big("page.split", "Split"), big("page.combine", "Combine"), small("page.replace", "Replace")],
                &[small("page.copy", "Copy pages"), small("page.cut", "Cut pages"), small("page.paste", "Paste pages")],
                &[
                    small("page.number", "Number pages"),
                    small("page.crop", "Crop"),
                    small("bookmark.add", "New bookmark"),
                    small("bookmark.from_structure", "Bookmarks from structure"),
                ],
            ];
            G
        }
        WpsTab::Convert => {
            const G: &[&[Item]] = &[
                &[
                    big("export.docx", "To Word"),
                    big("export.pptx", "To PowerPoint"),
                    big("export.image", "To image"),
                    small("export.text", "To text"),
                    small("export.html", "To HTML"),
                    small("export.rtf", "To RTF"),
                    small("export.all_images", "All images"),
                ],
                &[
                    big("create.file", "From file"),
                    small("create.images", "From images"),
                    small("create.clipboard", "From clipboard"),
                    small("create.multiple", "Multiple files"),
                ],
                &[big("ocr.recognize", "OCR"), small("ocr.recognize_batch", "OCR many files"), small("ocr.enhance_camera", "Camera image")],
                &[big("optimize.reduce", "Reduce size"), small("optimize.advanced", "Optimize"), small("standards.pdfa", "PDF/A…")],
            ];
            G
        }
        WpsTab::Forms => {
            const G: &[&[Item]] = &[
                &[big("form.prepare", "Prepare form"), big("form.detect", "Detect fields")],
                &[
                    small("form.add.text", "Text field"),
                    small("form.add.checkbox", "Checkbox"),
                    small("form.add.radio", "Radio button"),
                    small("form.add.combo", "Drop-down"),
                    small("form.add.list", "List box"),
                    small("form.add.date", "Date field"),
                    small("form.add.button", "Button"),
                    small("form.add.signature", "Signature field"),
                ],
                &[small("form.clear", "Clear form"), small("form.flatten", "Flatten fields"), small("form.fields", "Fields panel")],
                &[small("form.import_data", "Import data"), small("form.export_data", "Export data"), small("form.merge_data", "Merge data")],
            ];
            G
        }
        WpsTab::Protect => {
            const G: &[&[Item]] = &[
                &[
                    big("protect.password", "Password"),
                    small("protect.remove", "Remove security"),
                    small("protect.properties", "Security properties"),
                ],
                &[
                    big("redact.mark", "Redact"),
                    big("redact.search", "Find & redact"),
                    small("redact.pages", "Redact pages"),
                    small("redact.clear", "Clear marks"),
                ],
                &[big("redact.apply", "Apply"), small("protect.remove_hidden", "Remove hidden info"), small("redact.sanitize", "Sanitize")],
            ];
            G
        }
        WpsTab::Sign => {
            const G: &[&[Item]] = &[
                &[
                    big("sign.fill.signature", "Sign"),
                    big("sign.fill.initials", "Initials"),
                    small("sign.fill.text", "Add text"),
                    small("sign.fill.date", "Date"),
                    small("sign.fill.check", "Check"),
                    small("sign.fill.cross", "Cross"),
                ],
                &[big("sign.digital", "Digitally sign"), small("sign.certify", "Certify"), small("sign.validate", "Validate all")],
                &[big("sign.panel", "Signatures")],
            ];
            G
        }
        WpsTab::View => {
            const G: &[&[Item]] = &[
                &[
                    small("view.layout.single", "Single page"),
                    small("view.layout.two_up", "Two pages"),
                    small("view.continuous", "Continuous"),
                    small("view.layout.cover", "Cover page"),
                ],
                &[
                    small("view.fit_width_scrolling", "Fit width"),
                    small("view.fit_one_page", "One full page"),
                    small("view.fit_visible", "Fit visible"),
                    small("view.marquee_zoom", "Marquee zoom"),
                    small("nx.actual_size", "Actual size"),
                    small("nx.rotate_view", "Rotate view"),
                ],
                &[big("view.read_mode", "Read mode"), big("view.full_screen", "Full screen")],
                &[
                    small("nx.panel.pages", "Thumbnails"),
                    small("nx.panel.bookmarks", "Bookmarks"),
                    small("nx.panel.attachments", "Attachments"),
                    small("nx.panel.layers", "Layers"),
                ],
                &[small("view.previous_view", "Previous view"), small("view.next_view", "Next view")],
            ];
            G
        }
        WpsTab::Tools => {
            const G: &[&[Item]] = &[
                &[
                    big("a11y.check", "Accessibility"),
                    small("a11y.autotag", "Auto tag"),
                    small("a11y.alt_text", "Alt text"),
                    small("a11y.reading_options", "Reading options"),
                    small("a11y.tags", "Tags"),
                ],
                &[
                    small("measure.distance", "Distance"),
                    small("measure.perimeter", "Perimeter"),
                    small("measure.area", "Area"),
                    small("measure.scale", "Scale"),
                ],
                &[big("doc.compare", "Compare"), big("actions.wizard", "Actions")],
                &[small("tools.js_console", "JavaScript console"), small("tools.document_js", "Document JavaScripts")],
            ];
            G
        }
    }
}

/// The icon and the tooltip (English, translated when shown) of an item's command.
fn icon_and_tip(id: &str) -> (&'static str, &'static str) {
    if let Some((_, icon, tip)) = LOCAL.iter().find(|(local, _, _)| *local == id) {
        return (icon, tip);
    }
    match pdfcraft_engine::commands::command(id) {
        Some(spec) => (if icons::exists(spec.icon) { spec.icon } else { "square" }, spec.label),
        None => ("square", ""),
    }
}

/// The File (Tệp) button.
pub fn file_button_id() -> Id {
    Id::new("nexus_wps_file")
}
/// The strip's "…" button, shown when groups don't fit.
pub fn overflow_button_id() -> Id {
    Id::new("nexus_wps_overflow")
}
/// The save state ("Đã lưu" / "Chưa lưu"), which opens Save and Save As.
pub fn save_state_id() -> Id {
    Id::new("nexus_wps_save_state")
}
/// The top row's "⋯" menu.
pub fn more_button_id() -> Id {
    Id::new("nexus_wps_more")
}
/// The command search box.
pub fn search_id() -> Id {
    Id::new("nexus_wps_search")
}
/// The » button with the tabs that don't fit.
pub fn more_tabs_id() -> Id {
    Id::new("nexus_wps_more_tabs")
}

impl PdfCraftApp {
    /// Runs a strip item: an `nx.*` id here, anything else through the command registry.
    pub fn run_wps_item(&mut self, id: &str) {
        let view = self.active.and_then(|i| self.views.get_mut(i));
        match (id, view) {
            ("nx.select", _) => self.quick_tool = QuickTool::Select,
            ("nx.hand", _) => self.quick_tool = QuickTool::Hand,
            ("nx.zoom_in", Some(v)) => v.zoom_step(true),
            ("nx.zoom_out", Some(v)) => v.zoom_step(false),
            ("nx.actual_size", Some(v)) => v.set_zoom(1.0),
            ("nx.fit_width", Some(v)) => v.set_fit(Fit::Width),
            ("nx.fit_page", Some(v)) => v.set_fit(Fit::Page),
            ("nx.rotate_view", Some(v)) => v.rotate_view(true),
            (panel, _) if panel_for(panel).is_some() => {
                let p = panel_for(panel);
                self.choose_right_panel(if self.right == p { None } else { p });
            }
            (id, _) if id.starts_with("nx.") => self.notify_tr("Open a document first"),
            (id, _) => {
                self.execute(id);
            }
        }
    }

    /// Whether the strip item `id` can run now.
    fn wps_item_enabled(&self, id: &str) -> bool {
        if id.starts_with("nx.") {
            return id == "nx.select" || id == "nx.hand" || self.active.is_some();
        }
        pdfcraft_engine::commands::command(id).is_some_and(|spec| self.command_enabled(spec))
    }

    /// Whether the strip item `id` is the tool or panel in use (drawn selected).
    fn wps_item_on(&self, id: &str) -> bool {
        match id {
            "nx.select" => self.quick_tool == QuickTool::Select,
            "nx.hand" => self.quick_tool == QuickTool::Hand,
            "edit.edit_text" => self.quick_tool == QuickTool::EditText,
            "edit.text" => self.quick_tool == QuickTool::AddText,
            "edit.link" => self.quick_tool == QuickTool::Link,
            "redact.mark" => self.quick_tool == QuickTool::Redact,
            "view.marquee_zoom" => self.quick_tool == QuickTool::MarqueeZoom,
            "edit.snapshot" => self.quick_tool == QuickTool::Snapshot,
            "view.read_mode" => self.mode == Mode::Read,
            panel if panel_for(panel).is_some() => self.right == panel_for(panel),
            tool => match (crate::comments::CommentTool::from_command(tool), self.quick_tool) {
                (Some(t), QuickTool::Comment(current)) => t == current,
                _ => false,
            },
        }
    }

    /// NexusOffice: hand `path` to the host's tabs instead of opening it here, when this app is
    /// embedded, the host takes files and this app already has (or is opening) its document.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn hand_to_host(&mut self, path: &str) -> bool {
        let busy = !self.views.is_empty() || !self.pending_opens.is_empty() || self.password_prompt.is_some();
        match &self.host.open_document {
            Some(open) if self.embedded && busy => {
                open(path);
                self.backstage = None;
                true
            }
            _ => false,
        }
    }

    /// NexusOffice: every document but the tab's own goes to the host's tabs (a document made here
    /// from images, Combine or Extract, a file opened another way): a clean one with a file by its
    /// path, anything else by its bytes. Called every frame while embedded.
    pub(crate) fn hand_strays_to_host(&mut self) {
        if !self.embedded || (self.host.adopt_document.is_none() && self.host.open_document.is_none()) {
            return;
        }
        // Handed over and never answered for: after a while it counts as refused (it stays here),
        // so a host that drops the document can't keep the tab from closing. Frames keep coming
        // while one waits.
        let mut late = Vec::new();
        for (id, key, frames, generation) in &mut self.nexus_handed {
            *frames = frames.saturating_add(1);
            if *frames >= HANDOVER_FRAMES {
                late.push((*id, key.clone(), *generation));
            }
        }
        for (id, key, generation) in late {
            self.adoption_answered(id, Err(tx(NO_ANSWER).to_string()));
            // Kept here now; a late `Ok` is still answered ([`Self::late_adoption`]).
            if self.nexus_kept.contains(&id) {
                self.nexus_gave_up.push((id, key, generation));
            }
        }
        if !self.nexus_handed.is_empty()
            && let Some(ctx) = &self.ctx
        {
            ctx.request_repaint();
        }
        let own = match self.nexus_own {
            Some(own) if self.views.iter().any(|v| v.id == own) => own,
            _ => match self.views.first() {
                Some(v) => {
                    self.nexus_own = Some(v.id);
                    v.id
                }
                None => return,
            },
        };
        let waiting = |id: &pdfcraft_engine::DocId| self.nexus_handed.iter().any(|(h, _, _, _)| h == id);
        let strays: Vec<pdfcraft_engine::DocId> = self
            .views
            .iter()
            .map(|v| v.id)
            .filter(|id| *id != own && self.compare_old != Some(*id) && !self.nexus_kept.contains(id) && !waiting(id))
            .collect();
        for id in strays {
            let Some(doc) = self.session.get(id) else { continue };
            let (name, path, dirty, generation) = (doc.name.clone(), doc.path.clone(), doc.dirty, doc.edit_generation());
            let handed = match (&self.host.open_document, &self.host.adopt_document, path) {
                (Some(open), _, Some(p)) if !dirty => {
                    open(&p);
                    true
                }
                (_, Some(adopt), path) => match self.stray_bytes(id) {
                    Ok(bytes) => {
                        // The recovery copy moves with the document (the new tab writes the next
                        // one under its key). The document stays here, out of sight, until the
                        // host says the new tab opened (`adoption_answered`): bytes the new tab
                        // can't open would otherwise lose it, with no copy to recover.
                        let recovery_key = self.recovery_keys.get(&id).cloned();
                        self.nexus_handed.push((id, recovery_key.clone(), 0, generation));
                        let taken = adopt(AdoptedDocument { name: name.clone(), path, bytes, recovery_key, source: Some(id) });
                        // The host couldn't take it: kept here at once.
                        if let Err(e) = taken {
                            self.adoption_answered(id, Err(e));
                        }
                        false
                    }
                    // It can't be handed over: it stays open here (its changes, and its recovery
                    // copy, are kept; the bar above the document reaches it, and closing the tab
                    // asks about it), and the user is told once.
                    Err(e) => {
                        if !self.nexus_kept.contains(&id) {
                            self.nexus_kept.push(id);
                            let text = crate::i18n::fmt(tx(KEPT_NOTICE), &[("name", &name), ("e", &e)]);
                            self.notify(text);
                        }
                        false
                    }
                },
                // No hook takes it (a host with Open only, a document without a file): it stays.
                _ => false,
            };
            if handed && let Some(i) = self.views.iter().position(|v| v.id == id) {
                self.close_tab(i);
            }
        }
        // The tab's own document stays the one shown, unless the user is on one kept here.
        let shown = self.active_ids().map(|(_, id)| id);
        if !shown.is_some_and(|id| id == own || self.nexus_kept.contains(&id) || self.compare_old == Some(id))
            && let Some(i) = self.views.iter().position(|v| v.id == own)
        {
            self.active = Some(i);
        }
        self.nexus_kept.retain(|id| self.views.iter().any(|v| v.id == *id));
        self.nexus_handed.retain(|(id, _, _, _)| self.views.iter().any(|v| v.id == *id));
        self.nexus_gave_up.retain(|(id, _, _)| self.views.iter().any(|v| v.id == *id));
    }

    /// NexusOffice: the host's answer for a document this app handed over
    /// ([`Host::adopt_document`], [`AdoptedDocument::source`]). `Ok`: it opened in a tab of its
    /// own, and its copy here closes (its recovery copy on disk is the new tab's now). `Err(why)`
    /// (in the interface language): it didn't, so it stays in this tab, kept like one that
    /// couldn't be prepared, and the user is told. Hosts answer in the frame they were handed the
    /// document ([`AdoptDocumentFn`]); a late `Ok` (after [`HANDOVER_FRAMES`]) is logged and
    /// answered by [`Self::late_adoption`]. Any other answer for no document handed over is
    /// ignored.
    pub fn adoption_answered(&mut self, source: pdfcraft_engine::DocId, result: Result<(), String>) {
        let Some(at) = self.nexus_handed.iter().position(|(id, _, _, _)| *id == source) else {
            if result.is_ok() {
                self.late_adoption(source);
            }
            return;
        };
        let (id, key, _, _) = self.nexus_handed.remove(at);
        match result {
            Ok(()) => {
                // Closing must not delete the copy the new tab took over; one written here under
                // another key since (none went with the document) is this tab's, and goes.
                if key.is_some() && self.recovery_keys.get(&id) == key.as_ref() {
                    self.recovery_keys.remove(&id);
                }
                if let Some(i) = self.views.iter().position(|v| v.id == id) {
                    self.close_tab(i);
                }
            }
            Err(e) => {
                if self.nexus_kept.contains(&id) || !self.views.iter().any(|v| v.id == id) {
                    return;
                }
                self.nexus_kept.push(id);
                let name = self.session.get(id).map(|d| d.name.clone()).unwrap_or_default();
                let text = crate::i18n::fmt(tx(KEPT_NOTICE), &[("name", &name), ("e", &e)]);
                self.notify(text);
            }
        }
    }

    /// The host said `Ok` for document `source` after this app stopped waiting for it (it is kept
    /// here): the host broke the same-frame contract, which is logged. Unchanged since it was
    /// handed over, the new tab has it as it is: the copy here closes and the recovery copy is the
    /// new tab's. Changed here since: both stay (neither copy of the changes is lost), and the one
    /// here gets a recovery copy of its own. Typing not in the document yet counts as a change
    /// ([`Self::typing_pending`]). Said either way, and if the copy of its own couldn't be written.
    fn late_adoption(&mut self, source: pdfcraft_engine::DocId) {
        let Some(at) = self.nexus_gave_up.iter().position(|(id, _, _)| *id == source) else {
            log::warn!("NexusOffice: an Ok for document {} that wasn't waiting for one was ignored", source.0);
            return;
        };
        let (id, key, generation) = self.nexus_gave_up.remove(at);
        log::warn!(
            "NexusOffice: the host said document {} opened in a tab of its own after {HANDOVER_FRAMES} frames without an answer (hosts answer in the same frame); it was kept here meanwhile",
            id.0
        );
        let Some(doc) = self.session.get(id) else { return };
        let (name, same_edits) = (doc.name.clone(), doc.edit_generation() == generation);
        let Some(i) = self.views.iter().position(|v| v.id == id) else { return };
        // Typing not in the document yet (a form field, Fill & Sign text, a paragraph or text being
        // written) changes it too: closing would drop it without a word.
        let unchanged = same_edits && !self.typing_pending(i);
        let went_with = key.is_some() && self.recovery_keys.get(&id) == key.as_ref();
        if unchanged {
            // Closing must not delete the copy the new tab took over.
            if went_with {
                self.recovery_keys.remove(&id);
            }
            self.nexus_kept.retain(|k| *k != id);
            self.close_tab(i);
            self.notify(crate::i18n::fmt(tx(LATE_MOVED), &[("name", &name)]));
        } else {
            let own = if went_with { self.recovery_key_of_its_own(id) } else { Ok(()) };
            let text = match own {
                Ok(()) => crate::i18n::fmt(tx(LATE_BOTH), &[("name", &name)]),
                Err(e) => crate::i18n::fmt(tx(LATE_BOTH_NO_COPY), &[("name", &name), ("e", &e)]),
            };
            self.notify(text);
        }
    }

    /// A stray document's bytes for the host, checked to open in a new tab as they are (without
    /// a password: a recovered encrypted PDF unlocked here would not open there, and vanish).
    /// Only an encrypted document is opened again to check (a full parse on the UI thread, every
    /// time one is handed over): the bytes of one that isn't are the document as it is open here.
    /// Why one can't move is said in the interface language; the engine's reason goes to the log.
    fn stray_bytes(&self, id: pdfcraft_engine::DocId) -> Result<Arc<Vec<u8>>, String> {
        let unready = |e: &dyn std::fmt::Display| {
            log::warn!("a document couldn't be handed to a tab of its own: {e}");
            tx(STRAY_UNREADY).to_string()
        };
        let doc = self.session.get(id).ok_or_else(|| unready(&"the document is gone"))?;
        // Encrypted as opened, as it will be saved (protection added here), or opened with a
        // password: the copy may need it.
        let encrypted = doc.info.encrypted || doc.password.is_some() || doc.security_summary().is_some();
        let saved = self.session.save_bytes(id);
        // Tests make the save fail the way a full disk would, through the same mapping.
        #[cfg(test)]
        let saved = if tests::FAIL_STRAY_SAVES.with(std::cell::Cell::get) {
            Err(pdfcraft_engine::EditError::Write("the disk is full".into()))
        } else {
            saved
        };
        let bytes = saved.map_err(|e| unready(&e))?;
        if !encrypted {
            return Ok(bytes);
        }
        #[cfg(test)]
        tests::STRAY_PROBES.with(|n| n.set(n.get().saturating_add(1)));
        match pdfcraft_engine::Session::new().open("check", None, bytes.clone(), None) {
            Ok(_) => Ok(bytes),
            Err(pdfcraft_render::OpenError::NeedsPassword | pdfcraft_render::OpenError::WrongPassword) => {
                Err(tx("it needs its password").to_string())
            }
            Err(e) => Err(unready(&e)),
        }
    }

    /// NexusOffice: documents of this tab are still on their way to tabs of their own (recovered
    /// ones being read, waiting for their password, or handed over and not yet answered for):
    /// closing the tab now would drop them.
    pub fn handing_over(&self) -> bool {
        // Handed over, and the host hasn't said yet whether its new tab opened.
        if self.nexus_handed.iter().any(|(id, _, _, _)| self.views.iter().any(|v| v.id == *id)) {
            return true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if self.pending_opens.iter().any(|p| p.recovered.is_some()) {
            return true;
        }
        self.password_prompt.as_ref().is_some_and(|p| p.recovered.is_some())
    }

    /// NexusOffice: documents kept in this tab because they couldn't move to tabs of their own.
    pub fn kept_documents(&self) -> usize {
        self.nexus_kept.iter().filter(|id| self.views.iter().any(|v| v.id == **id)).count()
    }

    /// NexusOffice: embeds the app in a host window's tab: the WPS layout, [`Self::embedded`],
    /// and document ids of its own. Every app in the process numbers its documents in a range of
    /// its own (each has its own session), so interface state keyed by a document's id (scroll
    /// positions, comment composers, form fields) never passes from one tab to another.
    pub fn embed(&mut self) {
        static APPS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        self.wps = true;
        self.embedded = true;
        let n = APPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed).saturating_add(1);
        if let Err(e) = self.session.reserve_ids_from(n.saturating_mul(1 << 32)) {
            log::warn!("{e}");
        }
    }

    /// NexusOffice: autosave crash-recovery copies into `store`, without offering what is there
    /// (another tab of the host window offers it; see [`Self::enable_recovery`]).
    pub fn enable_autosave(&mut self, store: crate::RecoveryStore) {
        store.claim();
        self.recovery = Some(store);
    }

    /// NexusOffice: a host window's tab that isn't shown keeps writing its recovery copy.
    /// Recovered documents being read finish opening, and go to tabs of their own, while the tab
    /// is hidden too (one goes to a new tab, which hides this one, and the others would wait here).
    pub fn background_frame(&mut self, ctx: &egui::Context) {
        self.ctx = Some(ctx.clone());
        let now = ctx.input(|i| i.time);
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_pending_opens();
        self.hand_strays_to_host();
        self.autosave_tick(now);
    }

    /// NexusOffice: a blank A4 page named `name`, as this app's document, unchanged (closing it
    /// untouched asks nothing).
    pub fn open_blank_named(&mut self, name: &str) -> Result<(), String> {
        let bytes = self.session.create_blank(595.276, 841.89, 1).map_err(|e| e.to_string())?;
        let id = self.session.open(name, None, bytes, None).map_err(|e| e.to_string())?;
        self.session.set_reading_colors(id, self.reading_colors);
        let info = &self.session.get(id).ok_or("the new document could not be opened")?.info;
        self.views.push(crate::DocView::new(id, info, self.view_defaults));
        self.active = Some(self.views.len() - 1);
        Ok(())
    }

    /// The families the open text editor draws with (the document's own fonts), if any.
    fn editor_families(&self) -> Vec<String> {
        self.active
            .and_then(|i| self.views.get(i))
            .and_then(|v| v.line_editor.as_ref())
            .map(|ed| ed.fonts.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Whether the window's fonts have every family this app draws with: its interface families
    /// and, once they have had a pass to become active, the open text editor's document fonts.
    /// Another editor in a host window may have replaced either.
    pub fn own_fonts_installed(&self, ctx: &egui::Context) -> bool {
        if !theme::fonts_installed(ctx) {
            return false;
        }
        let families = self.editor_families();
        if families.is_empty() {
            return true;
        }
        let installed = ctx.data(|d| d.get_temp::<u64>(crate::edit_text_ui::source_fonts_pass_id()));
        if installed.is_some_and(|pass| pass >= ctx.cumulative_pass_nr()) {
            // Installed this pass: active from the next one.
            return true;
        }
        ctx.fonts(|f| families.iter().all(|name| f.definitions().families.contains_key(&egui::FontFamily::Name(name.as_str().into()))))
    }

    /// Embedded: this app's fonts installed beside the window's other faces, with the open text
    /// editor's document fonts too (another editor's font set had neither).
    pub(crate) fn install_embedded_fonts(&mut self, ctx: &egui::Context, cjk: theme::CjkPreference) {
        let mut fonts = theme::fonts_beside(ctx, cjk);
        let mut source = false;
        if let Some(i) = self.active
            && let Some(view) = self.views.get_mut(i)
            && let Some(doc) = self.session.get(view.id)
            && let Some(ed) = view.line_editor.as_mut()
            && !ed.fonts.is_empty()
        {
            ed.fonts = crate::edit_text_ui::add_source_fonts(&mut fonts, doc, ed.page);
            source = true;
        }
        ctx.set_fonts(fonts);
        if source {
            let pass = ctx.cumulative_pass_nr();
            ctx.data_mut(|d| d.insert_temp(crate::edit_text_ui::source_fonts_pass_id(), pass));
        }
    }

    /// NexusOffice: the tab came forward in a host window. Installs the fonts again only when
    /// another editor took some away (rebuilding the font atlas costs a frame); the look is
    /// applied again either way.
    pub fn reactivate(&mut self, ctx: &egui::Context) {
        if self.own_fonts_installed(ctx) {
            theme::apply(ctx, self.theme);
        } else {
            self.restyle();
        }
    }

    /// NexusOffice: install the fonts and the look again, painting nothing on the next frame. A
    /// host window with other editors calls it when this app's tab comes forward: another editor
    /// may have replaced the window's fonts and visuals meanwhile.
    pub fn restyle(&mut self) {
        self.styled = false;
        self.fonts_ready = false;
        // The open text editor isn't drawn on the skipped frame: it takes the keyboard back on
        // the next one instead of losing it (which applied the edit behind the user's back).
        for view in &mut self.views {
            if let Some(ed) = view.line_editor.as_mut() {
                ed.keep_focus();
            }
        }
    }

    /// NexusOffice: `bytes` as a new, unsaved document of this app (a document another tab made
    /// and handed over): closing it asks to save it.
    pub fn open_unsaved(&mut self, name: &str, bytes: Arc<Vec<u8>>) -> Result<(), String> {
        self.open_adopted(AdoptedDocument { name: name.to_string(), path: None, bytes, recovery_key: None, source: None })
    }

    /// NexusOffice: a document another tab handed over ([`Host::adopt_document`]), as this app's
    /// own: unsaved, at its file, and with its recovery copy, which this app writes at once
    /// (under the same key, as its own) when autosave is on ([`Self::enable_autosave`] first),
    /// and deletes once the document is saved or discarded.
    pub fn open_adopted(&mut self, doc: AdoptedDocument) -> Result<(), String> {
        let AdoptedDocument { name, path, bytes, recovery_key, source: _ } = doc;
        let id = self.session.open_unsaved(name, path, bytes).map_err(|e| e.to_string())?;
        if let Some(key) = recovery_key {
            self.recovery_keys.insert(id, key);
        }
        self.session.set_reading_colors(id, self.reading_colors);
        let info = &self.session.get(id).ok_or("the new document could not be opened")?.info;
        self.views.push(crate::DocView::new(id, info, self.view_defaults));
        self.active = Some(self.views.len() - 1);
        // No minute without a copy: the first autosave would wait for the interval.
        self.autosave_now();
        Ok(())
    }

    /// Whether a document is still being opened (read and prepared off the UI thread).
    pub fn opens_pending(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            !self.pending_opens.is_empty()
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }

    /// Whether a host file dialog (or any picker) is open or its answer is still to be used.
    pub fn file_dialog_open(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.pickers.pending()
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }
}

/// The right panel an `nx.panel.*` id toggles.
fn panel_for(id: &str) -> Option<RightPanel> {
    match id {
        "nx.panel.pages" => Some(RightPanel::Pages),
        "nx.panel.bookmarks" => Some(RightPanel::Bookmarks),
        "nx.panel.attachments" => Some(RightPanel::Attachments),
        "nx.panel.layers" => Some(RightPanel::Layers),
        _ => None,
    }
}

/// Whether `panel` sits on the right (a pane beside the document, as comments do in WPS) rather
/// than in the navigation sidebar on the left.
fn on_the_right(panel: RightPanel) -> bool {
    matches!(panel, RightPanel::Comments | RightPanel::Search | RightPanel::Compare | RightPanel::Accessibility)
}

/// Draws the whole window below the document tab strip (if any): the two top rows, the status
/// bar, the side panes and the document.
pub fn show(app: &mut PdfCraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    bar(app, ui, &t);
    let reading = app.mode == Mode::Read;
    if !app.wps_collapsed && !reading {
        strip(app, ui, &t);
    }
    if app.embedded && app.kept_documents() > 0 {
        kept_bar(app, ui, &t);
    }
    if app.active.is_some() {
        status_bar(app, ui, &t);
        if !reading {
            sidebar(app, ui, &t);
            panes(app, ui, &t);
        }
    }
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.pasteboard)).show(ui, |ui| {
        if app.combine_showing() && app.active.is_none() {
            back_bar(app, ui, &t);
        }
        match app.active {
            None if app.combine_showing() => crate::combine_ui::page(app, ui),
            None => crate::home::show(app, ui),
            Some(index) => match crate::compare_ui::side_by_side(app, index) {
                Some((old, new)) => crate::compare_ui::compare_area(app, old, new, ui),
                None => crate::canvas::document_area(app, index, ui),
            },
        }
    });
}

/// Embedded, with documents that couldn't move to tabs of their own: the tab's documents, to
/// switch between (and save, or close: closing asks about unsaved changes).
fn kept_bar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    let shown = app.active_ids().map(|(_, id)| id);
    let docs: Vec<(pdfcraft_engine::DocId, String, bool)> = app
        .views
        .iter()
        .filter(|v| app.nexus_own == Some(v.id) || app.nexus_kept.contains(&v.id))
        .filter_map(|v| app.session.get(v.id).map(|d| (v.id, d.name.clone(), d.dirty)))
        .collect();
    egui::Panel::top("wps_kept")
        .exact_size(28.0)
        .frame(egui::Frame::NONE.fill(t.info_bg).inner_margin(egui::Margin::symmetric(10, 0)).stroke(Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(tx("Also open in this tab:")).font(theme::regular(12.0)).color(t.text_muted));
                for (id, name, dirty) in docs {
                    let label = if dirty { format!("{name} •") } else { name };
                    if ui.selectable_label(shown == Some(id), egui::RichText::new(label).font(theme::regular(12.0))).clicked()
                        && let Some(i) = app.views.iter().position(|v| v.id == id)
                    {
                        app.active = Some(i);
                    }
                    if app.nexus_kept.contains(&id)
                        && ui.small_button("✕").on_hover_text(tx("Close")).clicked()
                        && let Some(i) = app.views.iter().position(|v| v.id == id)
                    {
                        app.request_close_tab(i);
                    }
                }
            });
        });
}

/// Above the Combine files page when it replaces the document: the way back to it.
fn back_bar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let label = tx("Back to the document");
        let resp = ui.add(egui::Button::new(egui::RichText::new(format!("‹  {label}")).color(t.accent_text)).frame(false));
        if resp.clicked() {
            app.combine_tab.focused = false;
            if app.active.is_none() && !app.views.is_empty() {
                app.active = Some(0);
            }
        }
    });
}

/// The File button's fill: the accent, faint, over the row (a little stronger under the pointer).
pub(crate) fn file_fill(t: &Tokens, hovered: bool) -> Color32 {
    t.titlebar.lerp_to_gamma(t.accent, if hovered { 0.16 } else { 0.08 })
}

/// File's label on `fill`: the accent, darkened toward the body text as far as 4.5:1 needs.
pub(crate) fn file_ink(t: &Tokens, fill: Color32) -> Color32 {
    [0.0, 0.2, 0.4, 0.6, 0.8]
        .iter()
        .map(|k| t.accent.lerp_to_gamma(t.text, *k))
        .find(|ink| theme::contrast_ratio(*ink, fill) >= 4.5)
        .unwrap_or(t.text)
}

/// The colour "Saved" is written in: green that keeps 4.5:1 on the row in either theme.
fn saved_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(0x6E, 0xE7, 0xB7) } else { Color32::from_rgb(0x04, 0x78, 0x57) }
}

/// The width of `text` in `font`.
fn text_w(ui: &Ui, text: &str, font: egui::FontId) -> f32 {
    ui.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font, Color32::WHITE).size().x)
}

/// The top row.
fn bar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("wps_bar")
        .exact_size(BAR_H)
        .frame(egui::Frame::NONE.fill(t.titlebar).inner_margin(egui::Margin { left: 10, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            let narrow = ui.ctx().content_rect().width() < NARROW_W;
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                file_button(ui, app, t);
                ui.add_space(8.0);
                quick_access(ui, app, t);
                ui.add_space(6.0);
                let (d, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
                ui.painter().rect_filled(d, 0.0, t.border);
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                    more_menu(ui, app, t);
                    search_box(ui, app, t, narrow);
                    ui.add_space(4.0);
                    save_state(ui, app, t);
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        tab_row(ui, app, t, full.height());
                    });
                });
            });
            ui.painter().hline(full.x_range(), full.max.y - 0.5, Stroke::new(1.0, t.divider));
        });
}

/// File (Tệp): three bars drawn in code and the word, on a faint accent; opens the File page.
fn file_button(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens) -> Response {
    let label = tx("File");
    let font = theme::semibold(13.0);
    let tw = text_w(ui, label, font.clone());
    let (r, _) = ui.allocate_exact_size(vec2(10.0 + 14.0 + 7.0 + tw + 12.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, file_button_id(), Sense::click());
    let fill = file_fill(t, resp.hovered());
    let ink = file_ink(t, fill);
    ui.painter().rect_filled(r, 6.0, fill);
    let x = r.min.x + 10.0;
    for k in [-4.0, 0.0, 4.0] {
        let y = r.center().y + k;
        ui.painter().line_segment([pos2(x, y), pos2(x + 14.0, y)], Stroke::new(1.6, ink));
    }
    ui.painter().text(pos2(x + 21.0, r.center().y), Align2::LEFT_CENTER, label, font, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if resp.clicked() {
        app.backstage = Some(if app.active.is_some() { crate::nexus_backstage::Page::Info } else { crate::nexus_backstage::Page::Open });
    }
    resp
}

/// Quick Access: Save, Print, Undo, Redo.
fn quick_access(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens) {
    for (id, icon, label) in
        [("file.save", "save", "Save"), ("print.dialog", "printer", "Print"), ("edit.undo", "undo-2", "Undo"), ("edit.redo", "redo-2", "Redo")]
    {
        let enabled = app.wps_item_enabled(id);
        let shown = tx(label);
        let tip = crate::commands::command_tip(ui.ctx(), &format!("{shown} ({{key}})"), id);
        let (r, resp) = ui.allocate_exact_size(vec2(CONTROL_H, CONTROL_H), if enabled { Sense::click() } else { Sense::hover() });
        if enabled && resp.hovered() {
            ui.painter().rect_filled(r, 6.0, t.hover);
        }
        icons::paint(ui, r, icon, 16.0, if enabled { t.icon } else { t.text_faint.gamma_multiply(0.6) });
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, shown));
        if resp.on_hover_text(tip).clicked() && enabled {
            app.run_wps_item(id);
        }
    }
}

/// Which tabs the row shows, given each tab's width and whether it must stay (the current one):
/// all when they fit, else as many as fit in order with room kept for the » button, the current
/// one always.
pub fn fit_tabs(widths: &[(f32, bool)], room: f32, more_w: f32) -> Vec<bool> {
    let w = |x: f32| if x.is_finite() { x.max(0.0) } else { 0.0 };
    if widths.iter().map(|(x, _)| w(*x)).sum::<f32>() <= room + 0.5 {
        return vec![true; widths.len()];
    }
    let mut left = room - more_w - widths.iter().filter(|(_, pinned)| *pinned).map(|(x, _)| w(*x)).sum::<f32>();
    widths
        .iter()
        .map(|(x, pinned)| {
            if *pinned {
                return true;
            }
            if w(*x) <= left + 0.5 {
                left -= w(*x);
                true
            } else {
                left = f32::NEG_INFINITY;
                false
            }
        })
        .collect()
}

/// The tool tabs, left aligned: the current one in the accent, semibold, underlined. A click on
/// the current tab folds the strip away (and back).
fn tab_row(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens, height: f32) {
    let labels: Vec<&str> = WpsTab::ALL.iter().map(|tab| tx(tab.label())).collect();
    let widths: Vec<(f32, bool)> =
        WpsTab::ALL.iter().zip(&labels).map(|(tab, shown)| (text_w(ui, shown, theme::semibold(13.0)) + 22.0, *tab == app.wps_tab)).collect();
    let fits = fit_tabs(&widths, ui.available_width(), MORE_TABS_W);
    for (((tab, shown), (w, _)), fit) in WpsTab::ALL.iter().zip(&labels).zip(&widths).zip(&fits) {
        if !fit {
            continue;
        }
        let (r, resp) = ui.allocate_exact_size(vec2(*w, height), Sense::click());
        let active = app.wps_tab == *tab;
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, *shown));
        if resp.hovered() && !active {
            ui.painter().rect_filled(r.shrink2(vec2(2.0, 6.0)), 6.0, t.hover);
        }
        let (font, color) = if active { (theme::semibold(13.0), t.accent_text) } else { (theme::regular(13.0), t.text) };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, *shown, font, color);
        if active {
            ui.painter().rect_filled(Rect::from_min_max(pos2(r.min.x + 10.0, r.max.y - 2.0), pos2(r.max.x - 10.0, r.max.y)), 1.0, t.accent);
        }
        if resp.clicked() {
            if active {
                app.wps_collapsed = !app.wps_collapsed;
            } else {
                app.wps_tab = *tab;
                app.wps_collapsed = false;
            }
        }
    }
    let left_out: Vec<(WpsTab, &str)> =
        WpsTab::ALL.iter().zip(&labels).zip(&fits).filter(|(_, fit)| !**fit).map(|((tab, shown), _)| (*tab, *shown)).collect();
    if !left_out.is_empty() {
        let (r, _) = ui.allocate_exact_size(vec2(MORE_TABS_W, height), Sense::hover());
        let resp = ui.interact(r, more_tabs_id(), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r.shrink2(vec2(0.0, 6.0)), 6.0, t.hover);
        }
        icons::paint(ui, r, "chevrons-right", 14.0, t.text_muted);
        let tip = tx("More tabs");
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
        egui::Popup::menu(&resp).show(|ui| {
            for (tab, shown) in left_out {
                if ui.selectable_label(false, shown).clicked() {
                    app.wps_tab = tab;
                    app.wps_collapsed = false;
                    ui.close();
                }
            }
        });
    }
}

/// The save state: "Saved" with a check, or "Not saved". A click offers Save and Save As.
fn save_state(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens) {
    let Some(index) = app.active else { return };
    let saved = !app.has_unsaved_work(index) && app.views.get(index).and_then(|v| app.session.get(v.id)).is_some_and(|d| d.path.is_some());
    let (text, color) = if saved { (tx("Saved"), saved_color(t.dark())) } else { (tx("Not saved"), t.text_muted) };
    let font = theme::regular(12.0);
    let tw = text_w(ui, text, font.clone());
    let icon_w = if saved { 20.0 } else { 0.0 };
    let (r, _) = ui.allocate_exact_size(vec2(8.0 + icon_w + tw + 4.0 + 10.0 + 8.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, save_state_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    if saved {
        icons::paint(ui, Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(15.0, 15.0)), "check", 14.0, color);
    }
    let x = r.min.x + 8.0 + icon_w;
    ui.painter().text(pos2(x, r.center().y), Align2::LEFT_CENTER, text, font, color);
    icons::paint(ui, Rect::from_center_size(pos2(x + tw + 9.0, r.center().y + 1.0), vec2(10.0, 10.0)), "chevron-down", 10.0, t.text_muted);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(200.0);
        if ui.button(tx("Save")).clicked() {
            ui.close();
            app.execute("file.save");
        }
        if ui.button(tx("Save as…")).clicked() {
            ui.close();
            app.execute("file.save_as");
        }
    });
}

/// The search box: a click opens Find tools and commands. Narrow windows show the magnifier alone.
fn search_box(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens, narrow: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(if narrow { CONTROL_H + 4.0 } else { 220.0 }, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, search_id(), Sense::click());
    ui.painter().rect(r, 8.0, if resp.hovered() { t.hover } else { t.field }, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let icon_x = if narrow { r.center().x } else { r.min.x + 16.0 };
    icons::paint(ui, Rect::from_center_size(pos2(icon_x, r.center().y), vec2(14.0, 14.0)), "search", 14.0, t.text_muted);
    let ph = tx("Find tools and commands");
    if !narrow {
        ui.painter().with_clip_rect(r.shrink2(vec2(8.0, 0.0))).text(
            pos2(r.min.x + 30.0, r.center().y),
            Align2::LEFT_CENTER,
            ph,
            theme::regular(12.0),
            t.text_muted,
        );
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, ph));
    let tip = crate::commands::command_tip(ui.ctx(), &format!("{ph} ({{key}})"), "view.palette");
    if resp.on_hover_text(tip).clicked() {
        app.palette_open = true;
    }
}

/// "⋯": the theme, Preferences, keyboard shortcuts and About.
fn more_menu(ui: &mut Ui, app: &mut PdfCraftApp, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(CONTROL_H, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, more_button_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    icons::paint(ui, r, "ellipsis", 18.0, t.text_muted);
    let tip = tx("More options");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    let resp = resp.on_hover_text(tip);
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        for (command, label) in [("view.theme.light", "Light gray"), ("view.theme.dark", "Dark gray"), ("view.theme.system", "Use system setting")] {
            if ui.button(tx(label)).clicked() {
                ui.close();
                app.execute(command);
            }
        }
        ui.separator();
        for (command, label) in [("app.preferences", "Preferences…"), ("help.shortcuts", "Keyboard shortcuts"), ("help.about", "About Nexus PDF")] {
            if ui.button(tx(label)).clicked() {
                ui.close();
                app.execute(command);
            }
        }
    });
}

/// The width a group takes in the strip (its divider and margins included).
fn group_width(ui: &Ui, items: &[Item]) -> f32 {
    columns(items).iter().map(|col| column_width(ui, col)).sum::<f32>() + 2.0 * columns(items).len().saturating_sub(1) as f32 + 12.0
}

/// A group's columns: a large button alone, small ones two at a time.
fn columns(items: &[Item]) -> Vec<Vec<Item>> {
    let mut cols: Vec<Vec<Item>> = Vec::new();
    for item in items {
        match cols.last_mut() {
            Some(col) if !item.big && col.len() == 1 && col.first().is_some_and(|c| !c.big) => col.push(*item),
            _ => cols.push(vec![*item]),
        }
    }
    cols
}

fn big_width(ui: &Ui, label: &str) -> f32 {
    (text_w(ui, tx(label), theme::regular(12.0)) + 14.0).max(46.0)
}

fn small_width(ui: &Ui, label: &str) -> f32 {
    6.0 + 16.0 + 6.0 + text_w(ui, tx(label), theme::regular(12.0)) + 8.0
}

fn column_width(ui: &Ui, col: &[Item]) -> f32 {
    col.iter().map(|i| if i.big { big_width(ui, i.label) } else { small_width(ui, i.label) }).fold(0.0, f32::max)
}

/// How many groups, from the first, fit in `room` given their widths: all of them
/// (`usize::MAX`) when they do.
pub fn fit_groups(widths: &[f32], room: f32) -> usize {
    let mut x = 0.0;
    for (i, w) in widths.iter().enumerate() {
        x += if w.is_finite() { w.max(0.0) } else { 0.0 };
        if x > room + 0.5 {
            return i;
        }
    }
    usize::MAX
}

/// The command strip: the current tab's groups, then "…" with the ones that didn't fit.
fn strip(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("wps_strip")
        .exact_size(STRIP_H)
        .frame(
            egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin { left: 10, right: 8, top: 3, bottom: 3 }).stroke(Stroke::new(1.0, t.divider)),
        )
        .show(ui, |ui| {
            let tab_groups = groups(app.wps_tab);
            let widths: Vec<f32> = tab_groups.iter().map(|g| group_width(ui, g)).collect();
            let full = ui.max_rect();
            let all = widths.iter().sum::<f32>() <= full.width() + 0.5;
            let room = if all { full.width() } else { (full.width() - OVERFLOW_W).max(0.0) };
            let shown = if all { tab_groups.len() } else { fit_groups(&widths, room).min(tab_groups.len()) };
            let mut x = full.min.x;
            let mut clicked = None;
            for (g, w) in tab_groups.iter().zip(&widths).take(shown) {
                let rect = Rect::from_min_size(pos2(x, full.min.y), vec2(*w, CONTENT_H));
                if let Some(id) = draw_group(ui, app, t, g, rect, true) {
                    clicked = Some(id);
                }
                x += w;
            }
            if shown < tab_groups.len() {
                let r = Rect::from_min_size(pos2(full.max.x - OVERFLOW_W + 4.0, full.min.y), vec2(OVERFLOW_W - 4.0, CONTENT_H));
                let resp = ui.interact(r, overflow_button_id(), Sense::click());
                if resp.hovered() || overflow_open(ui.ctx()) {
                    ui.painter().rect_filled(r, 6.0, t.hover);
                }
                icons::paint(ui, r, "ellipsis", 20.0, t.text_muted);
                let tip = tx("More commands");
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
                let resp = resp.on_hover_text(tip);
                if let Some(id) = overflow_panel(app, ui, t, &resp, shown) {
                    clicked = Some(id);
                }
            } else {
                ui.ctx().data_mut(|d| d.remove::<(WpsTab, Rect)>(overflow_open_id()));
            }
            let used: f32 = widths.iter().take(shown).sum();
            ui.ctx().data_mut(|d| d.insert_temp(strip_fit_id(), StripFit { tab: app.wps_tab, shown, groups: tab_groups.len(), used, room }));
            if let Some(id) = clicked {
                app.run_wps_item(id);
            }
        });
}

/// What the strip drew last frame (tests): the tab, how many of its groups, the width they took
/// and the room there was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StripFit {
    pub tab: WpsTab,
    pub shown: usize,
    pub groups: usize,
    pub used: f32,
    pub room: f32,
}

fn strip_fit_id() -> Id {
    Id::new("nexus_wps_strip_fit")
}

/// What the strip drew last frame.
pub fn strip_fit(ctx: &egui::Context) -> Option<StripFit> {
    ctx.data(|d| d.get_temp(strip_fit_id()))
}

/// Draws a group's buttons in `rect`, then its divider when `divider`. Returns the item clicked.
fn draw_group(ui: &mut Ui, app: &PdfCraftApp, t: &Tokens, items: &[Item], rect: Rect, divider: bool) -> Option<&'static str> {
    let mut clicked = None;
    let mut x = rect.min.x;
    let cols = columns(items);
    for col in &cols {
        let w = column_width(ui, col);
        for (row, item) in col.iter().enumerate() {
            let r = if item.big {
                Rect::from_min_size(pos2(x, rect.min.y), vec2(w, CONTENT_H))
            } else {
                Rect::from_min_size(pos2(x, rect.min.y + row as f32 * (SMALL_H + 2.0)), vec2(w, SMALL_H))
            };
            if button(ui, app, t, item, r) {
                clicked = Some(item.id);
            }
        }
        x += w + 2.0;
    }
    if divider {
        let dx = rect.max.x - 6.0;
        ui.painter().line_segment([pos2(dx, rect.min.y + 6.0), pos2(dx, rect.max.y - 6.0)], Stroke::new(1.0, t.divider));
    }
    clicked
}

/// One strip button at `r`; `true` when clicked (and enabled).
fn button(ui: &mut Ui, app: &PdfCraftApp, t: &Tokens, item: &Item, r: Rect) -> bool {
    let enabled = app.wps_item_enabled(item.id);
    let on = app.wps_item_on(item.id);
    let label = tx(item.label);
    let id = ui.id().with(("wps_item", item.id, r.min.x as i32, r.min.y as i32));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let fill = if on {
        t.selected
    } else if enabled && resp.is_pointer_button_down_on() {
        t.pressed
    } else if enabled && resp.hovered() {
        t.hover
    } else {
        Color32::TRANSPARENT
    };
    if fill.a() > 0 {
        ui.painter().rect_filled(r, 5.0, fill);
    }
    let (icon, tip) = icon_and_tip(item.id);
    let ink = if !enabled {
        t.text_faint.gamma_multiply(0.55)
    } else if on {
        t.accent_text
    } else {
        t.text
    };
    let icon_ink = if !enabled {
        ink
    } else if on {
        t.accent_text
    } else {
        t.icon
    };
    if item.big {
        icons::paint(ui, Rect::from_center_size(pos2(r.center().x, r.min.y + 15.0), vec2(20.0, 20.0)), icon, 20.0, icon_ink);
        ui.painter().with_clip_rect(r).text(pos2(r.center().x, r.max.y - 11.0), Align2::CENTER_CENTER, label, theme::regular(12.0), ink);
    } else {
        icons::paint(ui, Rect::from_center_size(pos2(r.min.x + 14.0, r.center().y), vec2(16.0, 16.0)), icon, 16.0, icon_ink);
        ui.painter().with_clip_rect(r).text(pos2(r.min.x + 28.0, r.center().y), Align2::LEFT_CENTER, label, theme::regular(12.0), ink);
    }
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, on, label));
    let tip = tx(tip);
    let tip = if tip.is_empty() { label.to_owned() } else { crate::commands::command_tip(ui.ctx(), &format!("{tip} ({{key}})"), item.id) };
    resp.on_hover_text(tip).clicked() && enabled
}

/// Where the "…" panel was drawn last frame, while it is open: (its tab, its rect).
fn overflow_open_id() -> Id {
    Id::new("nexus_wps_overflow_open")
}

/// Whether the "…" panel is open.
pub fn overflow_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<(WpsTab, Rect)>(overflow_open_id())).is_some()
}

/// The "…" panel: the groups from `from` on, under the button, wrapped to the window. It closes
/// on Escape, on another tab, on a click outside it, and once one of its buttons ran.
fn overflow_panel(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens, button: &Response, from: usize) -> Option<&'static str> {
    let ctx = ui.ctx().clone();
    let mut open: Option<(WpsTab, Rect)> = ctx.data(|d| d.get_temp(overflow_open_id()));
    if button.clicked() {
        open = match open {
            Some(_) => None,
            None => Some((app.wps_tab, Rect::NOTHING)),
        };
    } else if let Some((tab, rect)) = &open {
        let outside =
            ctx.input(|i| i.pointer.primary_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p) && !button.rect.contains(p)));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if *tab != app.wps_tab || escape || outside {
            open = None;
        }
    }
    let Some((tab, _)) = open else {
        ctx.data_mut(|d| d.remove::<(WpsTab, Rect)>(overflow_open_id()));
        return None;
    };
    let rest: Vec<&'static [Item]> = groups(tab).iter().skip(from).copied().collect();
    let max_w = (ctx.content_rect().width() - 24.0).max(120.0);
    let mut clicked = None;
    let area = egui::Area::new(overflow_open_id().with("area"))
        .order(egui::Order::Foreground)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(button.rect.right_bottom() + vec2(0.0, 6.0))
        .constrain(true)
        .show(&ctx, |ui| {
            egui::Frame::popup(ui.style()).fill(t.chrome).show(ui, |ui| {
                ui.set_max_width(max_w);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 6.0);
                    for g in &rest {
                        let w = group_width(ui, g);
                        let (rect, _) = ui.allocate_exact_size(vec2(w, CONTENT_H), Sense::hover());
                        if let Some(id) = draw_group(ui, app, t, g, rect, true) {
                            clicked = Some(id);
                        }
                    }
                });
            });
        });
    if clicked.is_some() {
        ctx.data_mut(|d| d.remove::<(WpsTab, Rect)>(overflow_open_id()));
    } else {
        ctx.data_mut(|d| d.insert_temp(overflow_open_id(), (tab, area.response.rect)));
    }
    clicked
}

/// The navigation rail on the left and, beside it, the navigation panel chosen there.
fn sidebar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::left("wps_rail")
        .resizable(false)
        .exact_size(RAIL_W)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(4, 8)).stroke(Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (panel, icon, tip) in [
                (RightPanel::Pages, "files", "Page thumbnails"),
                (RightPanel::Bookmarks, "bookmark", "Bookmarks"),
                (RightPanel::Attachments, "paperclip", "Attachments"),
                (RightPanel::Layers, "layers", "Layers"),
                (RightPanel::Fields, "text-cursor-input", "Form fields"),
                (RightPanel::Signatures, "signature", "Signatures"),
                (RightPanel::Tags, "tag", "Tags"),
                (RightPanel::Comments, "message-square-text", "Comments"),
            ] {
                let selected = app.right == Some(panel);
                if icons::button(ui, icon, 32.0, selected, tx(tip)).clicked() {
                    app.choose_right_panel(if selected { None } else { Some(panel) });
                }
            }
        });
    if app.right.is_some_and(|p| !on_the_right(p)) {
        egui::Panel::left("wps_nav")
            .resizable(true)
            .default_size(230.0)
            .size_range(170.0..=420.0)
            .frame(egui::Frame::NONE.fill(t.panel))
            .show(ui, |ui| crate::panels::right_panel(app, ui));
    }
}

/// The panes on the right: comments, search, compare or accessibility results; and the current
/// tool's options (Edit a PDF, Fill & Sign, Prepare a form, Redact, Organize pages…).
fn panes(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    let tool = app.left_open && (app.left != LeftPanel::AllTools || app.active.and_then(|i| app.views.get(i)).is_some_and(|v| v.organize));
    if tool {
        egui::Panel::right("wps_tool_pane")
            .resizable(true)
            .default_size(290.0)
            .size_range(240.0..=480.0)
            .frame(egui::Frame::NONE.fill(t.panel))
            .show(ui, |ui| crate::panels::left_panel(app, ui));
    }
    if app.right.is_some_and(on_the_right) {
        egui::Panel::right("wps_right_pane")
            .resizable(true)
            .default_size(320.0)
            .size_range(260.0..=520.0)
            .frame(egui::Frame::NONE.fill(t.panel))
            .show(ui, |ui| crate::panels::right_panel(app, ui));
    }
}

/// The status bar: the page (n / N, with the page box), the page buttons, the zoom.
fn status_bar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    let Some((index, id)) = app.active_ids() else { return };
    let Some(doc) = app.session.get(id) else { return };
    let labels: Vec<String> = doc.info.pages.iter().map(|p| p.label.clone()).collect();
    let count = labels.len();
    egui::Panel::bottom("wps_status")
        .exact_size(STATUS_H)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(10, 0)).stroke(Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            let Some(view) = app.views.get_mut(index) else { return };
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(egui::RichText::new(tx("Page")).font(theme::regular(12.0)).color(t.text_muted));
                let page_input_id = view.page_input_id();
                let reselect = std::mem::take(&mut view.page_input_reselect);
                if !ui.memory(|m| m.has_focus(page_input_id)) {
                    view.page_input = view.current_page_label(labels.get(view.current).map(String::as_str));
                } else if reselect {
                    let mut state = egui::TextEdit::load_state(ui.ctx(), page_input_id).unwrap_or_default();
                    let end = egui::text::CCursor::new(view.page_input.chars().count());
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), end)));
                    state.store(ui.ctx(), page_input_id);
                }
                let edit = egui::TextEdit::singleline(&mut view.page_input)
                    .id(page_input_id)
                    .desired_width(34.0)
                    .horizontal_align(egui::Align::Center)
                    .font(theme::medium(12.0))
                    .margin(vec2(2.0, 2.0));
                let r = ui.add(edit).on_hover_text(tx("Current page — type a page number or label (such as iv) and press Enter"));
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let typed = view.page_input.clone();
                        view.go_to_typed(&typed, &labels);
                    }
                    view.page_input = view.current_page_label(labels.get(view.current).map(String::as_str));
                }
                ui.label(egui::RichText::new(format!("/ {count}")).font(theme::regular(12.0)).color(t.text_muted));
                ui.add_space(4.0);
                if icons::button(ui, "chevron-up", 22.0, false, tx("Previous page")).clicked() {
                    view.step_page(false);
                }
                if icons::button(ui, "chevron-down", 22.0, false, tx("Next page")).clicked() {
                    view.step_page(true);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if icons::button(ui, "maximize-2", 22.0, view.fit == Fit::Page, tx("Zoom to page level")).clicked() {
                        view.set_fit(Fit::Page);
                    }
                    if icons::button(ui, "maximize", 22.0, view.fit == Fit::Width, tx("Fit to width")).clicked() {
                        view.set_fit(Fit::Width);
                    }
                    ui.add_space(6.0);
                    if icons::button(ui, "zoom-in", 22.0, false, tx("Zoom in")).clicked() {
                        view.zoom_step(true);
                    }
                    let zoom = if view.zoom.is_finite() { (view.zoom * 100.0).round().clamp(0.0, 100_000.0) as u32 } else { 100 };
                    let resp = ui.add(
                        egui::Button::new(egui::RichText::new(format!("{zoom}%")).font(theme::medium(12.0)).color(t.text))
                            .frame(false)
                            .min_size(vec2(46.0, 20.0)),
                    );
                    egui::Popup::menu(&resp).show(|ui| {
                        for pct in [50u32, 75, 100, 125, 150, 200, 400] {
                            if ui.button(format!("{pct}%")).clicked() {
                                view.set_zoom(pct as f32 / 100.0);
                                ui.close();
                            }
                        }
                    });
                    if icons::button(ui, "zoom-out", 22.0, false, tx("Zoom out")).clicked() {
                        view.zoom_step(false);
                    }
                });
            });
        });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    type H = egui_kittest::Harness<'static, PdfCraftApp>;

    thread_local! {
        /// Makes handing a stray document to the host fail (`stray_bytes`).
        pub(crate) static FAIL_STRAY_SAVES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        /// How many times a stray document was opened again to check it (`stray_bytes`).
        pub(crate) static STRAY_PROBES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        /// Warnings logged on this thread since [`capture_logs`].
        static LOGGED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    /// The tests' logger: keeps each thread's warnings for that thread's test.
    struct Capture;

    impl log::Log for Capture {
        fn enabled(&self, m: &log::Metadata) -> bool {
            m.level() <= log::Level::Warn
        }
        fn log(&self, r: &log::Record) {
            if self.enabled(r.metadata()) {
                LOGGED.with(|l| l.borrow_mut().push(r.args().to_string()));
            }
        }
        fn flush(&self) {}
    }

    static CAPTURE: Capture = Capture;

    /// Whether the tests' logger is the process's (installed once, by the first test that
    /// captures; `log` allows one logger per process).
    static INSTALLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    /// Keeps the warnings this thread logs from now on ([`logged`]). Fails the test when another
    /// logger got the process first: nothing would be captured, and an assertion that a warning
    /// is absent would pass for nothing.
    pub(crate) fn capture_logs() {
        let installed = *INSTALLED.get_or_init(|| {
            let ok = log::set_logger(&CAPTURE).is_ok();
            if ok {
                log::set_max_level(log::LevelFilter::Warn);
            }
            ok
        });
        assert!(installed, "another logger is installed: the tests' logger can't capture warnings");
        assert!(log::max_level() >= log::LevelFilter::Warn, "warnings are filtered out");
        LOGGED.with(|l| l.borrow_mut().clear());
    }

    /// The warnings this thread logged since [`capture_logs`].
    pub(crate) fn logged() -> Vec<String> {
        LOGGED.with(|l| l.borrow().clone())
    }

    /// A two-page PDF (A4) that says "Xin chào" on each page.
    fn fixture() -> Vec<u8> {
        let mut objs: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into()];
        objs.push("<< /Type /Pages /Kids [4 0 R 6 0 R] /Count 2 /MediaBox [0 0 595 842] >>".into());
        objs.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into());
        for i in 0..2 {
            objs.push(format!("<< /Type /Page /Parent 2 0 R /Contents {} 0 R /Resources << /Font << /F1 3 0 R >> >> >>", 5 + 2 * i));
            let body = format!("BT /F1 24 Tf 72 700 Td (Trang {}) Tj ET", i + 1);
            objs.push(format!("<< /Length {} >>\nstream\n{body}\nendstream", body.len()));
        }
        let mut out = b"%PDF-1.7\n".to_vec();
        let mut offs = Vec::new();
        for (i, o) in objs.iter().enumerate() {
            offs.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
        }
        let x = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
        for o in offs {
            out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{x}\n%%EOF\n", objs.len() + 1).as_bytes());
        out
    }

    /// A document in the WPS layout, embedded as a tab of the NexusOffice window, in `lang`.
    fn harness(size: (f32, f32), lang: &str, setup: impl FnOnce(&mut PdfCraftApp)) -> H {
        let mut a = PdfCraftApp::new();
        a.wps = true;
        a.embedded = true;
        a.system_clipboard = false;
        a.run_inline = true;
        a.set_option("language", lang).unwrap();
        a.open_bytes("Hợp đồng.pdf", None, fixture()).unwrap();
        a.right = Some(RightPanel::Pages);
        setup(&mut a);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(size.0, size.1)).with_max_steps(64).build_ui_state(
            |ui, app: &mut PdfCraftApp| {
                let ctx = ui.ctx().clone();
                app.frame_logic(&ctx);
                app.frame_ui(ui);
            },
            a,
        );
        h.run_steps(6);
        h
    }

    fn click_id(h: &mut H, id: Id) {
        let r = h.ctx.read_response(id).map(|r| r.rect);
        let r = r.unwrap_or_else(|| panic!("{id:?} is not drawn"));
        h.input_mut().events.push(egui::Event::PointerMoved(r.center()));
        h.run_steps(1);
        for pressed in [true, false] {
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.run_steps(1);
        }
        h.run_steps(4);
    }

    /// Every tab draws its strip at both window sizes within the strip's room (two rows of small
    /// buttons high); what doesn't fit folds into "…", whose panel holds the rest. Every button is
    /// a registry command (or one of the strip's own) with an icon the app ships.
    #[test]
    fn every_tab_fits_the_strip_at_both_sizes() {
        for size in [(1440.0, 900.0), (1024.0, 700.0)] {
            let mut h = harness(size, "vi", |_| {});
            for tab in WpsTab::ALL {
                h.state_mut().wps_tab = tab;
                h.run_steps(3);
                let fit = strip_fit(&h.ctx).expect("the strip was drawn");
                assert_eq!(fit.tab, tab);
                assert!(fit.shown >= 1, "{tab:?} at {size:?} shows a group");
                assert!(fit.used <= fit.room + 0.5, "{tab:?} at {size:?}: {fit:?}");
                let folded = h.ctx.read_response(overflow_button_id()).is_some();
                assert_eq!(folded, fit.shown < fit.groups, "{tab:?} at {size:?}: \"…\" exactly when groups fold");
                if folded {
                    click_id(&mut h, overflow_button_id());
                    assert!(overflow_open(&h.ctx), "{tab:?}: the panel opens");
                    h.key_press(egui::Key::Escape);
                    h.run_steps(3);
                }
                for g in groups(tab) {
                    for item in *g {
                        assert!(item.id.starts_with("nx.") || pdfcraft_engine::commands::command(item.id).is_some(), "{} is a command", item.id);
                        let (icon, _) = icon_and_tip(item.id);
                        assert!(icons::exists(icon), "{icon}");
                    }
                }
            }
            if size.0 >= 1440.0 {
                h.state_mut().wps_tab = WpsTab::Home;
                h.run_steps(3);
                let fit = strip_fit(&h.ctx);
                assert!(fit.is_some_and(|f| f.shown == f.groups), "Home fits a 1440 pt window: {fit:?}");
            }
        }
    }

    /// Group widths add up to the strip's room: "…" appears only when they don't fit.
    #[test]
    fn groups_and_tabs_fit_from_the_first_and_junk_widths_are_harmless() {
        assert_eq!(fit_groups(&[], 100.0), usize::MAX);
        assert_eq!(fit_groups(&[40.0, 40.0], 100.0), usize::MAX);
        assert_eq!(fit_groups(&[40.0, 40.0, 40.0], 100.0), 2);
        assert_eq!(fit_groups(&[120.0, 10.0], 100.0), 0);
        assert_eq!(fit_groups(&[f32::NAN, f32::INFINITY, 40.0], 100.0), usize::MAX, "junk widths count as 0");
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false)], 100.0, 20.0), [true, true]);
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, false)], 100.0, 20.0), [true, true, false]);
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, true)], 100.0, 20.0), [true, false, true], "the current tab stays");
        assert_eq!(fit_tabs(&[(f32::NAN, false), (f32::INFINITY, true)], 10.0, 20.0), [true, true]);
        let cols = columns(&[big("a", "A"), small("b", "B"), small("c", "C"), small("d", "D"), big("e", "E")]);
        assert_eq!(cols.iter().map(Vec::len).collect::<Vec<_>>(), [1, 2, 1, 1]);
    }

    /// The top row speaks Vietnamese; File (Tệp) opens the File page; Escape there goes back.
    #[test]
    fn file_opens_the_file_page() {
        let mut h = harness((1440.0, 900.0), "vi", |_| {});
        assert!(h.query_by_label("Trang chủ").is_some(), "the Home tab, in Vietnamese");
        assert!(h.query_by_label("Chỉnh sửa").is_some());
        h.get_by_label("Tệp").click();
        h.run_steps(4);
        assert_eq!(h.state().backstage, Some(crate::nexus_backstage::Page::Info), "File opens the File page");
        assert!(h.query_by_label("Lưu thành…").is_some(), "Save As is on the File page");
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        assert_eq!(h.state().backstage, None);
    }

    /// The Quick Access Toolbar: Undo and Redo work on an edit, and Save writes the file; the
    /// save state follows.
    #[test]
    fn quick_access_undo_redo_and_save_work() {
        let dir = std::env::temp_dir().join(format!("nexus-pdf-wps-qat-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Biên bản.pdf");
        std::fs::write(&path, fixture()).unwrap();
        let p = path.to_string_lossy().into_owned();
        let mut h = harness((1440.0, 900.0), "vi", |a| {
            a.close_tab(0);
            a.open_path(&p);
        });
        assert_eq!(h.state().views.len(), 1);
        assert!(h.query_by_label("Đã lưu").is_some(), "a file just opened is saved");
        h.state_mut().apply_edit(pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 });
        h.run_steps(3);
        assert!(h.query_by_label("Chưa lưu").is_some(), "an edit is not saved");
        h.get_by_label("Hoàn tác").click();
        h.run_steps(3);
        let rotated = |h: &H| h.state().session.get(h.state().views[0].id).map(|d| i32::from(d.info.pages[0].rotation)).unwrap_or(-1);
        assert_eq!(rotated(&h), 0, "Undo undid the rotation");
        h.get_by_label("Làm lại").click();
        h.run_steps(3);
        assert_eq!(rotated(&h), 90, "Redo redid it");
        h.get_by_label("Lưu").click();
        h.run_steps(4);
        assert!(!h.state().session.get(h.state().views[0].id).unwrap().dirty, "Save wrote the file");
        assert!(h.query_by_label("Đã lưu").is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// At 1024 pt the Comment tab's last groups fold into "…": its panel shows them and their
    /// buttons run (Comment list opens the comments pane); Escape closes the panel.
    #[test]
    fn narrow_windows_fold_groups_into_the_overflow_panel() {
        let wide = harness((1920.0, 900.0), "en", |a| a.wps_tab = WpsTab::Comment);
        assert!(wide.ctx.read_response(overflow_button_id()).is_none(), "everything fits a wide window");
        let mut h = harness((1024.0, 700.0), "en", |a| {
            a.wps_tab = WpsTab::Comment;
            a.right = None;
        });
        assert!(h.query_by_label("Comment list").is_none(), "the last group folded away");
        click_id(&mut h, overflow_button_id());
        assert!(overflow_open(&h.ctx));
        h.get_by_label("Comment list").click();
        h.run_steps(4);
        assert_eq!(h.state().right, Some(RightPanel::Comments), "the folded group's button ran");
        assert!(!overflow_open(&h.ctx), "the panel closes once a button ran");
        click_id(&mut h, overflow_button_id());
        assert!(overflow_open(&h.ctx));
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        assert!(!overflow_open(&h.ctx), "Escape closes the panel");
    }

    /// Tabs switch, a click on the current tab folds the strip away, and » holds the rest in a
    /// narrow window; Search opens the command palette.
    #[test]
    fn tabs_search_and_more_tabs_are_reachable() {
        let mut h = harness((1440.0, 900.0), "vi", |_| {});
        h.get_by_label("Bảo vệ").click();
        h.run_steps(3);
        assert_eq!(h.state().wps_tab, WpsTab::Protect);
        h.get_by_label("Bảo vệ").click();
        h.run_steps(3);
        assert!(h.state().wps_collapsed, "a click on the current tab folds the strip");
        click_id(&mut h, search_id());
        assert!(h.state().palette_open, "the box opens the command palette");
        let mut n = harness((1024.0, 700.0), "vi", |_| {});
        assert!(n.query_by_label("Công cụ").is_none(), "Tools doesn't fit");
        click_id(&mut n, more_tabs_id());
        n.get_by_label("Công cụ").click();
        n.run_steps(3);
        assert_eq!(n.state().wps_tab, WpsTab::Tools);
    }

    /// Embedded, the app never sets the window title; stand-alone it does.
    #[test]
    fn embedded_leaves_the_window_title_alone() {
        let hosted = harness((1024.0, 700.0), "en", |_| {});
        assert!(hosted.state().window_title.is_empty(), "a hosted app never names the window");
        let alone = harness((1024.0, 700.0), "en", |a| a.embedded = false);
        assert!(alone.state().window_title.contains("Hợp đồng.pdf"), "{}", alone.state().window_title);
    }

    /// Another editor in the host window replaced the window's fonts (without this app's named
    /// families). A host that draws the tab before its `logic` (a tab brought forward mid-frame)
    /// must not make it panic; `logic` then installs the fonts again.
    #[test]
    fn replaced_fonts_are_installed_again_without_a_panic() {
        let mut h = harness((1024.0, 700.0), "vi", |_| {});
        let mut ui_only = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1024.0, 700.0))
            .build_ui_state(|ui, app: &mut PdfCraftApp| app.frame_ui(ui), std::mem::take(h.state_mut()));
        // This window's fonts are another editor's (egui's own set, no "medium" or "semibold").
        assert!(!theme::fonts_installed(&ui_only.ctx));
        ui_only.run_steps(3);
        assert!(!ui_only.state().fonts_ready, "the app noticed and asked for its fonts again");
        *h.state_mut() = std::mem::take(ui_only.state_mut());
        h.ctx.set_fonts(egui::FontDefinitions::default());
        h.run_steps(4);
        assert!(theme::fonts_installed(&h.ctx), "logic installed them again");
        assert!(h.state().fonts_ready);
    }

    /// Hooks: File › New and an opened file go to the host while this app has its document; a
    /// document made here goes to the host by its bytes and leaves this app.
    #[test]
    fn new_open_and_made_documents_go_to_the_host() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let log: Rc<RefCell<Vec<String>>> = Rc::default();
        let (new_log, open_log, adopt_log) = (log.clone(), log.clone(), log.clone());
        let mut h = harness((1440.0, 900.0), "vi", move |a| {
            a.host.new_document = Some(Box::new(move || new_log.borrow_mut().push("new".into())));
            a.host.open_document = Some(Box::new(move |p| open_log.borrow_mut().push(format!("open {p}"))));
            a.host.adopt_document = Some(Box::new(move |doc| {
                adopt_log.borrow_mut().push(format!("adopt {} {}", doc.name, doc.bytes.len() > 100));
                Ok::<(), String>(())
            }));
        });
        h.state_mut().execute("create.blank");
        h.run_steps(2);
        h.state_mut().open_path("/không/có/thật.pdf");
        h.run_steps(2);
        // A document made here (as Create from clipboard or Combine do).
        let bytes = h.state().session.create_blank(200.0, 200.0, 1).unwrap();
        let id = h.state_mut().session.open_new("Ảnh.pdf", bytes).unwrap();
        let info = h.state().session.get(id).unwrap().info.clone();
        let defaults = h.state().view_defaults;
        h.state_mut().views.push(crate::DocView::new(id, &info, defaults));
        h.state_mut().active = Some(1);
        h.run_steps(3);
        assert_eq!(*log.borrow(), ["new", "open /không/có/thật.pdf", "adopt Ảnh.pdf true"]);
        assert_eq!(h.state().active, Some(0), "the tab's own document is shown");
        // Its tab opened: it leaves this app.
        h.state_mut().adoption_answered(id, Ok(()));
        assert_eq!(h.state().views.len(), 1, "only the tab's own document stays");
        assert_eq!(h.state().active, Some(0));
    }

    /// Save As through the host's dialog offers PDF only, and saves where it answers.
    #[test]
    fn save_as_asks_the_host_for_a_pdf() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let dir = std::env::temp_dir().join(format!("nexus-pdf-wps-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("Báo giá");
        let answer = target.to_string_lossy().into_owned();
        let asked: Rc<RefCell<Vec<FileDialogRequest>>> = Rc::default();
        let log = asked.clone();
        let mut h = harness((1440.0, 900.0), "vi", move |a| {
            a.host.file_dialog = Some(Box::new(move |req| {
                log.borrow_mut().push(req);
                let (tx, rx) = std::sync::mpsc::channel();
                tx.send(Ok(Some(answer.clone()))).unwrap();
                rx
            }));
        });
        h.state_mut().execute("file.save_as");
        for _ in 0..200 {
            h.run_steps(1);
            if dir.join("Báo giá.pdf").exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(*asked.borrow(), [FileDialogRequest::Save { name: "Hợp đồng.pdf".into(), types: vec!["pdf".into()] }]);
        assert!(dir.join("Báo giá.pdf").is_file(), "saved where the host said, as a PDF");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn snapshot_files(dir: &std::path::Path) -> usize {
        std::fs::read_dir(dir).map(|r| r.filter_map(Result::ok).filter(|e| e.path().extension().is_some_and(|x| x == "pdf")).count()).unwrap_or(0)
    }

    /// A recovered (or otherwise unsaved) document handed to the host keeps its recovery copy on
    /// disk and its file: the tab that takes it owns the copy (same key), Save writes back to the
    /// file, and then the copy goes. (The copy used to be deleted at the handoff, and the new tab
    /// had none for its first minute; Save asked where to save.)
    #[test]
    fn a_handed_over_document_keeps_its_recovery_copy_and_file() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let dir = std::env::temp_dir().join(format!("nexus-pdf-handoff-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("recovery")).unwrap();
        let file = dir.join("Báo cáo quý 3.pdf");
        std::fs::write(&file, fixture()).unwrap();
        let path = file.to_string_lossy().into_owned();
        let handed: Rc<RefCell<Vec<AdoptedDocument>>> = Rc::default();
        let log = handed.clone();
        let store = crate::RecoveryStore::new(dir.join("recovery"));
        let mut h = harness((1024.0, 700.0), "vi", move |a| {
            a.enable_autosave(store);
            a.host.adopt_document = Some(Box::new(move |doc| {
                log.borrow_mut().push(doc);
                Ok::<(), String>(())
            }));
        });
        // A second document with unsaved changes at its file (as File › Recover opens one), and
        // its snapshot written at once.
        let second = AdoptedDocument {
            name: "Báo cáo quý 3.pdf".into(),
            path: Some(path.clone()),
            bytes: std::sync::Arc::new(fixture()),
            recovery_key: None,
            source: None,
        };
        h.state_mut().open_adopted(second).unwrap();
        assert_eq!(h.state().views.len(), 2);
        assert_eq!(snapshot_files(&dir.join("recovery")), 1, "its copy is written as it opens");
        h.run_steps(2);
        let doc = handed.borrow_mut().pop().expect("the host took it");
        assert_eq!(doc.path.as_deref(), Some(path.as_str()), "with its file");
        assert!(doc.recovery_key.is_some(), "with its recovery key");
        h.state_mut().adoption_answered(doc.source.expect("answered for"), Ok(()));
        assert_eq!(h.state().views.len(), 1, "handed over");
        assert_eq!(snapshot_files(&dir.join("recovery")), 1, "the copy stays on disk across the handoff");
        // The tab that takes it.
        let mut b = PdfCraftApp::new();
        b.embed();
        b.enable_autosave(crate::RecoveryStore::new(dir.join("recovery")));
        b.open_adopted(doc).unwrap();
        assert_eq!(b.first_dirty(), Some(0), "unsaved there");
        assert_eq!(b.session.get(b.views[0].id).unwrap().path.as_deref(), Some(path.as_str()));
        assert!(b.save_active(crate::SaveTarget::InPlace), "Save writes to the file without asking");
        assert_eq!(snapshot_files(&dir.join("recovery")), 0, "saved: the copy goes");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A document that can't be handed to the host stays open here (its changes kept, closing
    /// the tab asks about it), and the failure is said once, not every frame.
    #[test]
    fn a_document_that_cant_be_handed_over_stays_and_is_reported_once() {
        let mut h = harness((1024.0, 700.0), "en", |a| {
            a.host.adopt_document = Some(Box::new(|_| Ok::<(), String>(())));
        });
        h.state_mut().open_unsaved("Scan.pdf", std::sync::Arc::new(fixture())).unwrap();
        let id = h.state().views[1].id;
        FAIL_STRAY_SAVES.with(|f| f.set(true));
        h.run_steps(1);
        let first = h.state().toast.clone();
        h.state_mut().toast = None;
        h.run_steps(4);
        FAIL_STRAY_SAVES.with(|f| f.set(false));
        assert!(first.is_some_and(|(t, _)| t.contains("Scan.pdf")), "the failure is reported");
        assert!(h.state().toast.is_none(), "once");
        assert!(h.state().views.iter().any(|v| v.id == id), "the document stays");
        assert!(h.state().views.iter().position(|v| v.id == id).is_some_and(|i| h.state().has_unsaved_work(i)), "with its changes");
    }

    /// Why a document couldn't move is said in Vietnamese in a Vietnamese interface, the
    /// engine's English reason left out (it read "…ở lại tab này: the disk is full").
    #[test]
    fn a_document_that_cant_be_handed_over_is_reported_in_vietnamese() {
        let mut h = harness((1024.0, 700.0), "vi", |a| {
            a.host.adopt_document = Some(Box::new(|_| Ok::<(), String>(())));
        });
        h.state_mut().open_unsaved("Bản quét.pdf", std::sync::Arc::new(fixture())).unwrap();
        capture_logs();
        FAIL_STRAY_SAVES.with(|f| f.set(true));
        h.run_steps(1);
        FAIL_STRAY_SAVES.with(|f| f.set(false));
        let toast = h.state().toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert_eq!(toast, "Không chuyển được “Bản quét.pdf” sang tab riêng nên tài liệu ở lại tab này: không chuẩn bị được tài liệu để mở ở tab mới");
        // The engine's own reason (the save failed as a full disk fails it) goes to the log.
        let logs = logged();
        assert!(logs.iter().any(|l| l.contains("could not be written: the disk is full")), "{logs:?}");
    }

    /// A document that isn't encrypted goes to the host without being opened again to check it
    /// (a whole extra parse on the UI thread); an encrypted one is still checked.
    #[test]
    fn only_an_encrypted_document_is_reopened_before_it_is_handed_over() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let handed: Rc<RefCell<Vec<String>>> = Rc::default();
        let log = handed.clone();
        let mut h = harness((1024.0, 700.0), "en", move |a| {
            a.host.adopt_document = Some(Box::new(move |doc| {
                log.borrow_mut().push(doc.name);
                Ok::<(), String>(())
            }));
        });
        h.state_mut().open_unsaved("Scan.pdf", std::sync::Arc::new(fixture())).unwrap();
        STRAY_PROBES.with(|n| n.set(0));
        h.run_steps(1);
        assert_eq!(*handed.borrow(), ["Scan.pdf".to_string()], "handed over");
        assert_eq!(STRAY_PROBES.with(std::cell::Cell::get), 0, "without opening it again");
        let a = h.state_mut();
        let id = a.session.open("Mật.pdf", None, std::sync::Arc::new(protected()), Some("matkhau")).unwrap();
        let view = crate::DocView::new(id, &a.session.get(id).unwrap().info, a.view_defaults);
        a.views.push(view);
        h.run_steps(1);
        assert_eq!(STRAY_PROBES.with(std::cell::Cell::get), 1, "an encrypted one is checked");
        assert!(h.state().views.iter().any(|v| v.id == id), "and stays: it needs its password");
    }

    /// The fixture, protected with the open password "matkhau" (written by the engine).
    fn protected() -> Vec<u8> {
        let mut s = pdfcraft_engine::Session::new();
        let id = s.open("a.pdf", None, std::sync::Arc::new(fixture()), None).unwrap();
        let protect = pdfcraft_engine::Protection { open_password: Some("matkhau".into()), ..Default::default() };
        s.apply(id, pdfcraft_engine::Edit::Protect(protect)).unwrap();
        s.save_bytes(id).unwrap().to_vec()
    }

    /// A document unlocked here with its password (a recovered encrypted PDF) would not open in a
    /// new tab without it: it stays in this tab instead of vanishing, the user is told (in the
    /// interface language) that it couldn't move, and it can be shown and saved from here.
    #[test]
    fn a_document_needing_its_password_stays_and_can_be_reached() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let handed: Rc<RefCell<Vec<String>>> = Rc::default();
        let log = handed.clone();
        let mut h = harness((1024.0, 700.0), "vi", move |a| {
            a.host.adopt_document = Some(Box::new(move |doc| {
                log.borrow_mut().push(doc.name);
                Ok::<(), String>(())
            }));
        });
        let a = h.state_mut();
        let id = a.session.open("Mật.pdf", None, std::sync::Arc::new(protected()), Some("matkhau")).unwrap();
        let view = crate::DocView::new(id, &a.session.get(id).unwrap().info, a.view_defaults);
        a.views.push(view);
        h.run_steps(2);
        assert!(handed.borrow().is_empty(), "not handed to a tab that can't open it: {:?}", handed.borrow());
        assert!(h.state().views.iter().any(|v| v.id == id), "it stays here");
        let toast = h.state().toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert!(toast.contains("Không chuyển được “Mật.pdf” sang tab riêng") && toast.contains("cần mật khẩu"), "{toast}");
        // Reachable: the bar names it, and showing it keeps it shown.
        assert!(h.query_by_label("Mật.pdf").is_some(), "the bar lists it");
        h.get_by_label("Mật.pdf").click();
        h.run_steps(3);
        assert_eq!(h.state().active_ids().map(|(_, i)| i), Some(id), "shown, and it stays shown");
    }

    /// A tab that isn't shown still hands its strays over (a recovered document finishing
    /// opening while another tab is shown).
    #[test]
    fn a_hidden_tab_hands_its_documents_over() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let handed: Rc<RefCell<Vec<String>>> = Rc::default();
        let log = handed.clone();
        let mut a = PdfCraftApp::new();
        a.embed();
        a.run_inline = true;
        a.open_bytes("Hợp đồng.pdf", None, fixture()).unwrap();
        let sources: Rc<RefCell<Vec<pdfcraft_engine::DocId>>> = Rc::default();
        let answers = sources.clone();
        a.host.adopt_document = Some(Box::new(move |doc| {
            log.borrow_mut().push(doc.name);
            answers.borrow_mut().extend(doc.source);
            Ok::<(), String>(())
        }));
        a.open_unsaved("Khôi phục 1.pdf", std::sync::Arc::new(fixture())).unwrap();
        a.open_unsaved("Khôi phục 2.pdf", std::sync::Arc::new(fixture())).unwrap();
        let ctx = egui::Context::default();
        a.background_frame(&ctx);
        assert_eq!(*handed.borrow(), vec!["Khôi phục 1.pdf".to_string(), "Khôi phục 2.pdf".to_string()], "both, in one frame");
        assert!(a.handing_over(), "until the host answers");
        for id in sources.borrow().iter() {
            a.adoption_answered(*id, Ok(()));
        }
        assert_eq!(a.views.len(), 1);
        assert!(!a.handing_over());
    }

    /// A document the host's hook refuses (it can't take it now) is kept here at once, not left
    /// waiting for an answer that will never come, and the user is told.
    #[test]
    fn a_document_the_host_refuses_is_kept_at_once() {
        let mut h = harness((1024.0, 700.0), "vi", |a| {
            a.host.adopt_document = Some(Box::new(|_| Err("tab đang bận".to_string())));
        });
        h.state_mut().open_unsaved("Gộp.pdf", std::sync::Arc::new(fixture())).unwrap();
        let id = h.state().views[1].id;
        h.run_steps(2);
        assert!(!h.state().handing_over(), "not waiting: the tab can close");
        assert_eq!(h.state().kept_documents(), 1, "kept");
        assert!(h.state().views.iter().any(|v| v.id == id));
        let toast = h.state().toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert_eq!(toast, "Không chuyển được “Gộp.pdf” sang tab riêng nên tài liệu ở lại tab này: tab đang bận");
    }

    /// A handover the host never answers counts as refused after [`HANDOVER_FRAMES`] frames: the
    /// document is kept here and the tab can close again (it waited for good, and every close of
    /// the tab or the window stopped there).
    #[test]
    fn a_handover_never_answered_gives_up_and_keeps_the_document() {
        let mut a = PdfCraftApp::new();
        a.embed();
        a.run_inline = true;
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        a.open_bytes("Hợp đồng.pdf", None, fixture()).unwrap();
        a.host.adopt_document = Some(Box::new(|_| Ok::<(), String>(())));
        a.open_unsaved("Gộp.pdf", std::sync::Arc::new(fixture())).unwrap();
        let ctx = egui::Context::default();
        a.background_frame(&ctx);
        assert!(a.handing_over(), "waiting for the answer");
        for _ in 0..HANDOVER_FRAMES {
            a.background_frame(&ctx);
        }
        assert!(!a.handing_over(), "no answer came: it stops waiting");
        assert_eq!(a.kept_documents(), 1, "the document is kept here");
        assert_eq!(a.views.len(), 2);
        let toast = a.toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert!(toast.ends_with("cửa sổ không mở tab mới cho tài liệu"), "{toast}");
        crate::i18n::set_current(crate::i18n::Lang::EN);
    }

    /// A document handed over and kept here after [`HANDOVER_FRAMES`] frames without an answer:
    /// the app, the document's id, the recovery key it went with, the recovery folder.
    fn given_up(tag: &str) -> (PdfCraftApp, pdfcraft_engine::DocId, String, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("nexus-pdf-late-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut a = PdfCraftApp::new();
        a.embed();
        a.run_inline = true;
        a.enable_autosave(crate::RecoveryStore::new(dir.clone()));
        a.open_bytes("Hợp đồng.pdf", None, fixture()).unwrap();
        let handed: std::rc::Rc<std::cell::RefCell<Vec<AdoptedDocument>>> = Default::default();
        let log = handed.clone();
        a.host.adopt_document = Some(Box::new(move |doc| {
            log.borrow_mut().push(doc);
            Ok::<(), String>(())
        }));
        let recovered =
            AdoptedDocument { name: "Gộp.pdf".into(), path: None, bytes: std::sync::Arc::new(fixture()), recovery_key: None, source: None };
        a.open_adopted(recovered).unwrap();
        assert_eq!(snapshot_files(&dir), 1, "its copy is written as it opens");
        let ctx = egui::Context::default();
        a.background_frame(&ctx);
        let doc = handed.borrow_mut().pop().expect("handed to the host");
        let (id, key) = (doc.source.unwrap(), doc.recovery_key.unwrap());
        for _ in 0..HANDOVER_FRAMES {
            a.background_frame(&ctx);
        }
        assert!(!a.handing_over());
        assert_eq!(a.kept_documents(), 1, "kept here");
        (a, id, key, dir)
    }

    /// A host that says Ok after the app stopped waiting (it broke the same-frame contract) is
    /// logged, and the document kept meanwhile, unchanged since, closes here: the new tab has it
    /// as it is, with its recovery copy (which stays on disk). It used to stay open here too,
    /// under the same recovery key as the new tab.
    #[test]
    fn a_late_ok_closes_the_kept_copy_when_unchanged() {
        capture_logs();
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let (mut a, id, key, dir) = given_up("same");
        a.adoption_answered(id, Ok(()));
        assert!(!a.views.iter().any(|v| v.id == id), "the copy here closed");
        assert_eq!(a.views.len(), 1);
        assert_eq!(a.kept_documents(), 0);
        assert!(dir.join(format!("{key}.pdf")).exists(), "the new tab's recovery copy stays");
        assert_eq!(snapshot_files(&dir), 1);
        let toast = a.toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert_eq!(toast, "“Gộp.pdf” đã mở được ở tab riêng nên được đóng ở tab này.");
        assert!(logged().iter().any(|l| l.contains("after 120 frames")), "{:?}", logged());
        // A second late answer: nothing waits for it, logged and ignored.
        a.adoption_answered(id, Ok(()));
        assert_eq!(a.views.len(), 1);
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A late Ok for a document changed here since it was handed over: both stay (the changes
    /// here aren't lost), and the one here gets a recovery copy of its own, written at once, so
    /// the two tabs never write over one copy.
    #[test]
    fn a_late_ok_keeps_both_when_changed_with_their_own_recovery_copies() {
        capture_logs();
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let (mut a, id, key, dir) = given_up("changed");
        a.session.apply(id, pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 }).unwrap();
        a.adoption_answered(id, Ok(()));
        assert!(a.views.iter().any(|v| v.id == id), "kept: it was changed here");
        assert_eq!(a.kept_documents(), 1);
        let own = a.recovery_keys.get(&id).cloned().expect("a recovery key");
        assert_ne!(own, key, "a key of its own");
        assert!(dir.join(format!("{key}.pdf")).exists(), "the new tab's copy stays");
        assert!(dir.join(format!("{own}.pdf")).exists(), "its own copy, written at once");
        assert_eq!(snapshot_files(&dir), 2);
        let toast = a.toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert!(toast.contains("giữ cả hai"), "{toast}");
        assert!(logged().iter().any(|l| l.contains("after 120 frames")), "{:?}", logged());
        // Closing it here removes its own copy only, never the new tab's.
        let i = a.views.iter().position(|v| v.id == id).unwrap();
        a.close_tab(i);
        assert!(dir.join(format!("{key}.pdf")).exists());
        assert!(!dir.join(format!("{own}.pdf")).exists());
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A late Ok while the user is typing in the kept copy (Fill & Sign text not in the document
    /// yet: no engine edit counts it) keeps both: closing it would drop the text without a word.
    #[test]
    fn a_late_ok_keeps_the_kept_copy_while_something_is_being_typed() {
        capture_logs();
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let (mut a, id, key, dir) = given_up("typing");
        let i = a.views.iter().position(|v| v.id == id).unwrap();
        a.views[i].fill_text = Some(crate::fill_sign::TypeBox {
            page: 0,
            at: [72.0, 700.0],
            text: "Nguyễn Văn An".into(),
            size: crate::fill_sign::TEXT_SIZE,
            focus: true,
        });
        a.adoption_answered(id, Ok(()));
        let i = a.views.iter().position(|v| v.id == id).expect("kept: something is being typed in it");
        assert_eq!(a.views[i].fill_text.as_ref().map(|t| t.text.as_str()), Some("Nguyễn Văn An"), "the text is still there");
        assert_eq!(a.kept_documents(), 1);
        assert_ne!(a.recovery_keys.get(&id), Some(&key), "a recovery key of its own");
        assert!(dir.join(format!("{key}.pdf")).exists(), "the new tab's copy stays");
        let toast = a.toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert!(toast.contains("giữ cả hai"), "{toast}");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A late Ok for a changed copy whose own recovery copy can't be written (the folder became a
    /// file): both stay, and the user is told the copy here has no recovery copy yet.
    #[test]
    fn a_late_ok_says_so_when_the_kept_copy_gets_no_recovery_copy() {
        capture_logs();
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let (mut a, id, _key, dir) = given_up("nocopy");
        a.session.apply(id, pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 }).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        std::fs::write(&dir, b"not a folder").unwrap();
        a.adoption_answered(id, Ok(()));
        assert!(a.views.iter().any(|v| v.id == id), "kept");
        let toast = a.toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert!(toast.contains("giữ cả hai") && toast.contains("chưa ghi được bản khôi phục riêng"), "{toast}");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let _ = std::fs::remove_file(&dir);
    }

    /// [`pdfcraft_engine::Session::snapshot_again`] hands an unchanged document out once, and
    /// the next real edit is handed out too (it used to set the snapshot mark one generation
    /// ahead, so the next edit landed on it and was never autosaved).
    #[test]
    fn snapshot_again_never_hides_the_next_edit_from_autosave() {
        let mut s = pdfcraft_engine::Session::new();
        let id = s.open_unsaved("x.pdf", None, std::sync::Arc::new(fixture())).unwrap();
        assert_eq!(s.autosave_snapshots().len(), 1);
        assert!(s.autosave_snapshots().is_empty(), "nothing new");
        s.snapshot_again(id);
        assert_eq!(s.autosave_snapshots().len(), 1, "handed out again");
        assert!(s.autosave_snapshots().is_empty(), "once");
        s.apply(id, pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 }).unwrap();
        assert_eq!(s.autosave_snapshots().len(), 1, "the next edit is autosaved");
        s.snapshot_again(id);
        s.apply(id, pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 }).unwrap();
        assert_eq!(s.autosave_snapshots().len(), 1);
        assert!(s.autosave_snapshots().is_empty());
    }

    /// A document handed to the host stays here, out of sight, until the host says whether its
    /// tab opened: one the new tab couldn't open (bytes that don't parse there) comes back, kept
    /// with its changes and its recovery copy, and the user is told; it isn't handed again. (It
    /// was closed as it was handed over: a document with no recovery copy, made by Combine or
    /// Extract, was lost.)
    #[test]
    fn a_document_whose_new_tab_did_not_open_stays_here() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let dir = std::env::temp_dir().join(format!("nexus-pdf-adopt-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let handed: Rc<RefCell<Vec<AdoptedDocument>>> = Rc::default();
        let log = handed.clone();
        let store = crate::RecoveryStore::new(dir.clone());
        let mut h = harness((1024.0, 700.0), "vi", move |a| {
            a.enable_autosave(store);
            a.host.adopt_document = Some(Box::new(move |doc| {
                log.borrow_mut().push(doc);
                Ok::<(), String>(())
            }));
        });
        // Made here (as Combine makes one): unsaved, no file.
        h.state_mut().open_unsaved("Gộp.pdf", std::sync::Arc::new(fixture())).unwrap();
        let id = h.state().views[1].id;
        h.run_steps(2);
        let doc = handed.borrow_mut().pop().expect("handed to the host");
        assert_eq!(doc.source, Some(id));
        assert!(handed.borrow().is_empty(), "once");
        assert!(h.state().views.iter().any(|v| v.id == id), "still here until the host answers");
        assert_ne!(h.state().active_ids().map(|(_, i)| i), Some(id), "out of sight");
        assert!(h.state().handing_over(), "the tab doesn't close meanwhile");
        h.state_mut().toast = None;
        h.state_mut().adoption_answered(id, Err("tệp bị hỏng hoặc không phải PDF".into()));
        h.run_steps(3);
        assert!(handed.borrow().is_empty(), "not handed again");
        assert!(!h.state().handing_over());
        let i = h.state().views.iter().position(|v| v.id == id).expect("it stays");
        assert!(h.state().has_unsaved_work(i), "with its changes");
        assert_eq!(h.state().kept_documents(), 1, "kept, reachable from the bar");
        assert!(h.query_by_label("Gộp.pdf •").is_some(), "the bar lists it, unsaved");
        assert_eq!(snapshot_files(&dir), 1, "its recovery copy stays");
        let toast = h.state().toast.clone().map(|(t, _)| t).unwrap_or_default();
        assert_eq!(toast, "Không chuyển được “Gộp.pdf” sang tab riêng nên tài liệu ở lại tab này: tệp bị hỏng hoặc không phải PDF");
        // A late or second answer changes nothing.
        h.state_mut().adoption_answered(id, Ok(()));
        assert!(h.state().views.iter().any(|v| v.id == id));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A document id range with no room to count up is refused (ids never wrap around).
    #[test]
    fn an_id_range_without_room_is_refused() {
        let mut s = pdfcraft_engine::Session::new();
        assert!(s.reserve_ids_from(u64::MAX).is_err());
        assert!(s.reserve_ids_from(u64::MAX / 2).is_ok());
        let id = s.open("a.pdf", None, std::sync::Arc::new(fixture()), None).unwrap();
        assert_eq!(id.0, u64::MAX / 2 + 1);
    }

    /// An inline edit of a comment card is unsaved work only once its text differs.
    #[test]
    fn an_unchanged_comment_edit_is_not_unsaved_work() {
        let mut h = harness((1024.0, 700.0), "en", |_| {});
        h.state_mut().views[0].comments.editing = Some((0, 0, String::new()));
        assert!(h.state().first_dirty().is_none(), "opened, nothing typed");
        h.state_mut().views[0].comments.editing = Some((0, 0, "Sửa lại".into()));
        assert_eq!(h.state().first_dirty(), Some(0));
    }

    /// Every tab and strip button reads Vietnamese in a Vietnamese interface (the fork's table or
    /// the shared catalog), and the fork's table has no English left in it.
    #[test]
    fn the_strip_speaks_vietnamese() {
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let keep = ["PDF/A…"];
        let mut english = Vec::new();
        for tab in WpsTab::ALL {
            let mut labels = vec![tab.label()];
            labels.extend(groups(tab).iter().flat_map(|g| g.iter().map(|i| i.label)));
            for label in labels {
                if tx(label) == label && !keep.contains(&label) {
                    english.push(label);
                }
            }
        }
        assert!(english.is_empty(), "{english:?}");
        for (en, vi) in NEXUS_VI {
            assert_ne!(en, vi);
            assert_eq!(vi.trim(), *vi);
        }
        crate::i18n::set_current(crate::i18n::Lang::EN);
    }

    /// "About Nexus PDF" (the ⋯ menu and the File page) names Nexus PDF and still finds the
    /// catalog's translation, which upstream keys with its own name (the label named upstream's
    /// product, which brand-check refuses in the fork's own files).
    #[test]
    fn about_names_nexus_pdf_and_is_translated() {
        use crate::i18n::{Lang, tr};
        assert_eq!(tr(Lang::EN, "About Nexus PDF"), "About Nexus PDF");
        let vi = tr(Lang::from_code("vi").unwrap(), "About Nexus PDF");
        assert!(vi.starts_with("Giới thiệu") && vi.contains("Nexus PDF"), "{vi}");
    }

    /// File's label keeps 4.5:1 on its faint accent fill, in both themes and with chosen accents;
    /// so does "Saved" on the row.
    #[test]
    fn file_label_is_readable_on_its_fill() {
        for kind in [theme::ThemeKind::Light, theme::ThemeKind::Dark] {
            let base = Tokens::for_kind(kind);
            for accent in [None, Some(Color32::from_rgb(0x4F, 0x46, 0xE5)), Some(Color32::from_rgb(0xFA, 0xCC, 0x15))] {
                let t = accent.map_or(base, |c| theme::with_accent(base, c));
                for hovered in [false, true] {
                    let fill = file_fill(&t, hovered);
                    let ratio = theme::contrast_ratio(file_ink(&t, fill), fill);
                    assert!(ratio >= 4.5, "{accent:?} hovered {hovered}: {ratio}");
                }
            }
            assert!(theme::contrast_ratio(saved_color(base.dark()), base.titlebar) >= 4.5);
        }
    }
}
