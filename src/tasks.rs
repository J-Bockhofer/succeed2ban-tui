use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};

use tokio::{
    sync::mpsc::UnboundedSender
  };

use crate::action::Action;


pub async fn notify_change(path: &str, _event_tx:UnboundedSender<Action>) -> Result<()> {
    // get file
    
    let mut pos = std::fs::metadata(path)?.len();
    // get pos to end of file
    let mut f = std::fs::File::open(path)?;
    

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // watch
    for res in rx {
        match res {
            Ok(_event) => {
                // ignore any event that didn't change the pos
                if f.metadata()?.len() == pos {
                    continue;
                }
                if f.metadata()?.len() == 0 {
                    continue;
                }

                // read from pos to end of file
                f.seek(std::io::SeekFrom::Start(pos))?;

                // update post to end of file
                pos = f.metadata()?.len();
                let reader = BufReader::new(&f);
                let mut msgs = vec!["".to_string()];

                for line in reader.lines() {
                    let cline = line.unwrap();
                    let nullchar = cline.chars().nth(cline.chars().count());
                    if !nullchar.is_none()
                    {
                        let newchar = nullchar.unwrap();
                        if newchar.is_whitespace()
                        {
                            pos -= 1;
                        }

                    }

                    //println!("> {:?}", cline);
                    msgs.push(cline.clone());                   
                }
                let addedlines = msgs.into_iter().collect::<Vec<String>>().join("\n");
                //println!("> {}", addedlines);
                _event_tx.send(Action::IONotify(String::from(addedlines))).unwrap();
            }
            Err(error) => println!("{error:?}"),
        }
    }

    Ok(())
}

