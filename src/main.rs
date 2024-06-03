extern crate gfx_device_gl;
extern crate image;
extern crate num_complex;
extern crate piston_window;
extern crate time;
extern crate rayon;

use image::{ImageBuffer, Rgba};
use num_complex::Complex;
use piston_window::{
    clear, Button, Image, MouseButton, MouseCursorEvent, PistonWindow, PressEvent, Texture,
    TextureSettings, WindowSettings,
};
use std::time::{Duration, Instant};
use rayon::prelude::*;

struct MandelbrotSettings {
    width: u32,
    height: u32,
    max_iterations: u32,
    zoom: f64,
    offset_x: f64,
    offset_y: f64,
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Mandelbrot!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let width = 640;
    let height = 480;

    let mut settings = MandelbrotSettings {
        width,
        height,
        max_iterations: 300,
        zoom: 1.,
        offset_x: 0.0,
        offset_y: 0.0,
    };

    let click_timeout = Duration::from_millis(500);
    let mut last_left_click = Instant::now() - click_timeout;
    let mut last_right_click = Instant::now() - click_timeout;
    let mut requires_redraw = true;
    let mut mouse_pos = [0.0, 0.0];
    let zoom_exp = 1.5;

    let mut image: Texture<gfx_device_gl::Resources> =
        unwrap_image_to_texture(generate_mandelbrot_image(&settings), &mut window);

    while let Some(event) = window.next() {
        if let Some(pos) = event.mouse_cursor_args() {
            mouse_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            let [x, y] = mouse_pos;
                let xi = (x as f64 - width as f64 / 2.) / width as f64 * 4.0 / settings.zoom
                    + settings.offset_x;
                let yi = (y as f64 - height as f64 / 2.) / height as f64 * 4.0 / settings.zoom
                    + settings.offset_y;
            if is_double_click(&mut last_left_click, click_timeout) {
                settings.offset_x = xi;
                settings.offset_y = yi;
                settings.zoom = settings.zoom.clone() * zoom_exp;
                requires_redraw = true;
            }
        }

        if let Some(Button::Mouse(MouseButton::Right)) = event.press_args() {
            if is_double_click(&mut last_right_click, click_timeout) {
                let [x, y] = mouse_pos;
                let xi = (x as f64 - width as f64 / 2.) / width as f64 * 4.0 / settings.zoom
                    + settings.offset_x;
                let yi = (y as f64 - height as f64 / 2.) / height as f64 * 4.0 / settings.zoom
                    + settings.offset_y;
                settings.offset_x = xi;
                settings.offset_y = yi;
                settings.zoom = settings.zoom.clone() / zoom_exp;
                requires_redraw = true;
            }
        }

        if requires_redraw {
            let img = generate_mandelbrot_image(&settings);
            image = unwrap_image_to_texture(img, &mut window);
            requires_redraw = false;
        }

        window.draw_2d(&event, |context, graphics, _| {
            clear([1.0; 4], graphics);
            Image::new().draw(&image, &Default::default(), context.transform, graphics);
        });
    }
}

fn is_double_click(last_click: &mut Instant, click_timeout: Duration) -> bool {
    let now = Instant::now();
    let is_double_click = now - *last_click < click_timeout;
    *last_click = now;
    is_double_click
}

fn unwrap_image_to_texture(
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    window: &mut PistonWindow,
) -> Texture<gfx_device_gl::Resources> {
    Texture::from_image(
        &mut window.create_texture_context(),
        &img,
        &TextureSettings::new(),
    )
    .unwrap()
}

fn generate_mandelbrot_image(settings: &MandelbrotSettings) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(settings.width, settings.height);

    let width_64 = settings.width as f64;
    let height_64 = settings.height as f64;
    let width_scale = 4. / settings.zoom / width_64;
    let height_scale = 4. / settings.zoom / height_64;
    let half_width = width_64 / 2.;
    let half_height = height_64 / 2.;

    let length = img.width() as usize;

    img.as_mut().par_chunks_mut(length * 4).enumerate().for_each(|(y, row)|{
        let yi = (y as f64 - half_height) * height_scale + settings.offset_y;
        for (x, pixel) in row.chunks_mut(4).enumerate() {
            let xi = (x as f64 - half_width) * width_scale + settings.offset_x;
            
            let c = Complex::new(xi, yi);
            let mut z = Complex::new(xi, yi);
            let mut i = 0;
            while i < settings.max_iterations && z.norm_sqr() <= 4. {
                z = z * z + c;
                i += 1;
            }

            let h = ((i as f32 / settings.max_iterations as f32).powf(0.22) * 255.) as u8;
            pixel.copy_from_slice(&[h, h, h, 255 as u8]);
        }
    });

    img
}
