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
}

pub const CREATE_COUNTRY_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS country(
    name TEXT NOT NULL PRIMARY KEY,
    code TEXT,
    banned INTEGER,
    warnings INTEGER
)
";

pub fn insert_new_country(conn: &Connection, name: &str, code:Option<&str>, num_banned:Option<usize>, num_messages:Option<usize>) -> Result<()> {
    let _code = code.unwrap_or("");
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);
    conn.execute(
        "INSERT OR REPLACE INTO country (name, code, banned, warnings) VALUES (?1, ?2, ?3, ?4)",
        (name, code, _banned, _msgs),
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


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct City {
    pub name: String,
    pub banned: usize,
    pub warnings: usize,
    pub region: String,
    pub country: String,
}
pub const CREATE_CITY_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS city(
    name TEXT NOT NULL PRIMARY KEY,
    banned INTEGER,
    warnings INTEGER,
    region TEXT REFERENCES region(name),
    country TEXT NOT NULL REFERENCES country(name)
)
";
pub fn insert_new_city(conn: &Connection, name: &str, country: &str, region:&str, num_banned:Option<usize>, num_messages:Option<usize>) -> Result<()> {
    let _banned = num_banned.unwrap_or(0);
    let _msgs = num_messages.unwrap_or(0);

    conn.execute(
        "INSERT OR REPLACE INTO city (name, banned, warnings, region, country) VALUES (?1, ?2, ?3, ?4, ?5)",
        (name, _banned, _msgs, region, country),
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


#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub id: usize,
    pub created_at: String,
    pub text: String,
    pub ip: String,
    pub is_jctl: bool,
    pub is_ban:bool,
}

pub const CREATE_MESSAGE_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL,
    text TEXT NOT NULL,
    ip TEXT NOT NULL REFERENCES ipmeta(ip),
    is_jctl INTEGER NOT NULL,
    is_ban INTEGER NOT NULL
)
";

pub fn insert_new_message(conn: &Connection, id: Option<usize>, created_at:&str,  text:&str, ip:&str, is_jctl:bool, is_ban:bool) -> Result<()> {
    let _id = id.unwrap_or(0);
    if _id == 0 {
        conn.execute(
            "INSERT OR REPLACE INTO messages (created_at, text, ip, is_jctl, is_ban) VALUES (?1, ?2, ?3, ?4, ?5)",
            (created_at, text, ip, is_jctl, is_ban),
        )?;
    } else {
        conn.execute(
            "INSERT OR REPLACE INTO messages (id, created_at, text, ip, is_jctl) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (_id, created_at, text, ip, is_jctl, is_ban),
        )?;       
    }

    Ok(())
}
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
            is_jctl: row.get(4)?,
            is_ban: row.get(5)?,
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



#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Geodata {
  pub ip: String,
  pub country: String,
  pub city: String,
  pub country_code: String,
  pub isp: String,
  pub region_name: String,
  pub lat: String, // .parse::<f64>().unwrap()
  pub lon: String, // .parse::<f64>().unwrap()
}
impl Geodata {
  pub fn new() -> Self {
    Self::default()
  }
}


// Tests need to be run seuentially on non-existant db
#[cfg(test)]
mod test {
    use crate::migrations::schema;
    use rusqlite::{Connection, Result};
    #[test]
    pub fn test_create_schema() -> Result<()>{
        let conn = Connection::open("test.db")?;
        conn.execute(schema::CREATE_COUNTRY_DB_SQL, []).expect("Error setting up country db");
        conn.execute(schema::CREATE_CITY_DB_SQL, []).expect("Error setting up city db");
        conn.execute(schema::CREATE_REGION_DB_SQL, []).expect("Error setting up Region db");
        conn.execute(schema::CREATE_ISP_DB_SQL, []).expect("Error setting up ISP db");
        conn.execute(schema::CREATE_IP_DB_SQL, []).expect("Error setting up IP db");
        conn.execute(schema::CREATE_MESSAGE_DB_SQL, []).expect("Error setting up IP db");
        Ok(())
    }

    #[test]
    pub fn test_insert() -> Result<()>{
        let conn = Connection::open("test.db")?;
        let _ = schema::insert_new_country(&conn, "Doitschland", Some("DE"), Some(0), Some(0)).expect("Country insertion failed");
        let _ = schema::insert_new_region(&conn, "Undetussen", "Doitschland", Some(0), Some(0)).expect("Region insertion failed");
        let _ = schema::insert_new_city(&conn, "Humburg", "Doitschland", "Undetussen",Some(0), Some(0)).expect("City insertion failed");
        let _ = schema::insert_new_ISP(&conn,"Telecum", Some(0), Some(0)).expect("ISP insertion failed");
        let _ = schema::insert_new_IP(&conn, "111.233.456.678", "2022-03-11 23:45:31:512", "3.12", "59.79", "Telecum", "Humburg", Some("Undetussen"), "Doitschland", Some("DDE"), 0, false, 0).expect("IP insertion failed");
        let _ = schema::insert_new_message(&conn, Option::None, "2022-03-11 23:45:31:512","OMG SUCH A MESSAGE", "111.233.456.678", true, false).expect("Message insertion failed");
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_ip() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let ip = schema::select_ip(&conn, "111.233.456.678").unwrap().unwrap();
        assert_eq!(ip.ip, "111.233.456.678".to_string());
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_isp() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = schema::select_isp(&conn, "Telecum").unwrap().unwrap();
        assert_eq!(isp.name, "Telecum".to_string());
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_city() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = schema::select_city(&conn, "Humburg").unwrap().unwrap();
        assert_eq!(isp.name, "Humburg".to_string());
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_region() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = schema::select_region(&conn, "Undetussen").unwrap().unwrap();
        assert_eq!(isp.name, "Undetussen".to_string());
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_country() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = schema::select_country(&conn, "Doitschland").unwrap().unwrap();
        assert_eq!(isp.name, "Doitschland".to_string());
        Ok(())
    }

    #[test]
    pub fn test_query_and_serialize_message_by_ip() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let msgs = schema::select_message_by_ip(&conn, "111.233.456.678").unwrap();

        let mut rmsgs: Vec<schema::Message> = vec![];

        let mut res: &str = "";

        for msg in msgs.into_iter() {
            let m = msg.unwrap_or_default();
            if m != schema::Message::default() {
                rmsgs.push(m);
            }
        }

        if !rmsgs.is_empty() {
            // stuff in vec
            res = rmsgs[0].ip.as_str();
        }


        assert_eq!(res, "111.233.456.678".to_string());
        Ok(())
    }


}






