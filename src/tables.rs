use mysql::*;
use mysql::prelude::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Lang {
    pub name: String,
    pub ratios: HashMap<String, f32>,
    pub content: String
}

pub fn start() -> std::result::Result<PooledConn, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")?;

    let otps = Opts::from_url(&url)?;

    let pool = Pool::new(otps)?;
    let mut conn = pool.get_conn()?;

    conn.query_drop("CREATE TABLE IF NOT EXISTS langs ( name VARCHAR(255), ratios LONGTEXT, content VARCHAR(255))")?;

    return Ok(conn);
}

impl Lang {
    pub fn from(name: String, ratios: HashMap<String, f32>, content: String) -> Lang {
        return Lang {
            name, ratios, content
        }
    }
}
