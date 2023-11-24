
use regex::Regex;
//let re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}).unwrap();
#[tokio::main]
async fn main() {

    // 2023-11-23 05:15:20,065 fail2ban.ipdns          [836]: WARNING Unable to find a corresponding IP address for whitelist-IP: [Errno -2] Name or service not known
    // 2023-11-23 05:30:26,385 fail2ban.filter         [836]: INFO    [sshd] Found 1.12.60.11 - 2023-11-23 05:30:26

    let teststr = "2023-11-23 05:30:26,385 fail2ban.filter         [836]: INFO    [sshd] Found 1.12.60.11 - 2023-11-23 05:30:26";

    let mut splitword: &str ="(/%&$§";

    let ip_re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
    let results: Vec<&str> = ip_re
      .captures_iter(teststr)
      .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
      .collect();
    let mut cip: &str = "";
    if !results.is_empty() {
      cip = results[0];
    }
    let ban_re = Regex::new(r"Ban").unwrap();
    let found_re = Regex::new(r"Found").unwrap();
    if ban_re.is_match(teststr) {splitword = "Ban";}
    else if found_re.is_match(teststr)  {splitword = "Found";}
    let mut fparts: Vec<&str> = vec![""];
    let mut sparts: Vec<&str> = vec![""];
    if !cip.is_empty() {
      fparts = teststr.split(cip).collect();
      sparts = fparts[0].split(splitword).collect();
      
      println!("{}",sparts.len());

    } // assume only left and right side - not multiple ips in one line

    println!("{}",cip);
    println!("{:?}", fparts);
    println!("{:?}", sparts);
    //let myip = "202.157.189.170";

    //let geodat = fetch_geolocation(myip).await.unwrap_or(serde_json::Value::default());

    //let geolat = geodat.get("lat").unwrap();
    //let geolon = geodat.get("lon").unwrap();


    //println!("{:?}", geolat.as_number().unwrap().to_string().parse::<f64>().unwrap());  
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