use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use smart_leds_trait::{SmartLedsWrite};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrb24;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGB8};

mod images;

fn load_frame(bytes: &[u8], x_size: u32, y_size: u32) -> Vec<RGB8> {
	let pixels:usize = bytes.len() / 4;
	let mut frame: Vec<RGB8> = Vec::with_capacity(pixels);
	for y in 0..y_size {
		for x in 0..x_size {
			let xp;
			if y % 2 == 0 {
				xp = x;
			} else {
				xp = x_size - x - 2;
			}
			let in_pos: usize = ((y*x_size + xp)*4).try_into().unwrap();
			&frame.push(RGB8::from((bytes[in_pos], bytes[in_pos+1], bytes[in_pos+2])));
		}
	}
	return frame;
}

fn main() {
	// It is necessary to call this function once. Otherwise some patches to the runtime
	// implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
	esp_idf_sys::link_patches();
	// Bind the log crate to the ESP Logging facilities
	esp_idf_svc::log::EspLogger::initialize_default();
	
	info!("Hello,  world!");
	
	// gpio19
	let led_pin = 19;
	let mut ws2812 = LedPixelEsp32Rmt::<RGB8, LedPixelColorGrb24>::new(0, led_pin).unwrap();
	let mut frame_no = 0;
	let mut frame_buf: Vec<Vec<RGB8>> = Vec::new();
	for n in 0..images::THOR_COUNT {
		frame_buf.push(load_frame(&images::THOR_FRAMES[n], images::THOR_X_LEN, images::THOR_Y_LEN));
	}
	loop {
		//let frame: Vec<RGB8> = frame_buf.get(frame_no).);
		//ws2812.write(frame.clone().into_iter()).unwrap();
		frame_buf.get(frame_no).and_then(|f| Some(ws2812.write(f.clone().into_iter()).unwrap()));
		frame_no = frame_no + 1;
		if frame_no >= images::THOR_COUNT {
			frame_no = 0;
		}
		sleep(Duration::from_millis(200));
	}
}
