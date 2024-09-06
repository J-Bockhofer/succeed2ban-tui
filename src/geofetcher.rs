use crate::migrations::schema::ip::IP;

pub async fn fetch_geolocation(ip: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("http://ip-api.com/json/{ip}");
    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    //println!("{:#?}", resp);
    Ok(resp) 
}

pub fn deserialize_geolocation(geodat: serde_json::Value, is_banned: bool) -> Option<IP> {

    let geoip = geodat.get("query");
    if geoip.is_none() {
        return None
    }
    let geoip = geoip.unwrap().as_str().unwrap();
        
    let timestamp = chrono::offset::Local::now().to_rfc3339();

    let geolat = geodat.get("lat").unwrap().as_number().unwrap().to_string(); // CRASH ON PI
    let geolon = geodat.get("lon").unwrap().as_number().unwrap().to_string();
    let geoisp = String::from(geodat.get("isp").unwrap().as_str().unwrap());

    let geocountry = String::from(geodat.get("country").unwrap().as_str().unwrap());
    let geocity = String::from(geodat.get("city").unwrap().as_str().unwrap());
    let geocountrycode = String::from(geodat.get("countryCode").unwrap().as_str().unwrap());
    let georegionname = String::from(geodat.get("regionName").unwrap().as_str().unwrap());


    let mut geodata: IP = IP::default();
    geodata.created_at = timestamp;
    geodata.ip = geoip.to_string();
    geodata.lat = geolat;
    geodata.lon = geolon;
    geodata.isp = geoisp;
    geodata.is_banned = is_banned;
    geodata.banned_times = match is_banned {false => 0, true => 1};
    geodata.country = geocountry;
    geodata.countrycode = geocountrycode;
    geodata.city = geocity;
    geodata.region = georegionname;
    geodata.warnings = 1;

    Some(geodata)
}

pub async fn fetch_home() -> Result<String, Box<dyn std::error::Error>>{
    let url = format!("https://ident.me/");
    let resp = reqwest::get(url)
        .await?;
    let text = resp.text().await.unwrap();
    //println!("{:#?}", resp);
    Ok(text)  
}

