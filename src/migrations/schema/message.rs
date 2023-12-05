use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};



#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub id: usize,
    pub created_at: String,
    pub text: String,
    pub ip: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub isp: String,
    pub is_jctl: bool,
    pub is_ban:bool,
}

pub const CREATE_MESSAGE_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL,
    text TEXT NOT NULL,
    ip TEXT NOT NULL REFERENCES ipmeta(ip),
    country TEXT NOT NULL REFERENCES country(name),
    region TEXT NOT NULL REFERENCES region(name),
    city TEXT NOT NULL REFERENCES city(name),
    isp TEXT NOT NULL REFERENCES isp(name),
    is_jctl INTEGER NOT NULL,
    is_ban INTEGER NOT NULL
)
";

pub fn insert_new_message(conn: &Connection, id: Option<usize>, created_at:&str,  text:&str, ip:&str, country:&str, region:&str, city:&str, isp:&str, is_jctl:bool, is_ban:bool) -> Result<()> {
    let _id = id.unwrap_or(0);
    if _id == 0 {
        conn.execute(
            "INSERT OR REPLACE INTO messages (created_at, text, ip, country, region, city, isp, is_jctl, is_ban) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (created_at, text, ip, country, region, city, isp, is_jctl, is_ban),
        )?;
    } else {
        conn.execute(
            "INSERT OR REPLACE INTO messages (id, created_at, text, ip, country, region, city, isp, is_jctl) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (_id, created_at, text, ip, country, region, city, isp, is_jctl, is_ban),
        )?;       
    }

    Ok(())
}
/// return all messages for a given ip
pub fn select_message_by_ip(conn: &Connection, ip:&str) -> Result<Vec<Option<Message>>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM messages WHERE ip=:ip;"
    )?;    
    let ip_iter = stmt.query_map(&[(":ip", ip)], |row| {
        Ok( Message {
            id: row.get(0)?,
            created_at: row.get(1)?,
            text: row.get(2)?,
            ip: row.get(3)?,
            country: row.get(4)?,
            region: row.get(5)?,
            city: row.get(6)?,
            isp: row.get(7)?,       
            is_jctl: row.get(8)?,
            is_ban: row.get(9)?,
        })
    })?;

    let mut results: Vec<Option<Message>> = vec![];

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.ip == ip {
            results.push(Some(aip));
        }
        else {
            results.push(Option::None);
        }
    }
    Ok(results)   
}

/// returns message timestamps for country
pub fn get_message_timestamps_by_country(conn: &Connection, country:&str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT created_at FROM messages WHERE country=:country;"
    )?;    
    let country_iter = stmt.query_map(&[(":country", country)], |row| {
        let mut message = Message::default();
        message.created_at = row.get(0)?;
        Ok(message)
    })?;

    let mut results: Vec<String> = vec![];

    for raip in country_iter {
        let aip = raip.unwrap_or_default();
        results.push(aip.created_at);
    }
    Ok(results)   
}

/// returns message timestamps for region
pub fn get_message_timestamps_by_region(conn: &Connection, region:&str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT created_at FROM messages WHERE region=:region;"
    )?;    
    let region_iter = stmt.query_map(&[(":region", region)], |row| {
        let mut message = Message::default();
        message.created_at = row.get(0)?;
        Ok(message)
    })?;

    let mut results: Vec<String> = vec![];

    for raip in region_iter {
        let aip = raip.unwrap_or_default();
        results.push(aip.created_at);
    }
    Ok(results)   
}

/// returns message timestamps for city
pub fn get_message_timestamps_by_city(conn: &Connection, city:&str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT created_at FROM messages WHERE city=:city;"
    )?;    
    let city_iter = stmt.query_map(&[(":city", city)], |row| {
        let mut message = Message::default();
        message.created_at = row.get(0)?;
        Ok(message)
    })?;

    let mut results: Vec<String> = vec![];

    for raip in city_iter {
        let aip = raip.unwrap_or_default();
        results.push(aip.created_at);
    }
    Ok(results)   
}

/// returns message timestamps for isp
pub fn get_message_timestamps_by_isp(conn: &Connection, isp:&str) -> Result<Vec<String>> {

    let mut stmt = conn.prepare(
        "SELECT created_at FROM messages WHERE isp=:isp;"
    )?;    
    let isp_iter = stmt.query_map(&[(":isp", isp)], |row| {
        let mut message = Message::default();
        message.created_at = row.get(0)?;
        Ok(message)
    })?;

    let mut results: Vec<String> = vec![];

    for raip in isp_iter {
        let aip = raip.unwrap_or_default();
        results.push(aip.created_at);
    }
    Ok(results)   
}