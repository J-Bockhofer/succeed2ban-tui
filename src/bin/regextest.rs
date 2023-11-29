
use std::sync::OnceLock;
use regex::Regex;

use std::time::Instant;

pub static TLM_REGEX: OnceLock<Regex> = OnceLock::new();

pub static TLM_VEC: OnceLock<Vec<&str>> = OnceLock::new();

pub const NUM_ITER: usize = 10000;


pub static TLM_REGEX_SHORT: OnceLock<Regex> = OnceLock::new();
pub static TLM_WORD: OnceLock<&str> = OnceLock::new(); // unnecessary but to keep both tests the same

fn main() {

    // NOTE: the regex is configured to ONLY match when the ENTIRE input matches, remove ^ at start and $ at end to change this. 
    let tlm_re = TLM_REGEX.get_or_init(|| Regex::new(r"^(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)$").unwrap());
    let tlm_vec = TLM_VEC.get_or_init(|| vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]);

    let tlm_re_short = TLM_REGEX_SHORT.get_or_init(|| Regex::new(r"^(Sep)$").unwrap());
    let tlm_word = TLM_WORD.get_or_init(|| "Sep");


    let word = "justaword";

    let matching_word = "Sep";

    let matching_early_word = "Jan";

    let really_long_word = "
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus quis tempor libero. Integer ipsum ante, dictum in massa et, fermentum tincidunt mauris. Aenean dictum sed turpis at cursus. Maecenas lacinia gravida lorem vitae vestibulum. Nulla sed libero dignissim quam bibendum scelerisque. Praesent neque sem, mollis et nunc eget, lacinia sagittis purus. Praesent euismod ornare nibh, blandit efficitur enim feugiat non. Aenean condimentum ligula vitae augue faucibus, eu aliquet dui commodo. Sed sit amet placerat tellus, sit amet viverra nibh. Ut nibh augue, sagittis at facilisis a, scelerisque mattis ligula. Etiam ut arcu elit. Praesent vitae metus vulputate, porta neque sit amet, imperdiet turpis. Nulla in lectus tincidunt, finibus ex eu, vestibulum orci. In hac habitasse platea dictumst. Maecenas cursus nibh sem. Morbi venenatis odio ex, vel accumsan turpis gravida quis.
    
    Proin eu massa eu sapien hendrerit congue. Nunc id mauris dolor. Suspendisse potenti. In fermentum ex eu tincidunt fermentum. Phasellus maximus sem justo, et maximus leo auctor sagittis. Mauris auctor ut nunc tristique condimentum. Sed ac consectetur odio, eu dapibus nibh. Fusce egestas libero id ultrices vulputate. Cras venenatis quam quis erat tincidunt, et aliquet velit hendrerit. Proin quis dictum lacus. Aenean vehicula nibh felis, ut dignissim odio condimentum nec. Mauris consequat lacinia nulla, ac fermentum odio. Curabitur eu libero blandit, facilisis nisl at, maximus leo. Vivamus sagittis purus id velit mattis, et aliquam ante posuere. In sollicitudin condimentum magna, nec consequat purus imperdiet id. In fringilla magna ut lectus finibus, ac malesuada enim porta.
    
    Curabitur est nibh, mollis ut arcu eget, finibus eleifend magna. Praesent eget auctor mi. Proin non mi in dui accumsan blandit. Integer fermentum vitae lacus eget commodo. Cras pharetra luctus mattis. Pellentesque vel leo eget lorem dapibus semper eu ut nunc. Maecenas sit amet est non libero porta lacinia sit amet dictum lacus. Morbi vulputate. ";


// SHORT WORD 
    println!("SHORT WORD NO MATCH");  

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_vec.contains(&word);
        }
    }
    let elapsed = now.elapsed();
    println!("Vector search, shord word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re.is_match(word);
        }
    }
    let elapsed = now.elapsed();
    println!("Regex search, shord word Elapsed: {:.2?}", elapsed);  
    println!(" "); 

// LONG WORD
    println!("LONG WORD"); 

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_vec.contains(&really_long_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Vector search, long word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re.is_match(really_long_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Regex search, long word Elapsed: {:.2?}", elapsed);  
    println!(" "); 

// MATCHING WORD
    println!("MATCHING WORD"); 

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_vec.contains(&matching_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Vector search, matching word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re.is_match(matching_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Regex search, matching word Elapsed: {:.2?}", elapsed);  
    println!(" "); 

// MATCHING EARLY WORD
    println!("MATCHING EARLY WORD"); 
    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_vec.contains(&matching_early_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Vector search, matching early word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re.is_match(matching_early_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Regex search, matching early word Elapsed: {:.2?}", elapsed);  
    println!(" "); 


// MATCHING WORD _ SINGLE WORD
    println!("SINGLE WORD MATCHING"); 
    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_word.eq(&matching_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Single &str search, matching word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re_short.is_match(matching_word);
        }
    }
    let elapsed = now.elapsed();
    println!("Single Regex search, matching word Elapsed: {:.2?}", elapsed);  
    println!(" "); 


// NOT MATCHING WORD _ SINGLE WORD
    println!("SINGLE WORD NOT MATCHING"); 
    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_word.eq(&word);
        }
    }
    let elapsed = now.elapsed();
    println!("Single &str search, not matching word Elapsed: {:.2?}", elapsed);    

    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            let _ = tlm_re_short.is_match(word);
        }
    }
    let elapsed = now.elapsed();
    println!("Single Regex search, not matching word Elapsed: {:.2?}", elapsed);  
    println!(" "); 


// PRIMITIVE SEARCH
    println!("PRIMITIVE SEARCH NO MATCH (for + .eq)"); 
    let now = Instant::now();
    {
        for _ in 0..NUM_ITER {
            for w in tlm_vec {
                let _ = tlm_word.eq(w);
            } 
            
        }
    }
    let elapsed = now.elapsed();
    println!("Primitive search no match, Elapsed: {:.2?}", elapsed);    
    println!(" "); 


}

