#![no_std]
#![feature(start)]

extern crate alloc;
use ogc::prelude::*;
use ogc::prelude::ogc_sys as sys;
use ogc::mem_cached_to_uncached;

const FIFO_SIZE: usize = 256 * 1024;

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

    // Other
    let background = sys::_gx_color { r: 0, g: 0, b: 0, a: 0 };
    let mut config = Video::get_preferred_mode();

    unsafe {
        let fifo_buffer = mem_cached_to_uncached!(libc::memalign(32, FIFO_SIZE));
        libc::memset(fifo_buffer, 0, FIFO_SIZE);
        sys::GX_Init(fifo_buffer, FIFO_SIZE as u32);
        sys::GX_SetCopyClear(background, 0x00FFFFFF);
        sys::GX_SetViewport(0.0, 0.0, config.framebuffer_width as f32, config.embed_framebuffer_height as f32, 0.0, 1.0);
        sys::GX_SetDispCopyYScale((config.extern_framebuffer_height as f32) / (config.framebuffer_width as f32));
        sys::GX_SetScissor(0, 0, config.framebuffer_width.into(), config.embed_framebuffer_height.into());
        sys::GX_SetDispCopySrc(0, 0, config.framebuffer_width, config.embed_framebuffer_height);
        sys::GX_SetDispCopyDst(config.framebuffer_width, config.extern_framebuffer_height);
        sys::GX_SetCopyFilter(config.anti_aliasing, &mut config.sample_pattern[0], sys::GX_TRUE as u8, &mut config.v_filter[0]);
        sys::GX_SetFieldMode(config.field_rendering, sys::GX_ENABLE as u8);
    }

    loop {
        // Wait for the next frame.
        Video::wait_vsync();
    }
}
