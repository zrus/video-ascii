use ::std::ops::Range;

#[macro_export]
macro_rules! min {
    ( $( $x:expr ),* ) => {
        {
            let mut min = u16::MAX;
            $(
                min = std::cmp::min(min, $x);
            )*
            min
        }
    }
}

#[macro_export]
macro_rules! max {
    ( $( $x:expr ),* ) => {
        {
            let mut max = u16::MIN;
            $(
                max = std::cmp::max(max, $x);
            )*
            max
        }
    }
}

pub fn map(value: u16, src_range: Range<u16>, dest_range: Range<u16>) -> u16 {
  if value < src_range.start || value > src_range.end {
    return 0u16;
  }

  let real_src_range = src_range.end - src_range.start;
  let real_value = value - src_range.start;
  let percentage_of_value = real_value as f32 / real_src_range as f32;

  let real_dest_range = dest_range.end - dest_range.start;
  let dest_value = percentage_of_value * real_dest_range as f32;
  dest_value as u16 + dest_range.start
}
