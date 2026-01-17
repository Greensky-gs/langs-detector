extern crate dotenv;
extern crate mysql;

mod data;
mod detector;
mod tables;

use dotenv::dotenv;
use mysql::PooledConn;

use crate::data::Data;
use crate::detector::Detector;
use crate::tables::start;
use std::collections::HashMap;

fn main() {
    dotenv().ok();

    let mut conn: PooledConn;
    match start() {
        Ok(res) => conn = res,
        Err(e) => return println!("FAIL : error : {:?}", e)
    };

    let mut data: Data = Data {
        importance: HashMap::new(),
        values: vec![]
    };

    data.build();

    let mut detector = Detector::new(conn);

    detector.show();

    let input = String::from("Une phrase aléatoire   en français avec un peu plus");

    println!("Result avec \x1b[33m{}\x1b[0m : {:?}", input, data.calc(&input));
}
