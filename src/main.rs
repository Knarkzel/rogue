#![no_std]
#![feature(start)]

mod display;

use display::Display;

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

use ogc::prelude::*;

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut video = Video::init();
    Video::configure(Video::get_preferred_mode().into());
    Video::set_next_framebuffer(video.framebuffer);
    Video::set_black(true);
    Video::flush();
    Video::wait_vsync();
    Video::set_black(false);

    let mut wii_display = Display::new(256 * 1024);
    wii_display.setup(&mut video.render_config);

    let fb_width = video.render_config.framebuffer_width as _;
    let emb_height = video.render_config.embed_framebuffer_height as _;

    loop {
        Gx::set_viewport(0.0, 0.0, fb_width, emb_height, 0.0, 0.0);

        Rectangle::new(Point::new(250, 100), Size::new(150, 100))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb888::WHITE)
                    .build(),
            )
            .draw(&mut wii_display)
            .unwrap();
        wii_display.flush(video.framebuffer);

        Video::set_next_framebuffer(video.framebuffer);
        Video::flush();
        Video::wait_vsync();
    }
}
