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

pub struct MetaInfo {
    pub country: country::Country,
    pub region: region::Region,
    pub city: city::City,
    pub isp: isp::ISP
  }
  
pub fn update_db_on_new_log(conn: &Connection, x:ip::IP, from_db:bool) -> MetaInfo {
    let mut country = country::select_country(conn, x.country.as_str()).unwrap_or_default().unwrap_or_default();
    if country == country::Country::default() {
        let _ = country::insert_new_country(conn, x.country.as_str(), Some(x.countrycode.as_str()), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
    }
    else {
        country.warnings += 1;
        if !from_db && x.is_banned {country.banned += 1;}
        let _ = country::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings), country.is_blocked).unwrap();
    }

    let mut region = region::select_region(conn, x.region.as_str()).unwrap_or_default().unwrap_or_default();
    if region == region::Region::default() {
        let _ = region::insert_new_region(conn, x.region.as_str(), x.country.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
    }
    else {
        region.warnings += 1;
        if !from_db && x.is_banned {region.banned += 1;}
        let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), region.is_blocked).unwrap();
    }

    let mut city = city::select_city(conn, x.city.as_str()).unwrap_or_default().unwrap_or_default();
    if city == city::City::default() {
        let _ = city::insert_new_city(conn, x.city.as_str(), x.country.as_str(), x.region.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
    }
    else {
        city.warnings += 1;
        if !from_db && x.is_banned {city.banned += 1;}
        let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(),city.region.as_str(), Some(city.banned), Some(city.warnings), city.is_blocked).unwrap();
    }

    let mut isp: isp::ISP = isp::select_isp(conn, x.isp.as_str()).unwrap_or_default().unwrap_or_default();
    if isp == isp::ISP::default() {
        let _ = isp::insert_new_ISP(conn, x.isp.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), x.country.as_str(), false).unwrap();
    }
    else {
        isp.warnings += 1;
        if !from_db && x.is_banned {isp.banned += 1;}
        let _ = isp::insert_new_ISP(conn, isp.name.as_str(), Some(isp.banned), Some(isp.warnings), x.country.as_str(), isp.is_blocked).unwrap();
    }
    return MetaInfo { country, region, city, isp }
}

pub fn update_ip_db_on_new_log(conn: &Connection, x: ip::IP, from_db: bool) {
    if !from_db {
      let _ = ip::insert_new_IP(conn, 
        x.ip.as_str(), x.created_at.as_str(), 
        x.lon.as_str(), x.lat.as_str(), 
        x.isp.as_str(), x.city.as_str(), 
        Some(x.region.as_str()), x.country.as_str(),
        Some(x.countrycode.as_str()), x.banned_times, 
          x.is_banned, x.warnings).unwrap();
    }
    else {
      // ip is in db
      let _ = ip::insert_new_IP(conn,
        x.ip.as_str(), x.created_at.as_str(), 
        x.lon.as_str(), x.lat.as_str(), 
        x.isp.as_str(), x.city.as_str(), 
        Some(x.region.as_str()), x.country.as_str(),
        Some(x.countrycode.as_str()), x.banned_times, 
          x.is_banned, x.warnings + 1).unwrap();
    }
  }


// Tests need to be run sequentially on non-existant db
//#[cfg(test)]
mod test {
    use crate::database::schema::{self, update_ip_db_on_new_log};
    use crate::database::schema::{message, isp, city, region, country, ip};
    use rusqlite::{Connection, Result};

    use super::update_db_on_new_log;

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


    #[test]
    pub fn test_db_update_on_new_log() -> Result<()>{
        let db_name ="test_update.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;
        insert_all(&conn)?;

        // "111.233.456.678", "2022-03-11 23:45:31:512", "3.12", "59.79", "Telecum", "Humburg", Some("Undetussen"), "Doitschland", Some("DDE"), 0, false, 0).expect("IP insertion failed")
        let before_city = city::select_city(&conn, "Humburg")?;

        let ip = ip::IP{
            ip: "111.233.456.678".to_string(),
            created_at: "2022-03-11 23:45:31:512".to_string(),
            lon: "3.12".to_string(),
            lat: "59.79".to_string(),
            isp: "Telecum".to_string(),
            city: "Humburg".to_string(),
            region: "Undetussen".to_string(),
            country: "Doitschland".to_string(),
            countrycode: "DDE".to_string(),
            banned_times: 0,
            is_banned: false,
            warnings: 1,
        };
        let meta = update_db_on_new_log(&conn, ip, true);

        let after_city = city::select_city(&conn, "Humburg")?;

        assert_eq!(before_city.unwrap().warnings+1, after_city.unwrap().warnings);

        cleanup_db(db_name);
        Ok(())
    } 

    #[test]
    pub fn test_db_update_ip_on_new_log() -> Result<()>{
        let db_name ="test_update_ip.db";
        let conn = Connection::open(db_name)?;
        schema::create_tables(&conn)?;
        insert_all(&conn)?;

        // "111.233.456.678", "2022-03-11 23:45:31:512", "3.12", "59.79", "Telecum", "Humburg", Some("Undetussen"), "Doitschland", Some("DDE"), 0, false, 0).expect("IP insertion failed")
        let before_ip = ip::select_ip(&conn, "111.233.456.678")?;

        let ip = ip::IP{
            ip: "111.233.456.678".to_string(),
            created_at: "2022-03-11 23:45:31:512".to_string(),
            lon: "3.12".to_string(),
            lat: "59.79".to_string(),
            isp: "Telecum".to_string(),
            city: "Humburg".to_string(),
            region: "Undetussen".to_string(),
            country: "Doitschland".to_string(),
            countrycode: "DDE".to_string(),
            banned_times: 0,
            is_banned: false,
            warnings: 0,
        };
        update_ip_db_on_new_log(&conn, ip, true);

        let after_ip= ip::select_ip(&conn, "111.233.456.678")?;

        assert_eq!(before_ip.unwrap().warnings+1, after_ip.unwrap().warnings);

        cleanup_db(db_name);
        Ok(())
    } 

}

