//! NexusOffice: one user action the app carries out as two engine edits (Load installed font,
//! then Edit text; `pdfcraft-ui-egui`'s `nexus_installed_font`) is one undo step, and leaves
//! nothing behind when its second edit fails.

use super::{DocId, Scope, Session};

impl Session {
    /// Join the last undo step with the one under it, as one step labelled as the last, when the
    /// one under it is labelled `under`. Undo then goes back to before both; Redo redoes both.
    /// Returns whether it joined them.
    pub fn nexus_join_last_steps(&mut self, id: DocId, under: &str) -> bool {
        let Ok(doc) = self.doc_mut(id) else { return false };
        let Some(editor) = doc.editor.as_mut() else { return false };
        let below = editor.undo.len().checked_sub(2).and_then(|i| editor.undo.get(i));
        if below.is_none_or(|s| s.0.wording != under) {
            return false;
        }
        let Some((label, _, _, scope)) = editor.undo.pop() else { return false };
        let Some(below) = editor.undo.last_mut() else { return false };
        below.0 = label;
        // Undo refreshes what both changed.
        if below.3 != scope {
            below.3 = Scope::Full;
        }
        true
    }

    /// Take back the last step, labelled `wording`, without keeping it for Redo: a step the user
    /// never asked for on its own, whose edit then failed. Returns whether it did.
    pub fn nexus_discard_last_step(&mut self, id: DocId, wording: &str) -> bool {
        let top = self.doc_mut(id).ok().and_then(|d| d.editor.as_ref()).and_then(|e| e.undo.last()).map(|s| s.0.wording);
        if top != Some(wording) || self.undo(id).is_err() {
            return false;
        }
        if let Ok(doc) = self.doc_mut(id)
            && let Some(editor) = doc.editor.as_mut()
        {
            editor.redo.pop();
        }
        true
    }
}
