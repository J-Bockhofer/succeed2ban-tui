use std::process::{Command, Stdio};
use std::io::Read;






#[tokio::main]
async fn main() {


  let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

  let _joinhandle = tokio::spawn(async move {

    // closure needs a sender to send to new string and a receiver to terminate the process

    let argus = vec!["-n", "1", "-f", "-u", "ssh"];
    let mut command = Command::new("journalctl");
    command
        .args(argus)
        .stdout(Stdio::piped());

    if let Ok(child) = command.spawn() {
        if let Some(mut out) = child.stdout {
          let mut pre_line: Vec<u8> = vec![];
            loop {            
              let mut chars: [u8; 1] = [0; 1];
              out.read(&mut chars).expect("didn't work");
              
              // check if we encounter a newline character, if yes send chars 
              if chars[0] == b'\n' || chars[0] == b'\r' {
                // terminate string
                //println!("Newline detected");
                //println!("{}", str::from_utf8(&pre_line).unwrap());
                let line = String::from_utf8(pre_line).unwrap();
                action_tx.send(line).unwrap_or_else(|err| {
                  println!("Send Error: {}", err);
                });
                pre_line = vec![];
              }
              else {
                // add char to line
                //print!("Pushing {}", str::from_utf8(&chars).unwrap());
                pre_line.push(chars[0]);
                //println!("{}", str::from_utf8(&pre_line).unwrap());
              }      
            }
            
        }
    } else {
      println!("Process failed to start");
    }

  });

  let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(100));
  loop {
     let msg = action_rx.try_recv().unwrap_or_default();
     if !msg.is_empty() {
      println!("Received {}", msg);
     }
     tick_interval.tick().await;
  }

}

