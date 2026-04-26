use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoMemoryState {
    #[serde(default)]
    pub next_unextracted_turn: usize,
    #[serde(default)]
    pub history_revision: u64,
    #[serde(default)]
    pub eligible_turns_since_last_extraction: usize,
}

impl AutoMemoryState {
    pub fn normalize_for_turn_count(&mut self, turn_count: usize) {
        self.next_unextracted_turn = self.next_unextracted_turn.min(turn_count);
    }

    pub fn has_pending_turns(&self, turn_count: usize) -> bool {
        self.next_unextracted_turn < turn_count
    }

    pub fn note_history_rollback(&mut self, target_turn: usize) {
        self.next_unextracted_turn = self.next_unextracted_turn.min(target_turn);
        self.history_revision = self.history_revision.saturating_add(1);
        self.eligible_turns_since_last_extraction = 0;
    }

    pub fn mark_extracted_through(&mut self, through_turn: usize, turn_count: usize) {
        self.next_unextracted_turn = through_turn.saturating_add(1).min(turn_count);
        self.eligible_turns_since_last_extraction = 0;
    }

    pub fn note_eligible_turn_and_check_threshold(
        &mut self,
        extract_every_eligible_turns: usize,
    ) -> bool {
        let threshold = extract_every_eligible_turns.max(1);
        self.eligible_turns_since_last_extraction =
            self.eligible_turns_since_last_extraction.saturating_add(1);
        self.eligible_turns_since_last_extraction >= threshold
    }
}

#[derive(Debug, Clone)]
pub struct AutoMemoryExtractionCursor {
    pub from_turn: usize,
    pub through_turn: usize,
    pub history_revision: u64,
}
