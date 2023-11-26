use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};

use tokio::{
    sync::mpsc::UnboundedSender
  };

use crate::action::Action;

use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;

// pass another receiver for the cancellation token?
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
                let addedlines = msgs.into_iter().collect::<Vec<String>>().join("++++");
                //println!("> {}", addedlines);
                _event_tx.send(Action::IONotify(addedlines)).unwrap();
            }
            Err(error) => println!("{error:?}"),
        }
    }

    Ok(())
}



pub async fn follow_file(path: &str, action_tx: tokio::sync::mpsc::UnboundedSender<Action>) {
    let mut file = tokio::fs::File::open(path).await.unwrap();
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
    let mut contents = vec![];
    let mut position = 0; // let mut pos = std::fs::metadata(path)?.len();
    
    //let filelength: usize = tokio::fs::metadata(path).await.unwrap().len().try_into().unwrap();

    let mut iternum = 0;
  
    loop {
  
        iternum += 1;
        contents.truncate(0);
        let _ = file.seek(tokio::io::SeekFrom::Start(position as u64)).await;
        position += file.read_to_end(&mut contents).await.unwrap() - 1; // this panics on deletion in file
        
/*         if position < usize::default() {
            position = 0;
        } */

        let thestring = String::from_utf8(contents.clone()).unwrap();
        // do_process(contents)
        //println!("{:?}", String::from_utf8(contents.clone()).unwrap());
        if iternum > 1 {
            if thestring.len() > 1 {
                action_tx.send(Action::IONotify(thestring)).unwrap_or_else(|error| {
                    println!("Notifythread sends Error -> {}", error);
                  });
            }
        }
        interval.tick().await;
    }
  }