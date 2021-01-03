use anyhow::Result;
use once_cell::sync::Lazy;
use training::Question;

static QUESTIONS: Lazy<Vec<Question>> = Lazy::new(|| {
    const QUESTIONS_JSON: &str = include_str!("../data/questions.json");
    serde_json::from_str(QUESTIONS_JSON).unwrap()
});

fn main() -> Result<()> {
    dbg!(QUESTIONS.len());
    Ok(())
}
