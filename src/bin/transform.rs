use std::fs::File;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use training::Question;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Question12 {
    text: String,
    A: String,
    B: String,
    C: String,
    D: String,
    ans: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Question3 {
    text: String,
    ans: bool,
}

fn transform<T: DeserializeOwned>(input: &str) -> Result<Vec<T>> {
    let mut reader = csv::Reader::from_path(input)?;
    let mut data = Vec::new();
    for result in reader.deserialize() {
        data.push(result.with_context(|| input.to_owned())?);
    }
    Ok(data)
}

fn main() -> Result<()> {
    let q1 = transform::<Question12>("data/马原单选.csv")?;
    let q2 = transform::<Question12>("data/马原多选.csv")?;
    let q3 = transform::<Question3>("data/马原判断.csv")?;

    dbg!(q1.len(), q2.len(), q3.len());

    let mut output: Vec<Question> = Vec::new();
    for q in q1 {
        let choices = [("A", q.A), ("B", q.B), ("C", q.C), ("D", q.D)]
            .iter()
            .map(|&(k, ref v)| (k.to_owned(), v.to_owned()))
            .collect();

        output.push(Question::SingleAns {
            text: q.text,
            choices,
            ans: q.ans,
        });
    }

    for q in q2 {
        let choices = [("A", q.A), ("B", q.B), ("C", q.C), ("D", q.D)]
            .iter()
            .map(|&(k, ref v)| (k.to_owned(), v.to_owned()))
            .collect();

        output.push(Question::MultiAns {
            text: q.text,
            choices,
            ans: q.ans.split("").map(ToOwned::to_owned).collect(),
        });
    }

    for q in q3 {
        output.push(Question::TrueOrFalse {
            text: q.text,
            ans: q.ans,
        })
    }

    let file = File::create("data/questions.json")?;
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(file, fmt);
    Serialize::serialize(&output, &mut ser)?;

    Ok(())
}
