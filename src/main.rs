mod brain;
mod database;
mod detector;
mod scrap;

use dotenv::dotenv;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::brain::Brain;
use crate::detector::Detector;
use crate::scrap::{start_scrapping};
use rand::{Rng,rng};
use std::env;
use std::fs;
use std::process::Command;

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
    println!("Welcome to \x1b[94mGreensky's langs detector\x1b[0m\n  Use the executable with an action : detect, train, or evolve, with : --action [action]\n  You can also load a specific model, using : \x1b[90m--brain-template path/to/the/file.json\x1b[0m");
    dotenv().ok();

    let mut detector = Detector::new();

    let mut rl = DefaultEditor::new().unwrap();
    let _hst = rl.load_history("history.txt");

    let args: Vec<String> = env::args().collect();
    let action = get_argument_by_name(&args, &"action".to_string());
    if let None = action {
        println!("You did not specify an action. Please do so using --action train|detect|evolve");
        return;
    }
    let action = action.unwrap();
    let valids = vec!["train".to_string(), "detect".to_string(), "evolve".to_string()];
    let selected = valids.iter().find(|&s| *s == action);
    if let None = selected {
        println!("Please specify a valid action : train, evolve or detect");
        return;
    }
    let selected = selected.unwrap();

    let brain_path = get_argument_by_name(&args, &"brain-template".to_string());
    if let Some(path) = brain_path {
       if !detector.load_brain(&path) {
            println!("\x1b[31mINFO\x1b[0m Tried to load from \x1b[90m{}\x1b[0m but failed. Using default brain", path);
       }
    }

    if *selected == "detect".to_string() {
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
    if *selected == "train".to_string() {
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
    if *selected == "evolve".to_string() {
        let repeats = get_argument_by_name(&args, &"repeats".to_string());
        if let None = repeats {
            println!("You need to specify an amount of repeats for the test for the evolving, using \x1b[92m--repeats 100\x1b[0m for example");
            return;
        }
        let repeats = repeats.unwrap().parse::<u64>().unwrap_or(0);
        if repeats == 0 {
            println!("Specify a positive amount of repeats");
            return;
        }

        let brains = get_argument_by_name(&args, &"brains".to_string());
        if let None = brains {
            println!("You need to specify an amount of brains to have in the list with \x1b[92m--brains 50\x1b[0m");
            return;
        }
        let brains = brains.unwrap().parse::<u64>().unwrap_or(0);
        if repeats == 0 {
            println!("Specify a positive amount of brains");
            return;
        }

        fn create_random_brain(i: &u64) {
            let brain = Brain::random(0.0, 20.0);
            brain.write(&format!("brains/{}.json", *i));
        }

        let mut rng_thread = rng();

        loop {
            if !fs::metadata("brains".to_string()).is_ok() {
                println!("Folder \x1b[90mbrains\x1b[0m does not exist... creating");
                match fs::create_dir("brains".to_string()) {
                    Err(e) => {
                        println!("An error occured : {:?}\nAborting.", e);
                        break;
                    },
                    Ok(_) => {}
                }

                println!("Creating \x1b[91m{}\x1b[0m random brains...", brains);

                let mut i = 0;
                while i < brains {
                    create_random_brain(&i);
                    i+=1;
                }
            }
            if fs::metadata("brains".to_string()).unwrap().is_file() {
                println!("Path \x1b[90mbrains\x1b[0m is a file. Aborting");
                break;
            }

            let mut brain_list: Vec<Brain> = vec![];
            println!("Loading \x1b[91m{}\x1b[0m brains...", brains);

            let mut i = 0;
            while i < brains {
                if !fs::metadata(format!("brains/{}.json", i).to_string()).is_ok() {
                    println!("Cannot find brain \x1b[31mnumber {}\x1b[0m. Creating...", i);
                    create_random_brain(&i);
                }

                let mut brain = Brain::new();
                brain.load(&format!("brains/{}.json", i));
                brain_list.push(brain);

                println!("Loaded brain \x1b[31mnumber {}\x1b[0m", i);
                i += 1;
            }

            let mut i = 0;
            let mut results: Vec<(u64, f64)> = vec![];
            println!("\x1b[35mStart tests... \x1b[0mThis might be long, so go grab a coffee");
            while i < brain_list.len() {
                println!("Testing \x1b[91mbrain {}\x1b[0m...", i);
                
                let brain = brain_list.get(i).unwrap();
                detector.switch_brain(brain);

                let test_results = detector.test(&repeats);
                if let None = test_results {
                    println!("Test for \x1b[91mbrain {}\x1b[31m failed\x1b[0m. Retrying...", i);
                    continue;
                }

                let test_results = test_results.unwrap();
                let mut sum: f64 = 0.0;
                for value in test_results.values() {
                    sum+=value;
                }

                results.push((i as u64, sum / (test_results.len() as f64)));
                i+=1;
            }

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let top: u64 = brains / 2;
            let mutations: u64 = top / 2;
            let fill: u64 = brains - (top + mutations);

            println!("\x1b[35mFinished testing phase\x1b[0m");

            let mut sum: f64 = 0.0;
            for (_a, b) in &results {
                sum += b;
            }
            let average = (sum / brains as f64) * 100.0;

            println!("Generation stats :\n    Best performance: \x1b[33m{}%\x1b[0m\n    Worst performance: \x1b[33m{}%\x1b[0m\n    Average performance: \x1b[33m{}%\x1b[0m", results.get(0 as usize).unwrap().1 * 100.0, results.get((brains - 1) as usize).unwrap().1 * 100.0, average);


            println!("Going next generation with:\n    conserve top: \x1b[33m{}\x1b[0m\n    Mutations of top \x1b[91m{}\x1b[0m : \x1b[33m{}\x1b[0m\n    Fill : \x1b[33m{}\x1b[0m", top, top, mutations, fill);

            let mut next_generation: Vec<Brain> = vec![];

            let mut j = 0;
            while j < top {
                let result_case = results.get(j as usize).unwrap();
                let corresponding = brain_list.get(result_case.0 as usize).unwrap();
                next_generation.push(corresponding.clone());

                j+=1;
            }
            let mut j = 0;
            while j < mutations {
                let result_case = results.get(j as usize).unwrap();
                let corresponding = brain_list.get(result_case.0 as usize).unwrap();

                let mutation = (*corresponding).mutate();
                next_generation.push(mutation);
                j+=1;
            }

            let mut j = 0;
            while j < fill {
                let random = Brain::random(0.0, 20.0);

                next_generation.push(random);
                j += 1;
            }

            println!("\x1b[35mSaving next generation...\x1b[0m");
            let mut j = 0;
            while j < next_generation.len() {
                let brain = next_generation.get(j as usize).unwrap();

                brain.write(&format!("brains/{}.json", j));
                j+=1;
            }

            let secs = rng_thread.random_range(5.0..8.0);

            println!("\x1b[35mSave finished... waiting \x1b[32m{}secs\x1b[0m", secs);
            let mut child = Command::new("sleep").arg(format!("{}", secs)).spawn().unwrap();
            let _res = child.wait().unwrap();
        }
    }
}
