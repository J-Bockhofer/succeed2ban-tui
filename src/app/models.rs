use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IOProducer {
  Journal,
  Log
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IOMessage {
  SingleLine(String, IOProducer),
  MultiLine(Vec<String>, IOProducer),
}

impl IOMessage {
  pub fn destructure(&self, sep: &str) -> (String, IOProducer) {
    let prod: IOProducer;
    let catmsg: String;
    match self.clone() {
      IOMessage::SingleLine(x, p) => {
        prod = p;
        catmsg = x},
      IOMessage::MultiLine(vx, p) => {
        prod = p;
        catmsg = vx.join(sep)
      },
    }
    return (catmsg, prod)
  }
}
