use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct ISP {
    pub name: String,
    pub banned: usize,
    pub warnings: usize,
}
pub const CREATE_ISP_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS isp(
    name TEXT NOT NULL PRIMARY KEY,
    banned INTEGER,
    messages INTEGER
)
";
#[allow(non_snake_case)]
pub fn insert_new_ISP(conn: &Connection, name: &str, num_banned:Option<usize>, num_messages:Option<usize>) -> Result<()> {
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);

    conn.execute(
        "INSERT OR REPLACE INTO isp (name, banned, messages) VALUES (?1, ?2, ?3)",
        (name, _banned, _msgs),
    )?;
    Ok(())
}

pub fn select_isp(conn: &Connection, isp:&str) -> Result<Option<ISP>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM isp WHERE name=:isp;"
    )?;    
    let ip_iter = stmt.query_map(&[(":isp", isp)], |row| {
        Ok( ISP {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
        })
    })?;

    for raip in ip_iter {
        let aip = raip.unwrap_or_default();
        if aip.name == isp.to_string() {
            return Ok(Some(aip))
        }
    }

    Ok(None)
}

pub fn get_all_isps(conn: &Connection) -> Result<Vec<ISP>> {

    let mut isps: Vec<ISP> = vec![];
    let mut stmt = conn.prepare(
        "SELECT * FROM isp"
    )?; 
    
    let isp_iter = stmt.query_map([], |row| {
        Ok( ISP {
            name: row.get(0)?,
            banned: row.get(1)?,
            warnings: row.get(2)?,
        })
    })?;

    for isp in isp_iter {
        if isp.is_ok() {
            let aisp = isp.unwrap();
            isps.push(aisp);
        }
    }
    Ok(isps)

}