mod brain;
mod database;
mod detector;
mod scrap;

use dotenv::dotenv;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::brain::Brain;
use crate::detector::Detector;
use crate::scrap::start_scrapping;
use std::env;

fn get_argument_by_name(arguments: &Vec<String>, name: &String) -> Option<String> {
    let mut i = 0;
    while i + 1 < arguments.len() {
        if arguments[i] == format!("--{}", name) {
            return Some(arguments[i + 1].clone());
        }
        i+=1;
    }
    return None;
}

fn main() {
    dotenv().ok();

    let mut detector = Detector::new();

    let mut rl = DefaultEditor::new().unwrap();
    let _hst = rl.load_history("history.txt");

    let args: Vec<String> = env::args().collect();
    let action = get_argument_by_name(&args, &"action".to_string());
    if let None = action {
        println!("You did not specify an action. Please do so using --action train|detect");
        return;
    }
    let action = action.unwrap();
    if action != "train".to_string() && action != "detect".to_string() {
        println!("Please specify a valid action : train or detect");
        return;
    }

    if action == "detect".to_string() {
        loop {
            match rl.readline("Enter sentence here: \x1b[33m") {
                Ok(line) => {
                    print!("\x1b[0m");
                    let _ = rl.add_history_entry(line.as_str());
                    let _ = rl.save_history("history.txt");

                    let result = detector.detect(line);
                    if let None = result {
                        println!("The brain has not been trained yet. Please do so using \x1b[93m--action train --repeats 500\x1b[0m (or any other positive integer)");
                        break;
                    }
                    let result = result.unwrap();

                    println!("Your sentence has been detected as \x1b[31m{}\x1b[0m", result);
                },
                Err(ReadlineError::Interrupted) => {
                    println!("\x1b[0mYou canceled your action. Now exiting the program.");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    println!("\x1b[0mEnd of file");
                },
                Err(err) => {
                    println!("\x1b[0mError : {:?}", err);
                }
            }
        }
    }
    if action == "train".to_string() {
        let repeats = get_argument_by_name(&args, &"repeats".to_string());
        if let None = repeats {
            println!("You need to specify an amount of repeats for the train, using \x1b[92m--repeats 100\x1b[0m");
            return;
        }
        let repeats = repeats.unwrap().parse::<u64>().unwrap_or(0);
        if repeats == 0 {
            println!("You need to specify an amount of repeats greater than 0");
            return;
        }

        start_scrapping(&mut detector, &repeats);
    }
}
