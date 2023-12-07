
use serde::{Deserialize, Serialize};
use rusqlite::{ Connection, Result};

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


fn main() -> Result<()>{
    use std::io::{stdin,stdout,Write};
    let mut s=String::new();

    fin_home();

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