use std::default;
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result};


pub mod message;
pub mod isp;
pub mod city;
pub mod region;
pub mod country;
pub mod ip;




// Tests need to be run sequentially on non-existant db
#[cfg(test)]
mod test {
    use crate::migrations::schema;
    use crate::migrations::schema::{message, isp, city, region, country, ip};
    use rusqlite::{Connection, Result};
    use serial_test::serial;
    #[test]
    #[serial]
    pub fn test_create_schema() -> Result<()>{
        let conn = Connection::open("test.db")?;
        conn.execute(country::CREATE_COUNTRY_DB_SQL, []).expect("Error setting up country db");
        conn.execute(city::CREATE_CITY_DB_SQL, []).expect("Error setting up city db");
        conn.execute(region::CREATE_REGION_DB_SQL, []).expect("Error setting up Region db");
        conn.execute(isp::CREATE_ISP_DB_SQL, []).expect("Error setting up ISP db");
        conn.execute(ip::CREATE_IP_DB_SQL, []).expect("Error setting up IP db");
        conn.execute(message::CREATE_MESSAGE_DB_SQL, []).expect("Error setting up IP db");
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_insert() -> Result<()>{
        let conn = Connection::open("test.db")?;
        let _ = country::insert_new_country(&conn, "Doitschland", Some("DE"), Some(0), Some(0), false).expect("Country insertion failed");
        let _ = region::insert_new_region(&conn, "Undetussen", "Doitschland", Some(0), Some(0), false).expect("Region insertion failed");
        let _ = city::insert_new_city(&conn, "Humburg", "Doitschland", "Undetussen",Some(0), Some(0), false).expect("City insertion failed");
        let _ = isp::insert_new_ISP(&conn,"Telecum", Some(0), Some(0), "Doitschland", false).expect("ISP insertion failed");
        let _ = ip::insert_new_IP(&conn, "111.233.456.678", "2022-03-11 23:45:31:512", "3.12", "59.79", "Telecum", "Humburg", Some("Undetussen"), "Doitschland", Some("DDE"), 0, false, 0).expect("IP insertion failed");
        let _ = message::insert_new_message(&conn, Option::None, "2022-03-11 23:45:31:512","OMG SUCH A MESSAGE", "111.233.456.678", "Doitschland", "Undetussen", "Humburg", "Telecum",true, false).expect("Message insertion failed");
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_query_and_serialize_ip() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let ip = ip::select_ip(&conn, "111.233.456.678").unwrap().unwrap();
        assert_eq!(ip.ip, "111.233.456.678".to_string());
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_query_and_serialize_isp() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = isp::select_isp(&conn, "Telecum").unwrap().unwrap();
        assert_eq!(isp.name, "Telecum".to_string());
        Ok(())
    }
    #[test]
    #[serial]
    pub fn test_query_all_isps() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isps = isp::get_all_isps(&conn).unwrap();
        assert_eq!(isps.into_iter().any(|c| {c.name == "Telecum"}), true);
        Ok(())    
    }

    #[test]
    #[serial]
    pub fn test_query_and_serialize_city() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = city::select_city(&conn, "Humburg").unwrap().unwrap();
        assert_eq!(isp.name, "Humburg".to_string());
        Ok(())
    }
    #[test]
    #[serial]
    pub fn test_query_all_cities() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let cities = city::get_all_cities(&conn).unwrap();
        assert_eq!(cities.into_iter().any(|c| {c.name == "Humburg"}), true);
        Ok(())    
    }

    #[test]
    #[serial]
    pub fn test_query_and_serialize_region() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = region::select_region(&conn, "Undetussen").unwrap().unwrap();
        assert_eq!(isp.name, "Undetussen".to_string());
        Ok(())
    }
    #[test]
    #[serial]
    pub fn test_query_all_regions() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let regions = region::get_all_regions(&conn).unwrap();
        assert_eq!(regions.into_iter().any(|c| {c.name == "Undetussen"}), true);
        Ok(())    
    }


    #[test]
    #[serial]
    pub fn test_query_and_serialize_country() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let isp = country::select_country(&conn, "Doitschland").unwrap().unwrap();
        assert_eq!(isp.name, "Doitschland".to_string());
        Ok(())
    }
    #[test]
    #[serial]
    pub fn test_query_all_countries() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let countries = country::get_all_countries(&conn).unwrap();
        assert_eq!(countries.into_iter().any(|c| {c.name == "Doitschland"}), true);
        Ok(())    
    }


    #[test]
    #[serial]
    pub fn test_query_and_serialize_message_by_ip() -> Result<()> {
        let conn = Connection::open("test.db")?;
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
        Ok(())
    }


    #[test]
    #[serial]
    pub fn test_get_timestamps_by_country() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let msgs: Vec<message::MiniMessage> = message::get_message_timestamps_by_country(&conn, "Doitschland").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);
        Ok(())
    }    
    #[test]
    #[serial]
    pub fn test_get_timestamps_by_region() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let msgs = message::get_message_timestamps_by_region(&conn, "Undetussen").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_get_timestamps_by_city() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let msgs = message::get_message_timestamps_by_city(&conn, "Humburg").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);
        Ok(())
    }

    #[test]
    #[serial]
    pub fn test_get_timestamps_by_isp() -> Result<()> {
        let conn = Connection::open("test.db")?;
        let msgs = message::get_message_timestamps_by_isp(&conn, "Telecum").unwrap();
        let mut res: message::MiniMessage = message::MiniMessage::default();
        if !msgs.is_empty() {
            // stuff in vec
            res = msgs[0].clone();
        }
        let ass = message::MiniMessage{ip:"111.233.456.678".to_string(), created_at:"2022-03-11 23:45:31:512".to_string()};
        assert_eq!(res, ass);
        Ok(())
    }    
}






