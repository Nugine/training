use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use serde::Serialize;
use training::Question;

fn main() -> Result<()> {
    let mut questions: Vec<Question> = Vec::new();

    let mut lines = BufReader::new(File::open("data/近代史.txt")?)
        .lines()
        .peekable();
    let mut lineno = 0;

    while let Some(line) = lines.next() {
        let line = line?.trim().to_owned();
        lineno += 1;
        if line.is_empty() {
            continue;
        }
        println!("position: data/近代史.txt:{}", lineno);
        dbg!(lineno, questions.len());

        let text = line;

        let ans = lines.next().unwrap()?;
        let a = lines.next().unwrap()?.trim().to_owned();
        let b = lines.next().unwrap()?.trim().to_owned();
        let c = lines.next().unwrap()?.trim().to_owned();
        let d = lines.next().unwrap()?.trim().to_owned();
        dbg!(&a, &b, &c, &d);
        assert!(a.starts_with("A."));
        assert!(b.starts_with("B."));
        assert!(c.starts_with("C."));
        assert!(d.starts_with("D."));

        let mut choices = vec![
            ("A", a[2..].to_owned()),
            ("B", b[2..].to_owned()),
            ("C", c[2..].to_owned()),
            ("D", d[2..].to_owned()),
        ];

        if let Some(Ok(e)) = lines.peek() {
            if !e.is_empty() {
                assert!(e.starts_with("E."));
                choices.push(("E", e.trim()[2..].to_owned()));
                let _ = lines.next();
            }
        }

        lineno += 5;

        let choices = choices
            .into_iter()
            .map(|(l, s)| (l.to_owned(), s))
            .collect();

        let q = if ans.len() == 1 {
            Question::SingleAns { text, choices, ans }
        } else {
            let ans = ans
                .split("")
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            Question::MultiAns { text, choices, ans }
        };
        dbg!(&q);
        questions.push(q);
    }

    let file = File::create("data/近代史.json")?;
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(file, fmt);
    Serialize::serialize(&questions, &mut ser)?;

    Ok(())
}
