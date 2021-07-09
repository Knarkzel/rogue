#![no_std]
#![feature(start)]
#![feature(default_free_fn)]

extern crate alloc;
use core::{convert::TryInto, ffi::c_void};

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb888,
    prelude::{IntoStorage, OriginDimensions, Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    Drawable, Pixel,
};

use ogc::{
    ffi::{
        guMtx44Identity, guOrtho, GX_Color1u32, GX_End, GX_LoadProjectionMtx, GX_PokeARGB,
        GX_Position3f32, GX_SetAlphaCompare, GX_SetClipMode, GX_ALWAYS, GX_AOP_AND,
        GX_BL_INVSRCALPHA, GX_BL_SRCALPHA, GX_BM_BLEND, GX_CLIP_ENABLE, GX_CLR_RGBA, GX_COLOR0A0,
        GX_CULL_NONE, GX_DIRECT, GX_F32, GX_GM_1_0, GX_GREATER, GX_LEQUAL, GX_LO_CLEAR,
        GX_MAX_Z24, GX_NONE, GX_ORTHOGRAPHIC, GX_PASSCLR, GX_PF_RGB8_Z24, GX_PNMTX0, GX_POS_XYZ,
        GX_QUADS, GX_RGBA8, GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_TEX_ST, GX_TRUE, GX_VA_CLR0,
        GX_VA_POS, GX_VA_TEX0, GX_VTXFMT0, GX_ZC_LINEAR,
    },
    prelude::*,
};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut video = Video::init();
    Video::configure(Video::get_preferred_mode().into());
    Video::set_next_framebuffer(video.framebuffer);
    Video::set_black(true);
    Video::flush();
    Video::wait_vsync();

    let mut wii_display = Display::new(256 * 1024);
    wii_display.setup(&mut video.render_config);
    Video::set_black(false);

    let width = video.render_config.framebuffer_width as f32;
    let height = video.render_config.embed_framebuffer_height as f32;

    loop {
        Gx::set_viewport(0.0, 0.0, width, height, 0.0, 0.0);

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

pub struct Display;

impl Display {
    pub fn new(fifo_size: usize) -> Self {
        let buffer = gp_fifo(fifo_size);
        Gx::init(buffer, fifo_size as u32);
        Self
    }

    pub fn flush(&self, framebuffer: *mut c_void) {
        Gx::draw_done();
        Gx::set_z_mode(GX_TRUE as _, GX_LEQUAL as _, GX_TRUE as _);
        Gx::copy_disp(framebuffer, GX_TRUE as _);
    }

    pub fn setup(&self, rc: &mut RenderConfig) {
        let mut ident: Mtx34 = [[0.0; 4]; 3];

        let color = (0, 0, 0, 0);
        Gx::set_copy_clear(color, GX_MAX_Z24);
        Gx::set_pixel_fmt(GX_PF_RGB8_Z24 as _, GX_ZC_LINEAR as _);

        let (width, height) = (rc.framebuffer_width, rc.embed_framebuffer_height);
        Gx::set_viewport(0.0, 0.0, width as _, height as _, 0.0, 0.0);

        let (emb, ext) = (rc.embed_framebuffer_height, rc.extern_framebuffer_height);
        let y_scale = Gx::get_y_scale_factor(emb, ext);
        let ext_fb_height = Gx::set_disp_copy_y_scale(y_scale);

        // GX_FALSE = 0, GX_TRUE = 1
        let half_aspect_ratio = (rc.vi_height == 2 * rc.extern_framebuffer_height) as u32;

        Gx::set_disp_copy_src(0, 0, rc.framebuffer_width, rc.embed_framebuffer_height);
        Gx::set_disp_copy_dst(rc.framebuffer_width, ext_fb_height as _);

        Gx::set_copy_filter(
            rc.anti_aliasing,
            rc.sample_pattern,
            GX_TRUE as _,
            rc.v_filter,
        );

        Gx::set_field_mode(rc.field_rendering, half_aspect_ratio as _);
        Gx::set_disp_copy_gamma(GX_GM_1_0 as _);

        // Clear VTX
        Gx::clear_vtx_desc();
        Gx::inv_vtx_cache();
        Gx::invalidate_tex_all();

        Gx::set_vtx_desc(GX_VA_TEX0 as _, GX_NONE as _);
        Gx::set_vtx_desc(GX_VA_POS as _, GX_DIRECT as _);
        Gx::set_vtx_desc(GX_VA_CLR0 as _, GX_DIRECT as _);

        Gx::set_vtx_attr_fmt(
            GX_VTXFMT0 as _,
            GX_VA_POS as _,
            GX_POS_XYZ as _,
            GX_F32 as _,
            0,
        );
        Gx::set_vtx_attr_fmt(GX_VTXFMT0 as _, GX_VA_TEX0, GX_TEX_ST as _, GX_F32 as _, 0);
        Gx::set_vtx_attr_fmt(
            GX_VTXFMT0 as _,
            GX_VA_CLR0,
            GX_CLR_RGBA as _,
            GX_RGBA8 as _,
            0,
        );
        Gx::set_z_mode(GX_TRUE as _, GX_LEQUAL as _, GX_TRUE as _);

        Gx::set_num_chans(1);
        Gx::set_num_tex_gens(1);
        Gx::set_tev_op(GX_TEVSTAGE0 as _, GX_PASSCLR as _);
        Gx::set_tev_order(
            GX_TEVSTAGE0 as _,
            GX_TEXCOORD0 as _,
            GX_TEXMAP0 as _,
            GX_COLOR0A0 as _,
        );

        // THIS SAFE FUNCTION DOES NOT WORK!
        // HAVE TO USE UNSAFE INSTEAD.
        // Gu::mtx_identity(ident);
        unsafe { guMtx44Identity(&mut ident as *mut _) };

        // THIS FUNCTION WORKS BUT IT PROBABLY DOESN'T APPLY THE CORRECT EFFECT
        // GRANTED THE ABOVE SAFE FUNCTION DOES NOT WORK.
        Gu::mtx_trans_apply(ident, ident, 0.0, 0.0, -100.0);
        // c_guMtxTransApply(&mut ident as *mut _, &mut ident as *mut _, 0.0, 0.0, -100.0);

        Gx::load_pos_mtx_imm(ident, GX_PNMTX0 as _);
        // GX_LoadPosMtxImm(&mut ident as *mut _, GX_PNMTX0 as _);

        // Gu::ortho(ident, 0.0, rc.embed_framebuffer_height as f32, 0.0, rc.framebuffer_width as f32, 0.0, 1000.0);
        unsafe {
            guOrtho(
                &mut ident as *mut _,
                0.0,
                rc.embed_framebuffer_height as f32,
                0.0,
                rc.framebuffer_width as f32,
                0.0,
                1000.0,
            );
        }

        // Gx::load_projection_mtx(ident, GX_ORTHOGRAPHIC as _);
        unsafe {
            GX_LoadProjectionMtx(&mut ident as *mut _, GX_ORTHOGRAPHIC as _);
        }

        Gx::set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        Gx::set_blend_mode(
            GX_BM_BLEND as _,
            GX_BL_SRCALPHA as _,
            GX_BL_INVSRCALPHA as _,
            GX_LO_CLEAR as _,
        );
        Gx::set_alpha_update(GX_TRUE as _);

        // Gx::set_alpha_compare(GX_GREATER as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
        unsafe {
            GX_SetAlphaCompare(GX_GREATER as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
        }

        Gx::set_color_update(GX_TRUE as _);
        Gx::set_cull_mode(GX_CULL_NONE as _);

        // Gx::set_clip_mode(GX_CLIP_ENABLE as _);
        unsafe {
            GX_SetClipMode(GX_CLIP_ENABLE as _);
        }

        Gx::set_scissor(
            0,
            0,
            rc.framebuffer_width.into(),
            rc.embed_framebuffer_height.into(),
        );
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=639, y @ 0..=527)) = coord.try_into() {
                let poke_x: u32 = x;
                let poke_y: u32 = y;
                unsafe {
                    GX_PokeARGB(
                        poke_x as u16,
                        poke_y as u16,
                        ogc::ffi::GXColor {
                            r: color.r(),
                            g: color.g(),
                            b: color.b(),
                            a: 255,
                        },
                    )
                };
            }
        }

        Ok(())
    }

    //Implement fill_contigous using texture stuffs

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        Gx::begin(GX_QUADS as _, GX_VTXFMT0 as _, 4);

        unsafe {
            GX_Position3f32(area.top_left.x as _, area.top_left.y as _, 0.0);
            GX_Color1u32(color.into_storage());
            GX_Position3f32(
                area.bottom_right().unwrap().x as _,
                area.top_left.y as _,
                0.0,
            );
            GX_Color1u32(color.into_storage());
            GX_Position3f32(
                area.bottom_right().unwrap().x as _,
                area.bottom_right().unwrap().y as _,
                0.0,
            );
            GX_Color1u32(color.into_storage());
            GX_Position3f32(
                area.top_left.x as _,
                area.bottom_right().unwrap().y as _,
                0.0,
            );
            GX_Color1u32(color.into_storage());
            GX_End();
        }

        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(640, 528)
    }
}
