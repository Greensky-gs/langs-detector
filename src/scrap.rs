use crate::detector::Detector;
use rand::{Rng,rng};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::process::Command;
use webpage::{Webpage, WebpageOptions};


pub fn get_urls_map() -> HashMap<String, String> {
    let mut langs: HashMap<String, String> = HashMap::new();
    langs.insert("fr".to_string(), "https://fr.wikipedia.org/wiki/Spécial:Page_au_hasard".to_string());
    langs.insert("en".to_string(), "https://en.wikipedia.org/wiki/Special:Random".to_string());

    return langs;
}

pub fn get_wikipedia_content(lang_url: String) -> Option<Vec<String>> {
    let info = Webpage::from_url(&lang_url, WebpageOptions::default());
    if let Err(e) = info {
        println!("Error while fetching : {:?}", e);
        return None;
    }
    let info = info.unwrap();

    println!("Fetched \x1b[90m{}\x1b[0m", info.http.url);

    let document = Html::parse_document(&info.http.body);
    let selector = Selector::parse(".mw-body-content .mw-content-ltr p").unwrap();

    let mut content: Vec<String> = vec![];
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join("");
        content.push(text);
    }

    return Some(content);
}

fn read_wikipedia(detector: &mut Detector, lang: &String, lang_url: String) -> Option<bool> {
    let texts = get_wikipedia_content(lang_url);
    if let None = texts {
        println!("Cannot fetch content");
        return None;
    }
    let texts = texts.unwrap();

    let mut appended = 0;
    for element in texts {
        if element.len() > 100 && element.len() < 1024 {
            println!("Appending a element of length \x1b[33m{}\x1b[0m for \x1b[34m{}\x1b[0m", element.len(), lang);
            detector.add_entry(lang, &element);

            appended += 1;
        }
    }

    if appended == 0 {
        println!("Nothing to append for \x1b[34m{}\x1b[0m. Considering this as a \x1b[31mFAIL\x1b[0m", lang);
        return None;
    }    

    return Some(true);
}
pub fn get_text_vectors(chunk_size: &u64) -> Option<Vec<(String, String)>> {
    let langs = get_urls_map();

    let mut vectors: Vec<(String, String)> = vec![];
    for key in langs.keys() {
        println!("Searching tests for \x1b[33m{}\x1b[0m", key);
        let url = langs.get(key).unwrap();

        let mut i: u64 = 0;
        while i < *chunk_size {
            let content = get_wikipedia_content(url.to_string());
            if let None = content {
                println!("No \x1b[31mcontent found\x1b[0m. Aborting");
                return None;
            }
            let content = content.unwrap();
            let mut j = 0;
            while j < content.len() && i < *chunk_size {
                let text = content.get(j).unwrap().clone();
                let text = text.trim();
                if text.len() > 100 && text.len() < 1024 {
                    vectors.push((key.to_string(), content.get(j).unwrap().clone()));
                    i+=1;
                }
                j+=1;
            }
        }
    }

    return Some(vectors);
}
pub fn start_scrapping(detector: &mut Detector, repeats: &u64) {
    let mut thread = rng();
    let langs = get_urls_map();

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
