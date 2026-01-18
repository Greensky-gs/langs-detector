use mysql::{PooledConn, Opts, Pool, params};
use mysql::prelude::Queryable;
use serde_json::from_str;
use std::collections::HashMap;

pub type Ratios = HashMap<String, f32>;

pub struct Lang {
    pub name: String,
    pub content: String,
    pub ratios: Ratios
}

pub struct Database {
    connection: Option<PooledConn>,
    pub vectors: Vec<Lang>,
    pub count: HashMap<String, usize>,
    connected: bool,
}

impl Lang {
    pub fn print(&self) {
        println!("Lang {{\n    name   : \x1b[34m{}\x1b[0m\n    content: \x1b[34m{}\x1b[0m\n    ratios : \x1b[33m{:?}\x1b[0m\n}}", self.name, self.content, self.ratios);
    }
    pub fn from(name: String, ratios: Ratios, content: String) -> Lang {
        return Lang {
            name,
            content,
            ratios
        }
    }
}

impl Database {
    pub fn new() -> Database {
        let mut db = Database {
            connection: None,
            vectors: vec![],
            connected: false,
            count: HashMap::new()
        };
        db.count.insert("fr".to_string(), 0);
        db.count.insert("en".to_string(), 0);

        return db;
    }
    pub fn connect(&mut self) -> Option<bool> {
        if self.connected {
            panic!("Database already connected");
        }
        let opts: Opts;
        match Opts::from_url(&std::env::var("DATABASE_URL").unwrap()) {
            Ok(options) => opts = options,
            Err(e) => {
                println!("\x1b[33mDatabase options error :\x1b[0m {:?}", e);
                return None;
            }
        }
        
        let pool: Pool;
        match Pool::new(opts) {
            Ok(p) => pool = p,
            Err(e) => {
                println!("\x1b[33mDatabase pool error :\x1b[0m : {:?}", e);
                return None;
            }
        }
        
        let mut conn: PooledConn;
        match pool.get_conn() {
            Ok(c) => conn = c,
            Err(e) => {
                println!("\x1b[33mDatabase connection error :\x1b[0m {:?}", e);
                return None;
            }
        }

        let _ = conn.query_drop("CREATE TABLE IF NOT EXISTS langs ( name VARCHAR(255), ratios LONGTEXT, content VARCHAR(1023)");

        self.connection = Some(conn);
        self.connected = true;

        return Some(true)
    }
    pub fn load(&mut self) -> Option<usize> {
        if !self.connected {
            panic!("Database not connected");
        }

        let mut fail = false;
        match self.connection.as_mut().unwrap().query_map("SELECT name, content, ratios FROM langs", |(name, content, ratios): (String, String, String)| {
            if fail {
                return;
            }
            let map: HashMap<String, serde_json::Value> = from_str(&ratios).unwrap();
            let mut copy: Ratios = HashMap::new();

            for (k, v) in &map {
                if let serde_json::Value::Number(num) = v {
                    copy.insert(k.to_string(), num.as_f64().unwrap() as f32);
                }
            }

            if copy.len() != map.len() {
                fail = true;
                return;
            }

            let lang = Lang::from(name.clone(), copy, content);
            self.count.entry(name.clone()).and_modify(|e| *e += 1).or_insert(1);
            self.vectors.push(lang);
        }) {
            Ok(_) => {},
            Err(_) => return None
        }

        if fail {
            return None;
        }

        return Some(self.vectors.len());
    }
    pub fn add_entry(&mut self, entry: &Lang) {
        if !self.connected {
            panic!("Database not connected");
        }

        let json = serde_json::to_string(&entry.ratios).unwrap();

        match self.connection.as_mut().unwrap().exec_batch("INSERT INTO langs (name, content, ratios) values (:name, :content, :ratios)", vec![params! {
            "name" => entry.name.clone(),
            "content" => entry.content.clone(),
            "ratios" => json
        }]) {
            Ok(_) => {
                self.add_count(&entry.name.clone());
            },
            Err(e) => println!("{:?}", e)
        };
    }

    fn add_count(&mut self, entry: &String) {
        self.count.entry(entry.to_string()).and_modify(|e| *e += 1).or_insert(1);
    }
}
