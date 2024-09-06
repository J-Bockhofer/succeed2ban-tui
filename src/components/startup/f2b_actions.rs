use crate::{action::Action, database::schema::ip::IP};



pub fn send_ban(ip: IP, ban_symbol: String,  tx: tokio::sync::mpsc::UnboundedSender<Action>) {

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        // check if is banned
        let output = std::process::Command::new("fail2ban-client")
          .arg("set")
          .arg("sshd")
          .arg("banip")
          .arg(&ip.ip)
          // Tell the OS to record the command's output
          .stdout(std::process::Stdio::piped())
          // execute the command, wait for it to complete, then capture the output
          .output()
          // Blow up if the OS was unable to start the program
          .unwrap();

        // extract the raw bytes that we captured and interpret them as a string
        let stdout = String::from_utf8(output.stdout).unwrap();
        if stdout.contains("0") {
          tx.send(Action::Banned(true)).expect("Failed to Ban ...");
          let fetchmsg = format!(" {} Banned IP: {}", ban_symbol, &ip.ip);
          tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Ban IP message failed to send");
        } else {
          let fetchmsg = format!(" {} Banned IP: {}", ban_symbol, &ip.ip);
          tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Ban IP message failed to send");                 
          tx.send(Action::Banned(false)).expect("Failed to Ban ...");
        }
      });

}

pub fn send_unban(ip: IP, unban_symbol: String,  tx: tokio::sync::mpsc::UnboundedSender<Action>) {

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        // check if is banned
        let output = std::process::Command::new("fail2ban-client")
          .arg("set")
          .arg("sshd")
          .arg("unbanip")
          .arg(&ip.ip)
          // Tell the OS to record the command's output
          .stdout(std::process::Stdio::piped())
          // execute the command, wait for it to complete, then capture the output
          .output()
          // Blow up if the OS was unable to start the program
          .unwrap();

        // extract the raw bytes that we captured and interpret them as a string
        let stdout = String::from_utf8(output.stdout).unwrap();
        if stdout.contains("0") {
          tx.send(Action::Unbanned(true)).expect("Failed to Unban !!!");
          let fetchmsg = format!(" {} Unbanned IP: {}", unban_symbol, &ip.ip);
          tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unban IP message failed to send");
        } else {
          let fetchmsg = format!(" {} Unbanned IP: {}", unban_symbol, &ip.ip);
          tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unban IP message failed to send");
          tx.send(Action::Unbanned(false)).expect("Failed to Unban !!!"); // idfkgetit
        }
      });

}
