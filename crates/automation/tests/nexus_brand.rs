//! NexusOffices: what Nexus PDF writes into people's files and tells other programs names Nexus
//! PDF, not upstream (spec §4.4): the Producer of a new PDF, the default author of comments, the
//! signature's build dictionary, MCP `serverInfo`, instructions and tool texts, and command labels.
//! Every check saves through the engine (the automation tools) and reads the file back.

use std::path::{Path, PathBuf};

#[cfg(feature = "mcp")]
use pdfcraft_automation::mcp::McpServer;
use pdfcraft_automation::{Automation, Content, tools};
use serde_json::{Value, json};

/// A one-page PDF.
fn fixture() -> Vec<u8> {
    let objs: [&[u8]; 3] = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 200 300] >>",
        b"<< /Type /Page /Parent 2 0 R >>",
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

/// A fresh directory with `a.pdf`, removed when dropped.
struct Dir(PathBuf);
impl Dir {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("nexus-pdf-brand-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.pdf"), fixture()).unwrap();
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ok(a: &mut Automation, tool: &str, args: Value) -> Value {
    match a.call(tool, &args) {
        Ok(mut c) => match c.remove(0) {
            Content::Json(v) => v,
            other => panic!("{tool}: expected JSON, got {other:?}"),
        },
        Err(e) => panic!("{tool} {args}: {e}"),
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn auto(dir: &Dir) -> Automation {
    Automation::new().with_root(dir.path()).unwrap().with_clock(|| 1_700_000_000)
}

/// A new PDF's Producer is Nexus PDF.
#[test]
fn new_documents_name_nexus_pdf_as_producer() {
    let dir = Dir::new("producer");
    let mut a = auto(&dir);
    let doc = ok(&mut a, "doc_create", json!({ "from": "text", "text": "Xin chao" }))["doc"].as_u64().unwrap();
    ok(&mut a, "doc_save", json!({ "doc": doc, "path": "new.pdf" }));
    let bytes = std::fs::read(dir.path().join("new.pdf")).unwrap();
    assert!(!contains(&bytes, b"PdfCraft"), "no upstream name in the file");
    let mut b = auto(&dir);
    let re = ok(&mut b, "doc_open", json!({ "path": "new.pdf" }))["doc"].as_u64().unwrap();
    assert_eq!(ok(&mut b, "doc_info", json!({ "doc": re }))["producer"], "Nexus PDF");
}

/// A comment added without an author is signed "Nexus PDF User" in the saved file.
#[test]
fn default_author_is_nexus() {
    let dir = Dir::new("author");
    let mut a = auto(&dir);
    let doc = ok(&mut a, "doc_open", json!({ "path": "a.pdf" }))["doc"].as_u64().unwrap();
    ok(&mut a, "comment_add", json!({ "doc": doc, "page": 1, "type": "note", "at": [40, 40], "contents": "hello" }));
    ok(&mut a, "comment_add", json!({ "doc": doc, "page": 1, "type": "rectangle", "rect": [20, 60, 120, 120] }));
    ok(&mut a, "doc_save", json!({ "doc": doc, "path": "commented.pdf" }));
    let bytes = std::fs::read(dir.path().join("commented.pdf")).unwrap();
    assert!(!contains(&bytes, b"PdfCraft"), "no upstream name in the file");
    let mut b = auto(&dir);
    let re = ok(&mut b, "doc_open", json!({ "path": "commented.pdf" }))["doc"].as_u64().unwrap();
    let list = ok(&mut b, "comment_list", json!({ "doc": re }));
    let comments = list["comments"].as_array().unwrap();
    assert_eq!(comments.len(), 2, "{list}");
    for c in comments {
        assert_eq!(c["author"], "Nexus PDF User", "{c}");
    }
}

/// A signature records Nexus PDF as the signing application (`/Prop_Build /App /Name`).
#[test]
fn signatures_name_nexus_pdf_as_the_application() {
    let dir = Dir::new("sign");
    let mut a = auto(&dir);
    ok(&mut a, "sign_id_create", json!({ "name": "Ada Lovelace", "key": "p256", "password": "secret12", "path": "ada.p12" }));
    let doc = ok(&mut a, "doc_open", json!({ "path": "a.pdf" }))["doc"].as_u64().unwrap();
    ok(
        &mut a,
        "sign_document",
        json!({ "doc": doc, "id": "ada.p12", "password": "secret12", "page": 1, "rect": [20, 200, 180, 250], "out": "signed.pdf" }),
    );
    let signed = std::fs::read(dir.path().join("signed.pdf")).unwrap();
    assert!(contains(&signed, b"/Prop_Build"), "a build dictionary");
    assert!(contains(&signed, b"/Name /Nexus#20PDF"), "the application is Nexus PDF");
    assert!(!contains(&signed, b"PdfCraft"), "no upstream name in the signed file");
}

/// `command_list` (MCP, `run`) shows registry labels with the Nexus name and no ArtCraft links.
#[test]
fn command_labels_name_nexus_pdf_and_offer_no_artcraft_links() {
    let mut a = Automation::new();
    let list = ok(&mut a, "command_list", json!({}));
    let commands = list["commands"].as_array().unwrap();
    let about = commands.iter().find(|c| c["id"] == "help.about").unwrap();
    assert_eq!(about["label"], "About Nexus PDF");
    for c in commands {
        let label = c["label"].as_str().unwrap_or_default();
        assert!(!label.contains("PdfCraft") && !label.contains("ArtCraft"), "{c}");
        assert!(!["help.discord", "help.website", "help.app_page"].contains(&c["id"].as_str().unwrap_or_default()), "{c}");
    }
}

/// No tool's title or description names upstream or its command names.
#[test]
fn tool_texts_name_the_nexus_binaries() {
    for t in tools() {
        for text in [t.title, t.description] {
            assert!(!text.contains("PdfCraft") && !text.contains("ArtCraft") && !text.contains("pdfcraft-cli"), "{}: {text}", t.name);
        }
    }
}

/// MCP: `serverInfo.title`, the instructions and the meta tools say Nexus PDF; `serverInfo.name`
/// stays `pdfcraft` (the machine identifier clients are configured by).
#[cfg(feature = "mcp")]
#[test]
fn server_info_and_instructions_name_nexus_pdf() {
    for compact in [false, true] {
        let mut s = McpServer::new(Automation::new()).with_compact(compact);
        let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-03-26" } });
        let reply: Value = serde_json::from_str(&s.handle_line(&init.to_string()).unwrap()).unwrap();
        let result = &reply["result"];
        assert_eq!(result["serverInfo"]["name"], "pdfcraft");
        assert_eq!(result["serverInfo"]["title"], "Nexus PDF");
        assert_eq!(result["serverInfo"]["websiteUrl"], "https://github.com/nhdai111983/pdfcraft");
        let instructions = result["instructions"].as_str().unwrap();
        assert!(instructions.starts_with("Nexus PDF edits PDFs."), "{instructions}");
        let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
        let tools = s.handle_line(&list.to_string()).unwrap();
        assert!(!tools.contains("PdfCraft") && !tools.contains("ArtCraft"), "compact {compact}: tools/list names upstream");
    }
}
