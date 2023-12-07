use ratatui::widgets::ListState;


#[derive(Default, Clone)]
pub struct Animation<T> {
  pub state: ListState,
  pub keyframes: Vec<T>,
}

impl<T> Animation<T> {
  pub fn with_items(keyframes: Vec<T>) -> Animation<T> {
      Animation {
          state: ListState::default(),
          keyframes,
      }
  }

  pub fn next(&mut self) {
    if self.keyframes.is_empty() {return;}
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.keyframes.len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      //println!("next Item: {i}");
      self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
    if self.keyframes.is_empty() {return;}
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.keyframes.len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

}