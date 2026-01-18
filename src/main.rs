mod brain;
mod database;
mod detector;
mod scrap;

use dotenv::dotenv;

use crate::database::Database;
use crate::brain::Brain;
use crate::detector::Detector;
use crate::scrap::start_scrapping;

fn main() {
    dotenv().ok();

    let mut detector = Detector::new();

    start_scrapping(&mut detector, &100);
}
