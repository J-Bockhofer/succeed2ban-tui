
use serde::{Deserialize, Serialize};
use rusqlite::{ Connection, Result};

use tokio;


use local_ip_address::local_ip;

#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct City {
    pub name: String,
    pub text: String
}
pub const CREATE_CITY_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS city(
    name TEXT NOT NULL PRIMARY KEY,
    info TEXT NOT NULL
)
";



pub fn insert_new_city(conn: &Connection, name: &str, text:&str) -> Result<()> {
    

    conn.execute(
        "INSERT OR REPLACE INTO city (name, info) VALUES (?1, ?2)",
        (name, text),
    )?;
    Ok(())
}

// This `derive` requires the `serde` dependency.
#[derive(Deserialize)]
struct Ip {
    origin: String,
}



pub async fn fetch_home() -> Result<String, Box<dyn std::error::Error>>{
    let url = format!("https://ident.me/");
    let resp = reqwest::get(url)
        .await?;
    let text = resp.text().await.unwrap();

    //println!("{:#?}", resp);
    Ok(text)  
}

#[tokio::main]
async fn main() -> Result<()>{
    use std::io::{stdin,stdout,Write};
    let mut s=String::new();



    //println!("ip: {}", ip.origin);

    let home = fetch_home().await.unwrap();
    println!("Got: {}", home);
    let dat = fetch_geolocation(&home).await.unwrap();
    println!("Geo: {:?}", dat);

    let conn = Connection::open("testinput.db")?;
    conn.execute(CREATE_CITY_DB_SQL, []).expect("Error setting up city db");
    print!("Please enter some text: ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }

    
    insert_new_city(&conn, "Test", s.as_str())?;


    println!("You typed: {}",s);

    Ok(())
}


fn fin_home() {
    let my_local_ip = local_ip();

    if let Ok(my_local_ip) = my_local_ip {
        println!("This is my local IP address: {:?}", my_local_ip);
    } else {
        println!("Error getting local IP: {:?}", my_local_ip);
    }
}

pub async fn fetch_geolocation(ip: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {

    let url = format!("http://ip-api.com/json/{ip}");
    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("{:#?}", resp);
    Ok(resp) 

}