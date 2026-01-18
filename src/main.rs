mod database;
mod brain;
mod detector;

use dotenv::dotenv;

use crate::database::Database;
use crate::brain::Brain;
use crate::detector::Detector;

fn main() {
    dotenv().ok();
    println!("Hello world!");

    let mut detector = Detector::new();


    let input = String::from("Je suis un texte en français");

    detector.detect(input);
}
