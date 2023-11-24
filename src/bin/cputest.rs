
use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;


#[tokio::main]
async fn main() {
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();

    // try make filereader here?
    let action_tx2: tokio::sync::mpsc::UnboundedSender<String> = action_tx.clone();
    let action_tx3 = action_tx.clone();
    
    let mut iternum: usize = 0;
    
    let _filewatcher = tokio::task::spawn(async move {
      tokio::spawn(async move  {
        let path = String::from("/home/projects/ratui/text.txt"); // easy test "/home/projects/ratui/text.txt" // /var/log/fail2ban.log
        println!("Notifythread hello");

        let _aw = follow_file(path.as_str(), action_tx2).await;

/*         let _resp = notify_change(&path, action_tx2).await.unwrap_or_else(|err| {
          println!("Error {}", err);
        }); */
      });
    });

    let _handle = tokio::spawn(async move  {
      println!("Iterthread hello");
      loop {
        iternum += 1;
        //println!("Iteration {}", iternum);
        std::thread::sleep(std::time::Duration::from_secs(1));
        action_tx3.send(format!("{}", iternum)).unwrap_or_else(|error| {
          println!("Iterthread sends Error -> {}", error);
        });
      }
    });

    let _recv_handle = tokio::spawn(async move  {
      println!("Recv hello");
      let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
      loop {

        let action = action_rx.try_recv().unwrap_or(String::from("Nothing received"));

        match action.as_str() {
          "Nothing received" => {},
          x => {
            println!(" {} ", x);
          },
        }

        //println!(" {} ", action);
        interval.tick().await;
      }
    }).await.unwrap();



/*     loop {
        println!("Receiver Loop");

      } */
        
      //let (r1, r2) = (filewatcher.await.unwrap(), _handle.await.unwrap());
      //println!("filewatcher {}", r1.unwrap().status());
      //println!("iterthread {}", r2.unwrap().status());
    
    
}

// I/O Reader task behind
pub async fn notify_change(path: &str, action_tx: tokio::sync::mpsc::UnboundedSender<String>) -> Result<()> {
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

                  println!("> {:?}", cline);
                  msgs.push(cline.clone());                   
              }
              let addedlines = msgs.into_iter().collect::<Vec<String>>().join("++++");
              println!("> {}", addedlines);
              action_tx.send(format!("{}", addedlines)).unwrap_or_else(|error| {
                println!("Notifythread sends Error -> {}", error);
              });
              //_event_tx.send(Action::IONotify(addedlines)).unwrap();
          }
          Err(error) => println!("{error:?}"),
      }
  }

  Ok(())
}

pub async fn follow_file(path: &str, action_tx: tokio::sync::mpsc::UnboundedSender<String>) {
  let mut file = tokio::fs::File::open(path).await.unwrap();
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
  let mut contents = vec![];
  let mut position = 0;
  
  let mut iternum = 0;

  loop {

      iternum += 1;
      contents.truncate(0);
      let _ = file.seek(tokio::io::SeekFrom::Start(position as u64)).await;
      position += file.read_to_end(&mut contents).await.unwrap() - 1;
      
      // do_process(contents)
      //println!("{:?}", String::from_utf8(contents.clone()).unwrap());
      if iternum > 1 {
        action_tx.send(String::from_utf8(contents.clone()).unwrap()).unwrap_or_else(|error| {
          println!("Notifythread sends Error -> {}", error);
        });
      }
      interval.tick().await;
  }
}