extern crate mysql;

use mysql::PooledConn;
use serde_json::from_str;

use crate::data::Data;
use crate::tables::Lang;
use crate::mysql::prelude::Queryable;
use std::collections::HashMap;

pub struct Detector {
    ready: bool,
    conn: PooledConn,
    vectors: Vec<Lang>,
    data: Data
}

impl Detector {
    pub fn new(conn: PooledConn) -> Detector {
        return Detector {
            ready: false,
            conn,
            vectors: vec![],
            data: Data::new()
        }
    }

    pub fn show(self) {
        println!("{:?}", self.vectors);
    }

    pub fn load(&mut self) -> Option<usize> {
        let mut failed = false;

        match self.conn.query_map("select name, ratios, content from langs", |(name, ratios, content): (String, String, String)| {
            let map: HashMap<String, serde_json::Value> = from_str(&ratios).unwrap();
            let mut copy: HashMap<String, f32> = HashMap::new();

            for (k, v) in &map {
                if let serde_json::Value::Number(num) = v {
                    if let Some(f) = num.as_f64() {
                        copy.insert(k.to_string(), f as f32);
                    }
                }
            }

            if copy.len() != map.len() {
                return failed = true;
            }

            self.vectors.push(Lang::from(name, copy, content));
        }) {
            Ok(lst) => {
                if failed {
                    return None;
                }
                return Some(self.vectors.len());
            },
            Err(e) => {
                println!("Failed : {:?}", e);
                return None;
            }
        }
    }

    pub fn recalc(&mut self) {
        for vector in &mut self.vectors {
            let map = self.data.calc(&vector.content);
            vector.ratios = map;
        }
    }

}
