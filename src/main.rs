#![no_std]
#![feature(start)]

extern crate alloc;
use ogc::prelude::*;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // Initialise the video system
    let video = Video::init();

    // Initialise the console, required for print.
    Console::init(&video);

    // Set up the video registers with the chosen mode.
    Video::configure(video.render_config.into());

    // Tell the video hardware where our display memory is.
    Video::set_next_framebuffer(video.framebuffer);

    // Make the display visible.
    Video::set_black(false);

    // Flush the video register changes to the hardware.
    Video::flush();

    // Wait for Video setup to complete.
    Video::wait_vsync();

    // Input
    Wpad::init();

    loop {
        // Wait for the next frame.
        Video::wait_vsync();

        Wpad::scan_pads();

        if Wpad::buttons_down(Remote::One) == WpadButton::A {
            println!("bruh");
        }

        // println!("L: {}, R: {}", pad.trigger_l(Controller::One), pad.trigger_r(Controller::One));
    }
}
