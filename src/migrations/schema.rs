use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


pub mod message;
pub mod isp;
pub mod city;
pub mod region;
pub mod country;
pub mod ip;


pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(country::CREATE_COUNTRY_DB_SQL, []).expect("Error setting up country db");
    conn.execute(city::CREATE_CITY_DB_SQL, []).expect("Error setting up city db");
    conn.execute(region::CREATE_REGION_DB_SQL, []).expect("Error setting up Region db");
    conn.execute(isp::CREATE_ISP_DB_SQL, []).expect("Error setting up ISP db");
    conn.execute(ip::CREATE_IP_DB_SQL, []).expect("Error setting up IP db");
    conn.execute(message::CREATE_MESSAGE_DB_SQL, []).expect("Error setting up IP db");
    Ok(())
}


// Tests need to be run sequentially on non-existant db
//#[cfg(test)]
mod test {
    use crate::migrations::schema;
    use crate::migrations::schema::{message, isp, city, region, country, ip};
    use rusqlite::{Connection, Result};

    fn cleanup_db(filename: &str) {
        let file = std::path::Path::new(filename);
        let cleanup = std::fs::remove_file(file);
        assert!(cleanup.is_ok(), "db cleanup failed");
    }

    fn insert_all(conn: &Connection) -> Result<()> {
        let _ = country::insert_new_country(&conn, "Doitschland", Some("DE"), Some(0), Some(0), false).expect("Country insertion failed");
        let _ = region::insert_new_region(&conn, "Undetussen", "Doitschland", Some(0), Some(0), false).expect("Region insertion failed");
        let _ = city::insert_new_city(&conn, "Humburg", "Doitschland", "Undetussen",Some(0), Some(0), false).expect("City insertion failed");
        let _ = isp::insert_new_ISP(&conn,"Telecum", Some(0), Some(0), "Doitschland", false).expect("ISP insertion failed");
        let _ = ip::insert_new_IP(&conn, "111.233.456.678", "2022-03-11 23:45:31:512", "3.12", "59.79", "Telecum", "Humburg", Some("Undetussen"), "Doitschland", Some("DDE"), 0, false, 0).expect("IP insertion failed");
        let _ = message::insert_new_message(&conn, Option::None, "2022-03-11 23:45:31:512","OMG SUCH A MESSAGE", "111.233.456.678", "Doitschland", "Undetussen", "Humburg", "Telecum",true, false).expect("Message insertion failed");
        Ok(())       
    }

    #[test]
    pub fn test_db_create_tables() -> Result<()>{
        let db_name ="test.db";
        let conn = Connection::open(db_name)?;

        schema::create_tables(&conn)?;

        cleanup_db(db_name);
        Ok(())
    }

    #[test]
    pub fn test_db_insert() -> Result<()>{
        let db_name ="test_insert.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;

        insert_all(&conn)?;

        cleanup_db(db_name);
        Ok(())
    } 

    #[test]
    pub fn test_db_query() -> Result<()>{
        let db_name ="test_query.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;
        insert_all(&conn)?;

        let ip = ip::select_ip(&conn, "111.233.456.678").unwrap().unwrap();
        assert_eq!(ip.ip, "111.233.456.678".to_string());

        let isp = isp::select_isp(&conn, "Telecum").unwrap().unwrap();
        assert_eq!(isp.name, "Telecum".to_string());

        let isps = isp::get_all_isps(&conn).unwrap();
        assert_eq!(isps.into_iter().any(|c| {c.name == "Telecum"}), true);

        let isp = city::select_city(&conn, "Humburg").unwrap().unwrap();
        assert_eq!(isp.name, "Humburg".to_string());

        let cities = city::get_all_cities(&conn).unwrap();
        assert_eq!(cities.into_iter().any(|c| {c.name == "Humburg"}), true);

        let region = region::select_region(&conn, "Undetussen").unwrap().unwrap();
        assert_eq!(region.name, "Undetussen".to_string());

        let regions = region::get_all_regions(&conn).unwrap();
        assert_eq!(regions.into_iter().any(|c| {c.name == "Undetussen"}), true);

        let country = country::select_country(&conn, "Doitschland").unwrap().unwrap();
        assert_eq!(country.name, "Doitschland".to_string());

        let countries = country::get_all_countries(&conn).unwrap();
        assert_eq!(countries.into_iter().any(|c| {c.name == "Doitschland"}), true);

        cleanup_db(db_name);
        Ok(())
    } 


    #[test]
    pub fn test_db_query_message() -> Result<()>{
        let db_name ="test_message.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;
        insert_all(&conn)?;

        let msgs = message::select_message_by_ip(&conn, "111.233.456.678").unwrap();
        let mut rmsgs: Vec<message::Message> = vec![];
        let mut res: &str = "";
        for msg in msgs.into_iter() {
            let m = msg.unwrap_or_default();
            if m != message::Message::default() {
                rmsgs.push(m);
            }
        }
        if !rmsgs.is_empty() {
            // stuff in vec
            res = rmsgs[0].ip.as_str();
        }
        assert_eq!(res, "111.233.456.678".to_string());

        cleanup_db(db_name);
        Ok(())
    }


    #[test]
    pub fn test_db_query_timestamp() -> Result<()>{
        let db_name ="test_timestamp.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;
        insert_all(&conn)?;

        // by country
        let msgs: Vec<message::MiniMessage> = message::get_message_timestamps_by_country(&conn, "Doitschland").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);

        // by region
        let msgs = message::get_message_timestamps_by_region(&conn, "Undetussen").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);

        // by city        
        let msgs = message::get_message_timestamps_by_city(&conn, "Humburg").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);

        // by isp
        let msgs = message::get_message_timestamps_by_isp(&conn, "Telecum").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);

        cleanup_db(db_name);
        Ok(())
    }
}






