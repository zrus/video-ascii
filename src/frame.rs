pub type Frame = Vec<Vec<String>>;

pub fn new_frame(num_cols: usize, num_rows: usize) -> Frame {
  let mut rows = Vec::with_capacity(num_rows);
  for _ in 0..num_rows {
    let mut row = Vec::with_capacity(num_cols);
    for _ in 0..num_cols {
      row.push("_".to_owned());
    }
    rows.push(row);
  }
  rows
}

pub trait Drawable {
  fn draw(&self, frame: &mut Frame);
}
