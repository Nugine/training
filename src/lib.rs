use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize)]
pub enum Question {
    SingleAns {
        text: String,
        choices: BTreeMap<String, String>,
        ans: String,
    },
    MultiAns {
        text: String,
        choices: BTreeMap<String, String>,
        ans: BTreeSet<String>,
    },
    TrueOrFalse {
        text: String,
        ans: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub current_no: usize,
    pub question_states: BTreeMap<usize, QuestionState>,
    pub completed_count: usize,
    pub complete_threshold: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionState {
    pub is_completed: bool,
    pub try_count: usize,
    pub failed_count: usize,
    pub correct_count: usize,
}
