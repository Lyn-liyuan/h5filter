mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys;


#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);


    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);


    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! console_log {

    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct GrayColors {
    buffer_ctx: web_sys::CanvasRenderingContext2d,
    video: web_sys::HtmlVideoElement,
    buffer: web_sys::HtmlCanvasElement,
    show: web_sys::HtmlCanvasElement,
    show_ctx: web_sys::CanvasRenderingContext2d,
    video_width: u32,
    video_height: u32,
    show_width: u32,
    show_height: u32,
}

#[wasm_bindgen]
pub fn sethook() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
impl GrayColors {
    pub fn new(vid: &str, cid: &str) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let video = document.get_element_by_id(vid).unwrap();
        let video = video
            .dyn_into::<web_sys::HtmlVideoElement>()
            .map_err(|_| ())
            .unwrap();

        let show = document.get_element_by_id(cid).unwrap();
        let show = show
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let show_ctx = show
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        GrayColors {
            video_width: video.video_width(),
            video_height: video.video_height(),
            show_width: show.width(),
            show_height: show.height(),
            buffer_ctx: context,
            video: video,
            buffer: canvas,
            show: show,
            show_ctx: show_ctx,
        }
    }

    pub fn set_size(&mut self, video_width: u32, video_height: u32) {
        self.show.set_width(video_width);
        self.show.set_height(video_height);
        self.buffer.set_width(video_width);
        self.buffer.set_height(video_height);
        self.video_height = video_height;
        self.video_width = video_width;
        self.show_height = video_height;
        self.show_width = video_width;
    }

    fn old_filter(&self, data: &mut Vec<u8>) {
        let len = data.len() / 4;
        for i in 0..len {
            let r = data[i * 4 + 0] as f64;
            let g = data[i * 4 + 1] as f64;
            let b = data[i * 4 + 2] as f64;

            let r_color = r;
            let g_color = g;
            let b_color = (b.sqrt() * 8f64) as f32;

            data[i * 4 + 0] = if r_color > 255f64 {
                255u8
            } else {
                r_color as u8
            };
            data[i * 4 + 1] = if g_color > 255f64 {
                255u8
            } else {
                g_color as u8
            };
            data[i * 4 + 2] = if b_color > 255f32 {
                255u8
            } else {
                b_color as u8
            };
            data[i * 4 + 3] = data[i * 4 + 3]
        }
    }

    fn comic_filter(&self, data: &mut Vec<u8>) {
        let len = data.len() / 4;
        for i in 0..len {
            let br = data[i * 4 + 0] as i32;
            let bg = data[i * 4 + 1] as i32;
            let bb = data[i * 4 + 2] as i32;

            let r_color = (bg - bb + bg + br).abs() * br >> 8;
            let g_color = (bb - bg + bb + br).abs() * br >> 8;
            let b_color = (bb - bg + bb + br).abs() * bg >> 8;

            data[i * 4 + 0] = if r_color > 255 { 255u8 } else { r_color as u8 };
            data[i * 4 + 1] = if g_color > 255 { 255u8 } else { g_color as u8 };
            data[i * 4 + 2] = if b_color > 255 { 255u8 } else { b_color as u8 };
            data[i * 4 + 3] = data[i * 4 + 3]
        }
    }

    fn soft_filter(&self, data: &mut Vec<u8>, opacity: i32) {
        let len = data.len() / 4;
        for i in 0..len {
            let br = data[i * 4 + 0] as i32;
            let bg = data[i * 4 + 1] as i32;
            let bb = data[i * 4 + 2] as i32;

            let r_color = (br * opacity + br * (255 - opacity)) >> 8;
            let g_color = (bg * opacity + bg * (255 - opacity)) >> 8;
            let b_color = (bb * opacity + bb * (255 - opacity)) >> 8;

            data[i * 4 + 0] = if r_color > 255 { 255u8 } else { r_color as u8 };
            data[i * 4 + 1] = if g_color > 255 { 255u8 } else { g_color as u8 };
            data[i * 4 + 2] = if b_color > 255 { 255u8 } else { b_color as u8 };
            data[i * 4 + 3] = data[i * 4 + 3]
        }
    }

    pub fn rander(&self) {
        match self
            .buffer_ctx
            .draw_image_with_html_video_element(&self.video, 0f64, 0f64)
        {
            Ok(_) => {}
            Err(e) => {
                console_log!("draw_image_with_html_video_element {:?}", e);
            }
        }

        let frame: web_sys::ImageData = match self.buffer_ctx.get_image_data(
            0 as f64,
            0 as f64,
            self.show_width as f64,
            self.show_height as f64,
        ) {
            Ok(value) => value,
            Err(e) => {
                console_log!("draw_image_with_html_video_element {:?}", e);
                panic!("put_image_data")
            }
        };
        let mut data = frame.data();
        self.old_filter(&mut data);
        let output = match web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&mut data),
            self.show_width,
            self.show_height,
        ) {
            Ok(value) => value,
            Err(e) => {
                console_log!("new_with_u8_clamped_array_and_sh {:?}", e);
                panic!("new_with_u8_clamped_array_and_sh")
            }
        };
        match self.show_ctx.put_image_data(&output, 0f64, 0f64) {
            Ok(_) => {}
            Err(e) => {
                console_log!("put_image_data {:?}", e);
                panic!("put_image_data")
            }
        };
    }
}
