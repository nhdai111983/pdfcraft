//! NexusOffice layout in the style of WPS Office (its "Universal UI"; Henry's approved mockup,
//! 2026-10-24), the same layout as Nexus Docs' and Nexus Sheet's so the suite's tabs feel like one
//! product. Only the arrangement is borrowed: every icon is a Lucide icon PdfCraft already ships
//! (`icons.rs`) or drawn in code here, the colours are the app's own (`theme.rs`), and the command
//! names come from PdfCraft's command registry.
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

/// What a host window with tabs (NexusOffice) does for the embedded app. Every hook is optional;
/// without one the app behaves as it does stand-alone.
#[derive(Default)]
pub struct Host {
    /// File › New (a blank PDF), Ctrl+N: the host opens a new tab; this document stays.
    pub new_document: Option<Box<dyn Fn()>>,
    /// A PDF picked in Open (or a recent file, or a dropped one) while this app shows a document:
    /// the host opens it in a tab of its own, or brings that tab forward.
    pub open_document: Option<Box<dyn Fn(&str)>>,
    /// A new, unsaved document this app made (from images, the clipboard, Combine, Extract…):
    /// `(name, bytes)`, for a tab of its own. The app closes its copy once handed over.
    pub adopt_document: Option<Box<dyn Fn(&str, Arc<Vec<u8>>)>>,
    /// Open and Save As dialogs answered later (wins over the app's own pickers for File › Open
    /// and Save).
    pub file_dialog: Option<Box<dyn Fn(FileDialogRequest) -> Receiver<FileDialogAnswer>>>,
}

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
                &[big("nx.hand", "Hand"), big("nx.select", "Select"), small("view.marquee_zoom", "Marquee zoom"), small("edit.snapshot", "Snapshot")],
                &[
                    small("nx.zoom_in", "Zoom in"),
                    small("nx.zoom_out", "Zoom out"),
                    small("nx.fit_width", "Fit to width"),
                    small("nx.fit_page", "Fit page"),
                    small("nx.actual_size", "Actual size"),
                    small("nx.rotate_view", "Rotate view"),
                ],
                &[
                    big("edit.edit_text", "Edit text"),
                    big("edit.text", "Add text"),
                    big("comment.highlight", "Highlight"),
                    big("comment.note", "Note"),
                ],
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
                &[big("optimize.reduce", "Reduce size"), small("optimize.advanced", "Optimize"), small("standards.pdfa", "PDF/A")],
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
        let strays: Vec<pdfcraft_engine::DocId> = self.views.iter().map(|v| v.id).filter(|id| *id != own && self.compare_old != Some(*id)).collect();
        for id in strays {
            let Some(doc) = self.session.get(id) else { continue };
            let (name, path, dirty) = (doc.name.clone(), doc.path.clone(), doc.dirty);
            let handed = match (&self.host.open_document, &self.host.adopt_document, path) {
                (Some(open), _, Some(p)) if !dirty => {
                    open(&p);
                    true
                }
                (_, Some(adopt), _) => match self.session.save_bytes(id) {
                    Ok(bytes) => {
                        adopt(&name, bytes);
                        true
                    }
                    Err(e) => {
                        self.notify_fmt("Couldn't save {name}: {e}", &[("name", &name), ("e", &e.to_string())]);
                        false
                    }
                },
                _ => false,
            };
            if handed && let Some(i) = self.views.iter().position(|v| v.id == id) {
                self.close_tab(i);
            }
        }
        // The tab's own document stays the one shown.
        if let Some(i) = self.views.iter().position(|v| v.id == own) {
            self.active = Some(i);
        }
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

    /// NexusOffice: install the fonts and the look again, painting nothing on the next frame. A
    /// host window with other editors calls it when this app's tab comes forward: another editor
    /// may have replaced the window's fonts and visuals meanwhile.
    pub fn restyle(&mut self) {
        self.styled = false;
        self.fonts_ready = false;
    }

    /// NexusOffice: `bytes` as a new, unsaved document of this app (a document another tab made
    /// and handed over): closing it asks to save it.
    pub fn open_unsaved(&mut self, name: &str, bytes: Arc<Vec<u8>>) -> Result<(), String> {
        let id = self.session.open_new(name, bytes).map_err(|e| e.to_string())?;
        self.session.set_reading_colors(id, self.reading_colors);
        let info = &self.session.get(id).ok_or("the new document could not be opened")?.info;
        self.views.push(crate::DocView::new(id, info, self.view_defaults));
        self.active = Some(self.views.len() - 1);
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

/// Above the Combine files page when it replaces the document: the way back to it.
fn back_bar(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let label = tl!("Back to the document");
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
    let label = tl!("File");
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
        let shown = tl!(label);
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
    let labels: Vec<&str> = WpsTab::ALL.iter().map(|tab| tl!(tab.label())).collect();
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
        let tip = tl!("More tabs");
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
    let (text, color) = if saved { (tl!("Saved"), saved_color(t.dark())) } else { (tl!("Not saved"), t.text_muted) };
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
        if ui.button(tl!("Save")).clicked() {
            ui.close();
            app.execute("file.save");
        }
        if ui.button(tl!("Save as…")).clicked() {
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
    let ph = tl!("Find tools and commands");
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
    let tip = tl!("More options");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    let resp = resp.on_hover_text(tip);
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        for (command, label) in [("view.theme.light", "Light gray"), ("view.theme.dark", "Dark gray"), ("view.theme.system", "Use system setting")] {
            if ui.button(tl!(label)).clicked() {
                ui.close();
                app.execute(command);
            }
        }
        ui.separator();
        for (command, label) in [("app.preferences", "Preferences…"), ("help.shortcuts", "Keyboard shortcuts"), ("help.about", "About PdfCraft")] {
            if ui.button(tl!(label)).clicked() {
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
    (text_w(ui, tl!(label), theme::regular(12.0)) + 14.0).max(46.0)
}

fn small_width(ui: &Ui, label: &str) -> f32 {
    6.0 + 16.0 + 6.0 + text_w(ui, tl!(label), theme::regular(12.0)) + 8.0
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
                let tip = tl!("More commands");
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
    let label = tl!(item.label);
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
    let tip = tl!(tip);
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
                if icons::button(ui, icon, 32.0, selected, tl!(tip)).clicked() {
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
                ui.label(egui::RichText::new(tl!("Page")).font(theme::regular(12.0)).color(t.text_muted));
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
                let r = ui.add(edit).on_hover_text(tl!("Current page — type a page number or label (such as iv) and press Enter"));
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let typed = view.page_input.clone();
                        view.go_to_typed(&typed, &labels);
                    }
                    view.page_input = view.current_page_label(labels.get(view.current).map(String::as_str));
                }
                ui.label(egui::RichText::new(format!("/ {count}")).font(theme::regular(12.0)).color(t.text_muted));
                ui.add_space(4.0);
                if icons::button(ui, "chevron-up", 22.0, false, tl!("Previous page")).clicked() {
                    view.step_page(false);
                }
                if icons::button(ui, "chevron-down", 22.0, false, tl!("Next page")).clicked() {
                    view.step_page(true);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if icons::button(ui, "maximize-2", 22.0, view.fit == Fit::Page, tl!("Zoom to page level")).clicked() {
                        view.set_fit(Fit::Page);
                    }
                    if icons::button(ui, "maximize", 22.0, view.fit == Fit::Width, tl!("Fit to width")).clicked() {
                        view.set_fit(Fit::Width);
                    }
                    ui.add_space(6.0);
                    if icons::button(ui, "zoom-in", 22.0, false, tl!("Zoom in")).clicked() {
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
                    if icons::button(ui, "zoom-out", 22.0, false, tl!("Zoom out")).clicked() {
                        view.zoom_step(false);
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    type H = egui_kittest::Harness<'static, PdfCraftApp>;

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
    /// a registry command (or one of the strip's own) with an icon PdfCraft ships.
    #[test]
    fn every_tab_fits_the_strip_at_both_sizes() {
        assert!(CONTENT_H + 6.0 <= STRIP_H, "two rows of small buttons fit the strip");
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
                assert!(strip_fit(&h.ctx).is_some_and(|f| f.shown == f.groups), "Home fits a 1440 pt window");
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
        h.get_by_label("Trang").click();
        h.run_steps(3);
        assert_eq!(h.state().wps_tab, WpsTab::Pages);
        h.get_by_label("Trang").click();
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
            a.host.adopt_document = Some(Box::new(move |name, bytes| adopt_log.borrow_mut().push(format!("adopt {name} {}", bytes.len() > 100))));
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
