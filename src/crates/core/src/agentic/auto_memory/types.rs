use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoMemoryState {
    #[serde(default)]
    pub next_unextracted_turn: usize,
    #[serde(default)]
    pub history_revision: u64,
    #[serde(default)]
    pub eligible_turns_since_last_extraction: usize,
    #[serde(default)]
    pub last_memory_consumed_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoMemoryScheduleDecision {
    NotReadyByEligibleTurns,
    CoolingDown { ready_at_ms: i64 },
    ReadyNow,
}

impl AutoMemoryScheduleDecision {
    pub const fn ready_at_ms(self) -> Option<i64> {
        match self {
            Self::CoolingDown { ready_at_ms } => Some(ready_at_ms),
            Self::NotReadyByEligibleTurns | Self::ReadyNow => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoMemoryQueueAction {
    Skip,
    QueueNow,
    QueueAt { ready_at_ms: i64 },
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

    pub fn mark_extracted_through(
        &mut self,
        through_turn: usize,
        turn_count: usize,
        consumed_at_ms: i64,
    ) {
        self.next_unextracted_turn = through_turn.saturating_add(1).min(turn_count);
        self.eligible_turns_since_last_extraction = 0;
        self.last_memory_consumed_at_ms = Some(consumed_at_ms);
    }

    pub fn note_eligible_turn_and_schedule_decision(
        &mut self,
        extract_every_eligible_turns: usize,
        min_extract_interval_secs: u64,
        now_ms: i64,
    ) -> AutoMemoryScheduleDecision {
        let threshold = extract_every_eligible_turns.max(1);
        self.eligible_turns_since_last_extraction =
            self.eligible_turns_since_last_extraction.saturating_add(1);
        self.schedule_decision(threshold, min_extract_interval_secs, now_ms)
    }

    pub fn schedule_decision(
        &self,
        extract_every_eligible_turns: usize,
        min_extract_interval_secs: u64,
        now_ms: i64,
    ) -> AutoMemoryScheduleDecision {
        let threshold = extract_every_eligible_turns.max(1);
        if self.eligible_turns_since_last_extraction < threshold {
            return AutoMemoryScheduleDecision::NotReadyByEligibleTurns;
        }

        let Some(ready_at_ms) = self.next_ready_at_ms(min_extract_interval_secs) else {
            return AutoMemoryScheduleDecision::ReadyNow;
        };

        if now_ms >= ready_at_ms {
            AutoMemoryScheduleDecision::ReadyNow
        } else {
            AutoMemoryScheduleDecision::CoolingDown { ready_at_ms }
        }
    }

    pub fn next_ready_at_ms(&self, min_extract_interval_secs: u64) -> Option<i64> {
        if min_extract_interval_secs == 0 {
            return None;
        }

        let interval_ms = min_extract_interval_secs
            .saturating_mul(1000)
            .min(i64::MAX as u64) as i64;
        self.last_memory_consumed_at_ms
            .map(|last_consumed_at_ms| last_consumed_at_ms.saturating_add(interval_ms))
    }
}

#[derive(Debug, Clone)]
pub struct AutoMemoryExtractionCursor {
    pub from_turn: usize,
    pub through_turn: usize,
    pub history_revision: u64,
}
