use crate::detector::Detector;
use rand::{Rng,rng};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::process::Command;
use webpage::{Webpage, WebpageOptions};

fn read_wikipedia(detector: &mut Detector, lang: &String, lang_url: String) -> Option<bool> {
    let info: Webpage;
    match Webpage::from_url(&lang_url, WebpageOptions::default()) {
        Ok(w) => info = w,
        Err(e) => {
            println!("Error while fetching : {:?}", e);
            return None;
        }
    }

    println!("Fetched \x1b[90m{}\x1b[0m", info.http.url);

    let document = Html::parse_document(&info.http.body);
    let selector = Selector::parse(".mw-body-content .mw-content-ltr p").unwrap();

    let mut appended = 0;
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join("");

        if text.len() > 100 && text.len() < 1024 {
            println!("Appending a text of length \x1b[33m{}\x1b[0m for \x1b[34m{}\x1b[0m", text.len(), lang);
            detector.add_entry(lang, &text);

            appended += 1;
        }
    }

    if appended == 0 {
        println!("Nothing to append for \x1b[34m{}\x1b[0m. Considering this as a \x1b[31mFAIL\x1b[0m", lang);
        return None;
    }
    

    return Some(true);
}
pub fn start_scrapping(detector: &mut Detector, repeats: &u64) {
    let mut thread = rng();

    let mut langs: HashMap<String, String> = HashMap::new();
    langs.insert("fr".to_string(), "https://fr.wikipedia.org/wiki/Spécial:Page_au_hasard".to_string());
    langs.insert("en".to_string(), "https://en.wikipedia.org/wiki/Special:Random".to_string());

    let mut i = 0;
    while i < *repeats {
        let key = detector.get_least_registered_lang();
        if let None = key {
            break;
        }
        let key = key.unwrap();
        let target = langs.get(&key).unwrap();

        match read_wikipedia(detector, &key, target.to_string()) {
            Some(_) => {
                println!("Read successful for \x1b[34m{}\x1b[0m", key);
                i+=1;
            },
            None => {
                let secs = thread.random_range(1.0..2.5);
                println!("Fetch failed, trying again in \x1b[32m{}s\x1b[0m", secs);

                let mut child = Command::new("sleep").arg(format!("{}", secs)).spawn().unwrap();
                let _res = child.wait().unwrap();

                println!("Trying again");
            }
        }
    }
}
