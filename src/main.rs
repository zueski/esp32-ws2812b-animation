use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use smart_leds_trait::{SmartLedsWrite};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrb24;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGB8};

use std::thread;
use std::ptr;
use std::iter;
use smart_leds_trait::RGB;

use std::sync::{Arc, Mutex};

mod images;
mod animation;

static mut EVENT_QUEUE: Option<esp_idf_sys::QueueHandle_t> = None;

#[link_section = ".iram0.text"]
unsafe extern "C" fn button_interrupt(_: *mut core::ffi::c_void) {
	esp_idf_sys::xQueueGiveFromISR(EVENT_QUEUE.unwrap(), std::ptr::null_mut());
}

macro_rules! array2d_to_vec{
	($a:expr) => {
		{
			let mut v = Vec::with_capacity($a.len());
			for f in $a {
				v.push(f.to_vec());
			}
			v
		}
	}
}

fn main() {
	// It is necessary to call this function once. Otherwise some patches to the runtime
	// implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
	esp_idf_sys::link_patches();
	// Bind the log crate to the ESP Logging facilities
	esp_idf_svc::log::EspLogger::initialize_default();
	
	//let mut led = WS2812RMT::new(peripherals.pins.gpio2, peripherals.rmt.channel0)?;
	const BUTTON_GPIO_NUM: i32 = 2;
	
	// gpio19
	const LED_PIN_GPIO_NUM: u32 = 19;
	
	// Configures the button
	let io_conf = esp_idf_sys::gpio_config_t {
		pin_bit_mask: 1 << BUTTON_GPIO_NUM,
		mode: esp_idf_sys::gpio_mode_t_GPIO_MODE_INPUT,
		pull_up_en: true.into(),
		pull_down_en: false.into(),
		intr_type: esp_idf_sys::gpio_int_type_t_GPIO_INTR_POSEDGE, // Positive edge trigger = button down
	};
	
	// Queue configurations
	const QUEUE_TYPE_BASE: u8 = 0;
	const ITEM_SIZE: u32 = 0; // We're not posting any actual data, just notifying
	const QUEUE_SIZE: u32 = 1;
	
	// image setup
	let sequences: [animation::AnimationSequence; 2] = [
		animation::AnimationSequence::load(
			"Thor".to_string(),
			images::THOR_COUNT,
			130,
			images::THOR_X_LEN,
			images::THOR_Y_LEN,
			array2d_to_vec!(&images::THOR_FRAMES),
		),
		animation::AnimationSequence::load(
			"Rainbow".to_string(), 
			images::RAINBOW_COUNT, 
			40, 
			images::RAINBOW_X_LEN, 
			images::RAINBOW_Y_LEN, 
			array2d_to_vec!(&images::RAINBOW_FRAMES)
		),
	];
	let sequences_len = sequences.len();
	let animation_state: Arc<Mutex<animation::AnimationState>> = Arc::new(Mutex::new(
		animation::AnimationState {
			seq_index:0,
			frame_index:0,
		}
	));

	unsafe {
		// Writes the button configuration to the registers
		let _ = esp_idf_sys::esp!(esp_idf_sys::gpio_config(&io_conf));
		// Installs the generic GPIO interrupt handler
		let _ = esp_idf_sys::esp!(esp_idf_sys::gpio_install_isr_service(esp_idf_sys::ESP_INTR_FLAG_IRAM as i32));
		// Instantiates the event queue
		EVENT_QUEUE = Some(esp_idf_sys::xQueueGenericCreate(QUEUE_SIZE, ITEM_SIZE, QUEUE_TYPE_BASE));
		// Registers our function with the generic GPIO interrupt handler we installed earlier.
		let _ = esp_idf_sys::esp!(esp_idf_sys::gpio_isr_handler_add(
			BUTTON_GPIO_NUM,
			Some(button_interrupt),
			std::ptr::null_mut()
		));
	}
	let animation_state_sender = Arc::clone(&animation_state);
	thread::spawn(move || {
		let mut led = LedPixelEsp32Rmt::<RGB8, LedPixelColorGrb24>::new(1, 8).unwrap();
		loop {
			let res;
			unsafe {
				// Maximum delay
				const QUEUE_WAIT_TICKS: u32 = 1000;
				// Reads the event item out of the queue
				res = esp_idf_sys::xQueueReceive(EVENT_QUEUE.unwrap(), ptr::null_mut(), QUEUE_WAIT_TICKS);
			}
			// If the event has the value 0, nothing happens. if it has a different value, the button was pressed.
			match res {
				1 => {
					// Generates random rgb values and sets them in the led.
					random_light(&mut led);
					if let Ok(mut a) = animation_state_sender.lock() {
						a.inc_seq(sequences_len);
					}
				}
				_ => {}
			};
		}
	});
	
	let mut ws2812 = LedPixelEsp32Rmt::<RGB8, LedPixelColorGrb24>::new(0, LED_PIN_GPIO_NUM).unwrap();
	loop {
		let mut delay_ms = 20;
		if let Ok(mut a) = animation_state.lock() {
			let seq_index = a.seq_index;
			let frame_index = a.frame_index;
			delay_ms = sequences[seq_index].delay_ms;
			ws2812.write(sequences[seq_index].get_image_iter(frame_index)).unwrap();
			a.inc_frame(sequences[seq_index].frame_count);
		}
		sleep(Duration::from_millis(delay_ms));
	}
}


#[allow(unused)]
fn random_light<CDev: std::convert::From<smart_leds_trait::RGB<u8>> + ws2812_esp32_rmt_driver::driver::color::LedPixelColor>(led: &mut LedPixelEsp32Rmt<RGB<u8>, CDev>) {
	let mut color = RGB8::new(0, 0, 0);
	unsafe {
		let r = esp_idf_sys::esp_random() as u8;
		let g = esp_idf_sys::esp_random() as u8;
		let b = esp_idf_sys::esp_random() as u8;
		color = RGB8::new(r, g, b);
	}
	led.write(iter::once(color)).unwrap();
}