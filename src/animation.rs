
use ws2812_esp32_rmt_driver::RGB8;
use std::iter::Iterator;

pub struct AnimationState {
	pub seq_index: usize,
	pub frame_index: usize,
}
impl AnimationState {
	pub fn inc_seq(&mut self, seq_max: usize) {
		self.seq_index = self.seq_index + 1;
		if self.seq_index >= seq_max {
			self.seq_index = 0;
		}
	}
	pub fn inc_frame(&mut self, frame_max: usize) {
		self.frame_index = self.frame_index + 1;
		if self.frame_index >= frame_max {
			self.frame_index = 0;
		}
	}
}

pub struct SequenceIter<'a> {
	pos: usize,
	frames: &'a Vec<RGB8>,
}
impl Iterator for SequenceIter<'_> {
	type Item = RGB8;
	fn next(&mut self) -> Option<Self::Item> {
		if self.pos >= self.frames.len() {
			return None;
		}
		let r = Some(self.frames[self.pos]);
		self.pos += 1;
		return r;
	}
}

pub struct AnimationSequence {
	pub name: String,
	pub frame_count: usize,
	pub delay_ms: u64,
	pub x_size: usize,
	pub y_size: usize,
	frames: Vec<Vec<RGB8>>,
}
impl AnimationSequence {
	pub fn load(
		name: String,
		frame_count: usize,
		delay_ms: u64,
		x_size: usize,
		y_size: usize,
		frames: Vec<Vec<u8>>,
	) -> AnimationSequence {
		let mut frame_buf: Vec<Vec<RGB8>> = Vec::new();
		for frame in frames {
			frame_buf.push(AnimationSequence::load_frame(&frame, x_size, y_size));
		}
		return AnimationSequence {
			name: name,
			frame_count: frame_count,
			delay_ms: delay_ms,
			x_size: x_size,
			y_size: y_size,
			frames: frame_buf,
		};
	}
	
	fn load_frame(bytes: &[u8], x_size: usize, y_size: usize) -> Vec<RGB8> {
		let pixels:usize = bytes.len() / 4;
		let mut frame: Vec<RGB8> = Vec::with_capacity(pixels);
		for y in 0..y_size {
			for x in 0..x_size {
	//~ 			let xp;
	//~ 			if y % 2 == 0 {
	//~ 				xp = x;
	//~ 			} else {
	//~ 				xp = x_size - x - 2;
	//~ 			}
				let in_pos: usize = ((y*x_size + x)*4).try_into().unwrap();
				let _ = &frame.push(RGB8::from((
					bytes[in_pos] / 8,
					bytes[in_pos+1] / 8,
					bytes[in_pos+2] / 8,
				)));
			}
		}
		return frame;
	}
	
	pub fn get_image_iter(&self, index: usize) ->  SequenceIter {
		return SequenceIter {
			pos:0,
			frames: &self.frames[index],
		};
	}
}

