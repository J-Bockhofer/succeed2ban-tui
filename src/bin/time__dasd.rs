use chrono;

fn main() {
    let stampstr = chrono::offset::Local::now().to_rfc3339();
    //let stampstr = "2022-03-11 23:45:31.51254645+01:00".to_string();
    let cts = chrono::DateTime::parse_from_rfc3339(&stampstr).unwrap();

    //let dt = chrono::DateTime::with_timezone(&self, chrono::TimeZone::from_local_datetime(&self, chrono::NaiveDateTime::now()))
    println!("{:?}",cts);
    println!("{:?}", chrono::offset::Local::now());
}