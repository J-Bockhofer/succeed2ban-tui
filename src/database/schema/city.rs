
use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct City {
    pub name: String,
    pub banned: usize,
    pub warnings: usize,
    pub region: String,
    pub country: String,
    pub is_blocked: bool,
}
pub const CREATE_CITY_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS city(
    name TEXT NOT NULL PRIMARY KEY,
    banned INTEGER,
    warnings INTEGER,
    region TEXT REFERENCES region(name),
    country TEXT NOT NULL REFERENCES country(name),
    is_blocked INTEGER NOT NULL
)
";
pub fn insert_new_city(conn: &Connection, name: &str, country: &str, region:&str, num_banned:Option<usize>, num_messages:Option<usize>, is_blocked:bool) -> Result<()> {
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);

    conn.execute(
        "INSERT OR REPLACE INTO city (name, banned, warnings, region, country, is_blocked) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (name, _banned, _msgs, region, country, is_blocked),
    )?;
    Ok(())
}
pub fn select_city(conn: &Connection, city:&str) -> Result<Option<City>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM city WHERE name=:city;"
    )?;    
    let ip_iter = stmt.query_map(&[(":city", city)], |row| {
        Ok( City {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
            region: row.get(3)?,
            country: row.get(4)?,
            is_blocked: row.get(5)?,
        })
    })?;

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.name == city.to_string() {
            return Ok(Some(aip))
        }
    }

    Ok(None)
}

pub fn get_all_cities(conn: &Connection) -> Result<Vec<City>> {

    let mut cities: Vec<City> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM city"
    )?; 
    
    let city_iter = stmt.query_map([], |row| {
        Ok( City {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
            region: row.get(3)?,
            country: row.get(4)?,
            is_blocked: row.get(5)?,
        })
    })?;

    for city in city_iter {
        if city.is_ok() {
            let acity = city.unwrap();
            cities.push(acity);
        }
    }
    Ok(cities)

}