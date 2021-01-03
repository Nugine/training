use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::File;
use std::io::stdout;

use anyhow::{Context, Result};
use crossterm::cursor;
use crossterm::terminal::{self, ClearType};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use training::{Question, QuestionState, State};

static QUESTIONS: Lazy<Vec<Question>> = Lazy::new(|| {
    const QUESTIONS_JSON: &str = include_str!("../data/questions.json");
    serde_json::from_str(QUESTIONS_JSON).unwrap()
});

const STATE_PATH: &str = "state.training.bin";

fn init_state() -> State {
    let mut qss: Vec<QuestionState> = QUESTIONS
        .iter()
        .enumerate()
        .map(|(i, _)| QuestionState {
            no: i + 1,
            try_count: 0,
            failed_count: 0,
            correct_count: 0,
        })
        .collect();

    qss.shuffle(&mut rand::thread_rng());

    State {
        recent_incorrect: Vec::new(),
        recent_correct: Vec::new(),
        total_questions: qss.len(),
        questions_states: qss,
        complete_threshold: 3,
    }
}

fn load_state(path: &str) -> Result<State> {
    match File::open(path) {
        Ok(file) => {
            let state = bincode::deserialize_from(file)?;
            Ok(state)
        }
        Err(_) => {
            let state = init_state();
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

fn read_line(state: &State, ans_str: &str, allow_empty: bool) -> Result<String> {
    loop {
        let line: String = text_io::read!("{}\n");
        let line = line.trim().to_owned();
        match line.as_str() {
            "" if allow_empty => continue,
            "h" | "H" => {
                println!("提示：{}", ans_str);
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
}

fn main() -> Result<()> {
    let state_path = env::current_dir()?.join(&STATE_PATH);
    let mut state = load_state(STATE_PATH)
        .with_context(|| format!("加载状态失败：状态文件位置：{}", state_path.display()))?;

    let questions = &**QUESTIONS;

    loop {
        if state.questions_states.is_empty() {
            println!("全部完成！");
            break;
        }

        let qs_idx = {
            let select_recent_incorrect =
                !state.recent_incorrect.is_empty() && rand::random::<f64>() > 0.618;

            if select_recent_incorrect {
                let no = state.recent_incorrect[0];
                state
                    .questions_states
                    .iter()
                    .position(|qs| qs.no == no)
                    .unwrap()
            } else {
                let select_recent_correct =
                    !state.recent_correct.is_empty() && rand::random::<f64>() > 0.618;

                if select_recent_correct {
                    let no = state.recent_correct[0];
                    state
                        .questions_states
                        .iter()
                        .position(|qs| qs.no == no)
                        .unwrap()
                } else {
                    0
                }
            }
        };

        let qs = &state.questions_states[qs_idx];
        println!(
            "训练进度 {:>4}/{:>4}",
            state.total_questions - state.questions_states.len(),
            state.total_questions
        );

        println!(
            "题目编号 {:>4}/{:>4}，尝试次数 {:>2}，错误次数 {:>2}, 连续正确次数 {:>2}",
            qs.no,
            questions.len(),
            qs.try_count,
            qs.failed_count,
            qs.correct_count
        );

        println!();

        let (is_correct, ans_str) = match &questions[qs.no - 1] {
            Question::SingleAns { text, choices, ans } => {
                println!("[单选] {}", text);
                let shuffled_choices = {
                    let mut v = choices.keys().collect::<Vec<_>>();
                    v.shuffle(&mut rand::thread_rng());
                    v.into_iter()
                        .zip(choices.values())
                        .collect::<BTreeMap<_, _>>()
                };
                for (label, content) in &shuffled_choices {
                    println!("       {}. {}", label, content);
                }
                let shuffled_ans = shuffled_choices
                    .keys()
                    .copied()
                    .find(|&k| shuffled_choices[k] == &choices[ans])
                    .unwrap();
                let line = read_line(&state, shuffled_ans, true)?;
                (
                    line.eq_ignore_ascii_case(shuffled_ans),
                    shuffled_ans.to_owned(),
                )
            }
            Question::MultiAns { text, choices, ans } => {
                println!("[多选] {}", text);

                let shuffled_choices = {
                    let mut v = choices.keys().collect::<Vec<_>>();
                    v.shuffle(&mut rand::thread_rng());
                    v.into_iter()
                        .zip(choices.values())
                        .collect::<BTreeMap<_, _>>()
                };
                for (label, content) in &shuffled_choices {
                    println!("       {}. {}", label, content);
                }
                let shuffled_ans = shuffled_choices
                    .keys()
                    .filter(|&&k| ans.iter().any(|a| &choices[a] == shuffled_choices[k]))
                    .map(|&k| k.to_owned())
                    .collect::<BTreeSet<_>>();
                for (label, content) in choices {
                    println!("       {}. {}", label, content);
                }
                let ans_str = format!("{:?}", shuffled_ans);
                let line = read_line(&state, &ans_str, true)?;
                let user_ans = line
                    .split("")
                    .filter(|w| !w.is_empty())
                    .map(|w| w.to_ascii_uppercase())
                    .collect::<BTreeSet<_>>();
                (user_ans == shuffled_ans, ans_str)
            }
            Question::TrueOrFalse { text, ans } => {
                println!("[判断] {}", text);
                let ans_str = if *ans { "对" } else { "错" };
                loop {
                    let line = read_line(&state, ans_str, true)?;
                    match line.as_str() {
                        "t" | "T" => break (*ans, ans_str.to_owned()),
                        "f" | "F" => break (!*ans, ans_str.to_owned()),
                        _ => println!("无效输入"),
                    }
                }
            }
        };

        {
            let mut qs = &mut state.questions_states[qs_idx];
            qs.try_count += 1;
            if is_correct {
                qs.correct_count += 1;
                if state
                    .recent_correct
                    .iter()
                    .find(|&&no| no == qs.no)
                    .is_none()
                {
                    state.recent_correct.push(qs.no);
                }
                if qs.correct_count >= state.complete_threshold {
                    if let Some(idx) = state.recent_incorrect.iter().position(|&no| no == qs.no) {
                        state.recent_incorrect.remove(idx);
                    }
                    if let Some(idx) = state.recent_correct.iter().position(|&no| no == qs.no) {
                        state.recent_correct.remove(idx);
                    }
                    state.questions_states.remove(qs_idx);
                }
                state.questions_states.shuffle(&mut rand::thread_rng());
                println!("正确！答案：{}", ans_str);
            } else {
                qs.correct_count = 0;
                qs.failed_count += 1;
                if state
                    .recent_incorrect
                    .iter()
                    .find(|&&no| no == qs.no)
                    .is_none()
                {
                    state.recent_incorrect.push(qs.no);
                }
                println!("错误！答案：{}", ans_str);
            }
        }

        {
            read_line(&state, &ans_str, false)?;
            crossterm::execute!(
                stdout(),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
        }
    }

    Ok(())
}
