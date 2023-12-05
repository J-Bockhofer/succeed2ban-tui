use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Region {
    pub name: String,
    pub banned: usize,
    pub warnings: usize,
    pub country: String,
}
pub const CREATE_REGION_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS region(
    name TEXT NOT NULL PRIMARY KEY,
    banned INTEGER,
    warnings INTEGER,
    country TEXT NOT NULL REFERENCES country(name)
)
";

pub fn insert_new_region(conn: &Connection, name: &str, country: &str, num_banned:Option<usize>, num_messages:Option<usize>) -> Result<()> {
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);

    conn.execute(
        "INSERT OR REPLACE INTO region (name, banned, warnings, country) VALUES (?1, ?2, ?3, ?4)",
        (name, _banned, _msgs, country),
    )?;
    Ok(())
}
pub fn select_region(conn: &Connection, region:&str) -> Result<Option<Region>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM region WHERE name=:region;"
    )?;    
    let ip_iter = stmt.query_map(&[(":region", region)], |row| {
        Ok( Region {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
            country: row.get(3)?,
        })
    })?;

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.name == region.to_string() {
            return Ok(Some(aip))
        }
    }

    Ok(None)
}


pub fn get_all_regions(conn: &Connection) -> Result<Vec<Region>> {

    let mut regions: Vec<Region> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM region"
    )?; 
    
    let region_iter = stmt.query_map([], |row| {
        Ok( Region {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
            country: row.get(3)?,
        })
    })?;

    for region in region_iter {
        if region.is_ok() {
            let aregion = region.unwrap();
            regions.push(aregion);
        }
    }
    Ok(regions)

}