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
    pub recent_incorrect: Vec<usize>,
    pub recent_correct: Vec<usize>,
    pub questions_states: Vec<QuestionState>,
    pub complete_threshold: usize,
    pub total_questions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionState {
    pub no: usize,
    pub try_count: usize,
    pub failed_count: usize,
    pub correct_count: usize,
}
