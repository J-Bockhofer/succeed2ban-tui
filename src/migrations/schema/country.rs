use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};

// TEXT as ISO8601 strings ("YYYY-MM-DD HH:MM:SS.SSS"). 
#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Country {
    pub name: String,
    pub code: String,
    pub banned: usize,
    pub warnings: usize,
    pub is_blocked: bool,
}

pub const CREATE_COUNTRY_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS country(
    name TEXT NOT NULL PRIMARY KEY,
    code TEXT,
    banned INTEGER,
    warnings INTEGER,
    is_blocked INTEGER NOT NULL
)
";

pub fn insert_new_country(conn: &Connection, name: &str, code:Option<&str>, num_banned:Option<usize>, num_messages:Option<usize>, is_blocked: bool) -> Result<()> {
    let _code = code.unwrap_or("");
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);
    conn.execute(
        "INSERT OR REPLACE INTO country (name, code, banned, warnings, is_blocked) VALUES (?1, ?2, ?3, ?4, ?5)",
        (name, code, _banned, _msgs, is_blocked),
    )?;
    Ok(())
}
pub fn select_country(conn: &Connection, country:&str) -> Result<Option<Country>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM country WHERE name=:country;"
    )?;    
    let ip_iter = stmt.query_map(&[(":country", country)], |row| {
        Ok( Country {
            name: row.get(0)?,
            code: row.get(1)?,
            banned: row.get(2)?,
            warnings: row.get(3)?,
            is_blocked: row.get(4)?,
        })
    })?;

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.name == country.to_string() {
            return Ok(Some(aip))
        }
    }

    Ok(None)
}

pub fn get_all_countries(conn: &Connection) -> Result<Vec<Country>> {

    let mut countries: Vec<Country> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM country"
    )?; 
    
    let country_iter = stmt.query_map([], |row| {
        Ok( Country {
            name: row.get(0)?,
            code: row.get(1)?,
            banned: row.get(2)?,
            warnings: row.get(3)?,
            is_blocked: row.get(4)?,
        })
    })?;

    for country in country_iter {
        if country.is_ok() {
            let acountry = country.unwrap();
            countries.push(acountry);
        }
    }
    Ok(countries)

}