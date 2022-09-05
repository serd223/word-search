//! Use release mode

/*
Sources:
turkish_words.txt: https://github.com/mertemin/turkish-word-list
wordle-nyt-allowed-guesses.txt: https://github.com/fredoverflow/wordle
wordle-nyt-answers-alphabetical.txt: https://github.com/fredoverflow/wordle
words_alpha.txt: https://github.com/dwyl/english-words/
*/

use rayon::prelude::*;

mod word;
use word::*;
mod file_read;
use file_read::*;

use std::collections::HashMap;
use std::time::Instant;
use std::sync::{Arc, Mutex};

const WORD_FILE_PATH: &str = "res/turkish_words.txt";
const MAX_LEN_DIFF: i32 = 3;
const SUGGESTION_COUNT: usize = 7;
const LEN_DIFF_OVER_WEIGHT_BIAS: f32 = 0.005;
const MAX_WEIGHT_DIFF: f32 = 0.79;
// Forces the first "len/n as i32" letters of the suggestion to exactly match the input.
// The program will make sure the first letter matches the input even if "len/n as i32" is 0.
const FORCED_LETTER_MULTIPLIER: usize = 4;
const SHOWN_WEIGHT_DIGITS: u32 = 8;
const WEIGHT_FORMAT_MULTIPLIER: f32 = (10u32).pow(SHOWN_WEIGHT_DIGITS - 1) as f32;

const EXIT_COMMAND: &str = "[exit";
const CHANGE_SOURCE_COMMAND: &str = "[csrc";
const SHOW_SOURCE_COMMAND: &str = "[ssrc";
const TOGGLE_TIME_ELAPSED_COMMAND: &str = "[tst";
const TOGGLE_SHOW_WEIGHT_COMMAND: &str = "[tsw";
const HELP_COMMAND: &str = "[help";

