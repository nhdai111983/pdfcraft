//! NexusOffice: the command line names the installed program, `nexus-pdf-cli` (packages install
//! the Cargo binary `pdfcraft-cli` under that name), and points at the fork, not at ArtCraft.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_pdfcraft-cli");

fn run(args: &[&str]) -> (String, String) {
    let out = Command::new(BIN).args(args).output().unwrap();
    (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

#[test]
fn usage_and_version_name_nexus_pdf_cli() {
    let (version, _) = run(&["--version"]);
    assert!(version.starts_with(&format!("nexus-pdf-cli {}", env!("CARGO_PKG_VERSION"))), "{version}");
    assert!(version.contains("https://github.com/sonnytech-vn/pdfcraft"), "{version}");
    let (usage, _) = run(&["--help"]);
    assert!(usage.contains("nexus-pdf-cli render <file.pdf> --page N"), "{usage}");
    assert!(usage.contains("https://github.com/sonnytech-vn/pdfcraft"), "{usage}");
    for text in [&version, &usage] {
        assert!(!text.contains("pdfcraft-cli") && !text.contains("discord") && !text.contains("artcraft"), "{text}");
    }
    let (_, unknown) = run(&["frobnicate"]);
    assert!(unknown.starts_with("nexus-pdf-cli: unknown command 'frobnicate'; see nexus-pdf-cli --help"), "{unknown}");
}

#[test]
fn the_app_hint_names_nexus_pdf() {
    let (_, err) = run(&["ui"]);
    assert!(err.contains("start the app with `nexus-pdf --control FILE`"), "{err}");
    assert!(!err.contains("pdfcraft"), "{err}");
}

/// Errors from the engine reach the command line in English: they name no upstream product.
#[test]
fn errors_from_a_junk_file_name_no_upstream_product() {
    let dir = std::env::temp_dir().join(format!("nexus-pdf-cli-junk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let junk = dir.join("junk.pdf");
    std::fs::write(&junk, b"this is not a PDF at all").unwrap();
    let junk = junk.to_string_lossy().into_owned();
    for args in [vec!["info", junk.as_str()], vec!["text", junk.as_str()], vec!["render", junk.as_str(), "--page", "1", "--out", "x.png"]] {
        let (out, err) = run(&args);
        assert!(!err.is_empty() || !out.is_empty(), "{args:?} says something");
        for text in [&out, &err] {
            assert!(!text.contains("PdfCraft") && !text.contains("pdfcraft-cli"), "{args:?}: {text}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
