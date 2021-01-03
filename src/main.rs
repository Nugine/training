use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::stdout;

use anyhow::Result;
use crossterm::cursor;
use crossterm::terminal::{self, ClearType};
use once_cell::sync::Lazy;
use training::{Question, QuestionState, State};

static QUESTIONS: Lazy<Vec<Question>> = Lazy::new(|| {
    const QUESTIONS_JSON: &str = include_str!("../data/questions.json");
    serde_json::from_str(QUESTIONS_JSON).unwrap()
});

fn load_state(path: &str) -> Result<State> {
    match File::open(path) {
        Ok(file) => {
            let state = bincode::deserialize_from(file)?;
            Ok(state)
        }
        Err(_) => {
            let state = State {
                current_no: 1,
                question_states: BTreeMap::new(),
                completed_count: 0,
                complete_threshold: 3,
            };
            save_state(path, &state)?;
            Ok(state)
        }
    }
}

fn save_state(path: &str, state: &State) -> Result<()> {
    let file = File::create(&path)?;
    bincode::serialize_into(&file, &state)?;
    Ok(())
}

fn main() -> Result<()> {
    const STATE_PATH: &str = "state.training.bin";

    let mut state = load_state(STATE_PATH)?;
    let questions = &**QUESTIONS;

    loop {
        {
            let qs = state
                .question_states
                .entry(state.current_no)
                .or_insert_with(|| QuestionState {
                    is_completed: false,
                    try_count: 0,
                    failed_count: 0,
                    correct_count: 0,
                });

            if qs.is_completed {
                state.current_no = (state.current_no - 1 + 1) % questions.len() + 1;
                continue;
            }

            println!(
                "训练进度 {:>4}/{:>4}",
                state.completed_count,
                questions.len()
            );

            println!(
                "题目编号 {:>4}/{:>4}，尝试次数 {:>2}，错误次数 {:>2}, 连续正确次数 {:>2}",
                state.current_no,
                questions.len(),
                qs.try_count,
                qs.failed_count,
                qs.correct_count
            );

            println!();
        }

        let q = &questions[state.current_no - 1];
        let is_correct: bool;
        let read_line = |state: &State, ans: &str, allow_empty: bool| -> Result<String> {
            loop {
                let line: String = text_io::read!("{}\n");
                let line = line.trim().to_owned();
                match line.as_str() {
                    "" if allow_empty => continue,
                    "h" | "H" => {
                        println!("提示：{}", ans);
                        continue;
                    }
                    "s" | "S" => {
                        save_state(&STATE_PATH, state)?;
                        println!("保存成功");
                        continue;
                    }
                    _ => break Ok(line),
                }
            }
        };
        let ans = match q {
            Question::SingleAns { text, choices, ans } => {
                println!("[单选] {}", text);
                for (label, content) in choices {
                    println!("       {}. {}", label, content);
                }
                let line = read_line(&state, ans, true)?;
                is_correct = line.eq_ignore_ascii_case(ans);
                ans.to_owned()
            }
            Question::MultiAns { text, choices, ans } => {
                println!("[多选] {}", text);
                for (label, content) in choices {
                    println!("       {}. {}", label, content);
                }
                let ans_str = format!("{:?}", ans);
                let line = read_line(&state, &ans_str, true)?;
                let user_ans = line
                    .split("")
                    .map(ToOwned::to_owned)
                    .collect::<BTreeSet<_>>();
                is_correct = user_ans == *ans;
                ans_str
            }
            Question::TrueOrFalse { text, ans } => {
                println!("[判断] {}", text);
                let ans_str = if *ans { "对" } else { "错" };
                let line = read_line(&state, ans_str, true)?;
                match line.as_str() {
                    "t" | "T" => is_correct = *ans,
                    "f" | "F" => is_correct = !*ans,
                    _ => panic!("无效输入"),
                }
                ans_str.to_owned()
            }
        };
        {
            let qs = state.question_states.get_mut(&state.current_no).unwrap();

            qs.try_count += 1;
            if is_correct {
                qs.correct_count += 1;
                if qs.correct_count >= state.complete_threshold {
                    qs.is_completed = true;
                    state.completed_count += 1;
                }
                println!("正确！答案：{}", ans);
            } else {
                qs.correct_count = 0;
                qs.failed_count += 1;
                println!("错误！答案：{}", ans);
            }
        }

        {
            read_line(&state, &ans, false)?;
            crossterm::execute!(
                stdout(),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
        }

        state.current_no = (state.current_no - 1 + 1) % questions.len() + 1;
        if state.completed_count >= questions.len() {
            println!("全部完成！");
            break;
        }
    }

    Ok(())
}