fn main() {
    let welcome_text = format!(r#"
    ------- Welcome to... -------
    # # # ---- SERD SEARCH ---- # # #

- Commands -
: {} to exit.
: {} to change search source file. (must be located in the res/ folder)
: {} to show current source file.
: {} to toggle the time elapsed text.
: {} to toggle the visibilty of weights in suggestions.
: {} to show this list again.
    "#, 
        EXIT_COMMAND, CHANGE_SOURCE_COMMAND, SHOW_SOURCE_COMMAND, 
        TOGGLE_TIME_ELAPSED_COMMAND, TOGGLE_SHOW_WEIGHT_COMMAND, HELP_COMMAND
    );
    let mut show_time_elapsed = false;
    let mut show_weight = false;

    let different_char_count = |a: &str, b: &str| -> i32 {
        let n = a.chars().count().min(b.chars().count());
        let mut ac = a.chars();
        let mut bc = b.chars();
        let mut res = 0;
        for _ in 0..n {
            if ac.next() != bc.next() {
                res += 1;
            }
        }
        res
    };
    println!("{}", welcome_text);
    
    let mut current_source = WORD_FILE_PATH.to_string();
    let mut word_list_str_buf = String::new();
    read_to(WORD_FILE_PATH, &mut word_list_str_buf).expect("Couldn't open file");
    word_list_str_buf = word_list_str_buf.to_lowercase();

    loop {
        print!("Enter text to search: ");
        {
            use std::io::{stdout, Write};
            stdout().flush().expect("Couldn't flush stdout.");
        }
        let mut input_string = String::new();
        {
            use std::io::stdin;
            stdin().read_line(&mut input_string).expect("Input not valid UTF-8");
        }
        if let Some('\n') = input_string.chars().next_back() { input_string.pop(); }
        if let Some('\r') = input_string.chars().next_back() { input_string.pop(); }

        //---------------------------------------------//
        if input_string.contains(EXIT_COMMAND) {
            break;
        }
        if input_string.contains(SHOW_SOURCE_COMMAND) {
            println!("{}", current_source);
            continue;
        }
        if input_string.contains(CHANGE_SOURCE_COMMAND) {
            let split_input: Vec<&str> = input_string.split(" ").collect();
            if split_input.len() <= 1 {
                continue;
            }
            let file_path = split_input[1];
            let mut full_path = String::from("res/");
            full_path.push_str(file_path);
            current_source = full_path.clone();
            let mut temp_buf = String::new();

            match read_to(full_path.as_str(), &mut temp_buf) {
                Err(_) => {println!("{} does not exist.", full_path); continue;}
                _ => ()
            };

            word_list_str_buf.clear();
            word_list_str_buf = temp_buf;
            word_list_str_buf = word_list_str_buf.to_lowercase();
            continue;
        }
        if input_string.contains(TOGGLE_TIME_ELAPSED_COMMAND) {
            show_time_elapsed ^= true;
            continue;
        }
        if input_string.contains(TOGGLE_SHOW_WEIGHT_COMMAND) {
            show_weight ^= true;
            continue;
        }
        if input_string.contains(HELP_COMMAND) {
            println!("{}", welcome_text);
            continue;
        }
        //---------------------------------------------//

        input_string = input_string.to_lowercase();
        let input = input_string.as_str();
        let input_len = input.chars().count();
        if input_len < 1 {
            continue;
        }
        let forced_letter_count = match (input_len / FORCED_LETTER_MULTIPLIER) as usize {
            0 => 1,
            n => n
        };
        let t = Instant::now();
        let input_word = Word::from_str(input);

        let mut input_chars = input.chars().clone();
        let mut word_prefix = String::new();
        for _ in 0..forced_letter_count {
            word_prefix.push(input_chars.next().unwrap());
        }
        // { str: total_difference }
        let suggestions: HashMap<&str, f32> = HashMap::new();
        let sm = Arc::new(Mutex::new(suggestions));

        let words_list: Vec<Word> = word_list_str_buf
            .par_lines()
            .filter(|s| {
                let len_diff = (input_len as i32 - s.chars().count() as i32).abs();
                !(
                    len_diff > MAX_LEN_DIFF 
                    || !s.starts_with(&word_prefix) 
                )
            })
            .map(|s| Word::from_str(s))
            .collect::<Vec<Word>>();
        
        if show_time_elapsed {
            println!("Loaded and filtered all words: {:?}", t.elapsed());
        }

        words_list.par_iter().for_each(|w| {
            let len_diff = (input_len as i32 - w.str_repr.chars().count() as i32).abs();
            
            let total: f32 = w.letters
                .iter()
                .map(|(c, w)| {
                    match input_word.letters.get(c) {
                        Some(iw) => (iw - w).abs(),
                        None => *w
                    }
                })
                .sum::<f32>()
                + (len_diff as f32 / input_len as f32) / 100.
                + different_char_count(input, w.str_repr) as f32 / input_len as f32 * 1.35;
            
            if total <= MAX_WEIGHT_DIFF {
                let mut krm = ""; //  Key To Remove
                let mut should_add = false;
                let mut sg = sm.lock().unwrap();
                if sg.len() < SUGGESTION_COUNT { should_add = true }

                if !should_add {
                    for (k, v) in sg.iter() {
                        let slendiff = (input_len as i32 - k.chars().count() as i32).abs();
                        if total < *v {
                            krm = k;
                            should_add = true;
                            break;
                        } else if len_diff < slendiff && (total - v).abs() < LEN_DIFF_OVER_WEIGHT_BIAS {
                            krm = k;
                            should_add = true;
                            break;
                        }
                    }
                }
                if should_add {
                    (*sg).remove(krm);
                    (*sg).insert(w.str_repr, total);
                }
                // drop(sg);
            }
        });
        
        if show_time_elapsed {
            println!("Total time: {:?}", t.elapsed());
        }

        let sg = sm.lock().unwrap();
        let mut sv: Vec<(&str, i32)> = (*sg)
            .iter()
            .map(|(k,v)| (*k, (*v * WEIGHT_FORMAT_MULTIPLIER) as i32))
            .collect();
        drop(sg);
        sv.sort_unstable_by_key(|(_, v)| *v);
        for s in sv {
            print!("- {}", s.0);
            if show_weight {
                print!(" ({})", s.1);
            }
            println!();
        }
    }
}
