use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize)]
enum Question {
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

fn main() -> Result<()> {
    Ok(())
}
