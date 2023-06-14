# ESP32C3 WS2812 Animation Display

Uses an 8x8 matrix of WS2812b addresssable led (left to right, top to bottom) to
display animation.  Converts the images in `resource/img` to byte arrays at build
time, then feeds those in, frame by frame, to `ws2812-esp32-rmt-driver`.  Enabled
a button (GPIO2) to flip between sequences.  WS2812 data line should connect to
pin GPIO19.

## Build and flash:
`cargo build --release && cargo espflash --release`
