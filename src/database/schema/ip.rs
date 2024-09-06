

use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct IP {
    pub ip: String,
    pub created_at: String,
    pub lon: String,
    pub lat: String,
    pub isp: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub countrycode: String,
    pub banned_times: usize,
    pub is_banned: bool,
    pub warnings: usize,
}
pub const CREATE_IP_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS ipmeta(
    ip TEXT NOT NULL PRIMARY KEY,
    created_at TEXT NOT NULL,
    lon TEXT NOT NULL,
    lat TEXT NOT NULL,
    isp TEXT NOT NULL REFERENCES isp(name),
    city TEXT NOT NULL REFERENCES city(name),
    region TEXT REFERENCES region(name),
    country TEXT NOT NULL REFERENCES country(name),
    countrycode TEXT,
    banned_times INTEGER NOT NULL,
    is_banned INTEGER NOT NULL,
    warnings INTEGER NOT NULL
)
";

#[allow(non_snake_case)]
pub fn insert_new_IP(conn: &Connection, 
    ip: &str, 
    created_at: &str,
    lon: &str,
    lat: &str,
    isp: &str,
    city: &str,
    region: Option<&str>,
    country: &str,
    countrycode: Option<&str>,
    num_banned: usize,
    is_banned: bool,
    num_warnings: usize) -> Result<()> {

    let _region = region.unwrap_or("");
    let _cc = countrycode.unwrap_or("");
    conn.execute(
        "INSERT OR REPLACE INTO ipmeta (ip, created_at, lon, lat, isp, city, region, country, countrycode, banned_times, is_banned, warnings) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        (ip, created_at, lon, lat, isp, city, _region, country, _cc, num_banned, is_banned, num_warnings),
    )?;
    Ok(())
}

pub fn select_ip(conn: &Connection, ip:&str) -> Result<Option<IP>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ipmeta WHERE ip=:ip;"
    )?;    
    let ip_iter = stmt.query_map(&[(":ip", ip)], |row| {
        Ok( IP {
            ip: row.get(0)?,
            created_at: row.get(1)?,
            lon: row.get(2)?,
            lat: row.get(3)?,
            isp: row.get(4)?,
            city: row.get(5)?,
            region: row.get(6)?,
            country: row.get(7)?,
            countrycode: row.get(8)?,
            banned_times: row.get(9)?,
            is_banned: row.get(10)?,
            warnings: row.get(11)?,
        })
    })?;

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.ip == ip.to_string() {
            return Ok(Some(aip))
        }
    }

    Ok(None)
}