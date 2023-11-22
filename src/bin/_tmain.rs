use regex::Regex;

//let re = Regex::new(r"(\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3})").unwrap();
fn main() {
    let re = Regex::new(r"(\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3})").unwrap();
    let hay = "\
    116.114.32.7
    path/to/bar:90:Something, Something, Something, Dark Side
    path/to/baz:12.334.332.88
    ";
    
    let results: Vec<&str> = re
        .captures_iter(hay)
        .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
        .collect();

    assert_eq!(results, vec![
    ("116.114.32.7"),
    ("12.334.332.88"),
    ]);  

    println!("{:?}", results);  
}
