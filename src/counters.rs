use std::iter;
use macroquad::time::get_time;

enum Span {
  Started(f64),
  Ended(f64),
}

#[derive(Default)]
pub struct Counters {
  spans: Vec<(usize, String, Span)>,
  frame_start: f64,
  open_spans: usize,
  max_span_name_size: usize,
}

impl Counters {
  pub fn begin(&mut self, name: &str) -> usize {
    let index = self.spans.len();
    self.spans.push((self.open_spans, name.to_string(), Span::Started(get_time())));
    self.open_spans += 1;
    self.max_span_name_size = self.max_span_name_size.max(name.len());
    index
  }

  pub fn end(&mut self, index: usize) {
    let Some((_, _, span)) = self.spans.get_mut(index) else {
      panic!("cannot end a non-existent span");
    };
    let elapsed = match span {
      Span::Started(start) => {
        get_time() - *start
      },
      Span::Ended(_) => panic!("cannot end a span twice"),
    };
    *span = Span::Ended(elapsed);
    self.open_spans -= 1;
  }

  pub fn reset(&mut self) {
    self.spans.clear();
    self.frame_start = get_time();
    self.open_spans = 0;
    self.max_span_name_size = 0;
  }

	pub fn stats(&self) -> String {
    let width = self.max_span_name_size;
  	let mut output = String::new();
		for (parent_count, name, span) in self.spans.iter() {
      let Span::Ended(elapsed) = span else {
        continue;
      };
      let elapsed = (elapsed * 1000.).floor() as i64;
      let indent = iter::repeat_n(' ', 2 * parent_count)
        .collect::<String>();
  		output.push_str(&format!("{indent}{name:width$} {elapsed}ms\n"));
		}
  	let total_elapsed = get_time() - self.frame_start;
    let total_elapsed = (total_elapsed * 1000.).floor() as i64;
		output.push_str(&format!("TOTAL: {total_elapsed}ms\n"));
		output
	}
}
