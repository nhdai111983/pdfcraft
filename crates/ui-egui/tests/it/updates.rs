//! Help ▸ Check for updates (issue #28). NexusOffices: Nexus PDF never checks for, downloads or
//! installs a release itself (upstream's `craft_update` check would ask `storytold/pdfcraft`); new
//! versions arrive through the NexusOffices installer, and the dialog says so, in English and
//! Vietnamese. Stand-in release sources count calls, so no test reaches the network.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;

fn release(tag: &str) -> craft_update::Release {
    craft_update::Release {
        tag: tag.to_string(),
        version: craft_update::parse_version(tag),
        html_url: format!("https://github.com/nhdai111983/pdfcraft/releases/tag/{tag}"),
        ..Default::default()
    }
}

/// An app whose release source offers v99.0.0 and counts how often it is asked.
fn harness(language: &'static str, startup_check: bool) -> (Harness<'static, PdfCraftApp>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = calls.clone();
    let h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", language).unwrap();
        let counted = counted.clone();
        app.set_update_source(Arc::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            Ok(Some(release("v99.0.0")))
        }));
        app.set_startup_update_check(startup_check);
        app
    });
    (h, calls)
}

/// A few frames, with time for a background check to report if one had started.
fn settle(h: &mut Harness<'static, PdfCraftApp>) {
    for _ in 0..5 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn check_for_updates_points_at_the_nexusoffices_installer() {
    let (mut h, calls) = harness("en", false);
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    h.get_by_label_contains("is updated through the NexusOffices installer");
    assert!(h.query_by_label_contains("Checking for a newer version").is_none());
    assert!(h.query_by_label_contains("is available").is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0, "no release source is asked");
    // Escape closes the dialog (the home screen's Nexus card has a "Close" button of its own).
    h.key_press(egui::Key::Escape);
    settle(&mut h);
    assert!(h.query_by_label_contains("NexusOffices installer").is_none(), "Close closes the dialog");
}

#[test]
fn check_for_updates_speaks_vietnamese() {
    let (mut h, calls) = harness("vi", false);
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    h.get_by_label_contains("được cập nhật qua trình cài đặt NexusOffices");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn nothing_is_asked_at_start_even_when_turned_on() {
    // Upstream's desktop app turns the check at start on; the Nexus build does not, and even
    // turned on it asks nothing and offers no upstream (or any) release.
    let (mut h, calls) = harness("en", true);
    settle(&mut h);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(h.query_by_label_contains("is available").is_none());
    assert!(h.query_by_label("UPDATE AND RESTART").is_none() && h.query_by_label("DOWNLOAD").is_none());
}
