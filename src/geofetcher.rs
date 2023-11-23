pub async fn fetch_geolocation(ip: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {

    let url = format!("http://ip-api.com/json/{ip}");
    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    //println!("{:#?}", resp);
    Ok(resp) 

}