pub type Frame = Vec<Vec<String>>;

pub fn new_frame(width: usize, height: usize) -> Frame {
  let mut cols = Vec::with_capacity(width);
  for _ in 0..width {
    let mut col = Vec::with_capacity(height);
    for _ in 0..height {
      col.push(" ".to_owned());
    }
    cols.push(col);
  }
  cols
}
