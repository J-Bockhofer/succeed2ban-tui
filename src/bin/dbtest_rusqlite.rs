
use rusqlite::{Connection, Result};

use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {

    
    let conn = Connection::open("cats.db")?;

    let connarc = Arc::new(Mutex::new(conn));

    let connarc2: Arc<Mutex<Connection>> = Arc::clone(&connarc);


    let _ = tokio::spawn(async move {
        let lock = connarc2.lock().await;
        lock.execute(
            "create table if not exists cat_colors (
            id integer primary key,
            name text not null unique
            )", []).expect("Something went wrong");
    }).await;

    let lock = connarc.lock().await;
    lock.execute(
        "create table if not exists cats (
        id integer primary key,
        name text not null,
        color_id integer not null references cat_colors(id)
        )", [])?;



    Ok(())

}