extern crate gfx_device_gl;
extern crate image;
extern crate num_complex;
extern crate piston_window;
extern crate rayon;
extern crate time;

use image::{ImageBuffer, Rgba}; // Image library
use num_complex::Complex; // Complex number struct
use piston_window::{
    Image, MouseButton, MouseCursorEvent, PistonWindow, Texture, TextureSettings, WindowSettings,
}; // Windowing library
use rayon::prelude::*; // Parallel iterator
use std::cell::RefCell; // Mutable reference cell
use std::rc::Rc; // Reference counted pointer


// Import other files
mod click_handler;
mod mandelbrot_settings;
use click_handler::DoubleClickHandler;
use mandelbrot_settings::MandelbrotSettings;

fn main() {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 480;

    let mut window: PistonWindow = WindowSettings::new("Mandelbrot!", [WIDTH, HEIGHT]) // Create a window builder object
        .exit_on_esc(true)
        .build() // Build the window
        .unwrap(); // Unwrap the result. If it is an error, panic and crash. Otherwise, return the window

    let settings = Rc::new(RefCell::new(MandelbrotSettings {
        width: WIDTH,
        height: HEIGHT,
        max_iterations: 300,
        zoom: 1.,
        zoom_exp: 1.5,
        offset_x: 0.0,
        offset_y: 0.0,
        gamma: 0.22
    }));

    // Mouse position. Use Rc and RefCell to mutate the mouse position in the event loop
    let mouse_pos = Rc::new(RefCell::new([0.0, 0.0] as [f64; 2]));

    // Clone the settings and mouse_pos to move them into the closures
    let settings_clone = Rc::clone(&settings);
    let mouse_pos_clone = Rc::clone(&mouse_pos);

    // Double click handlers. Didn't realise it would get so complicated with borrowing, but good to learn.
    let left_click_callback = Box::new(move || {
        let mut settings = settings_clone.borrow_mut();
        let [xi, yi] = mouse_to_screen(*mouse_pos_clone.borrow(), &*settings);
        settings.offset_x = xi;
        settings.offset_y = yi;
        settings.zoom = settings.zoom.clone() * settings.zoom_exp;
        true
    });

    // Clone the settings and mouse_pos for moving to closures
    let settings_clone = Rc::clone(&settings);
    let mouse_pos_clone = Rc::clone(&mouse_pos);
    let right_click_callback = Box::new(move || {
        let mut settings = settings_clone.borrow_mut();
        let [xi, yi] = mouse_to_screen(*mouse_pos_clone.borrow(), &*settings);
        settings.offset_x = xi;
        settings.offset_y = yi;
        settings.zoom = settings.zoom.clone() / settings.zoom_exp;
        true
    });

    // Create the click handlers
    let mut left_click_handler = DoubleClickHandler::new(left_click_callback, MouseButton::Left, None);
    let mut right_click_handler = DoubleClickHandler::new(right_click_callback, MouseButton::Right, None);
    let mut requires_recalculate: bool = false; // Flag to indicate if the image needs to be recalculated

    // Create a texture from the mandelbrot image to display initially
    let mut image: Texture<gfx_device_gl::Resources> =
        unwrap_image_to_texture(generate_mandelbrot_buffer(&*settings.borrow()), &mut window);

    // Event loop
    while let Some(event) = window.next() {
        // Update mouse position
        if let Some(pos) = event.mouse_cursor_args() {
            *mouse_pos.borrow_mut() = pos;
        }

        // Handle clicks
        requires_recalculate |= left_click_handler.handle_if_button_pressed(&event);
        requires_recalculate |= right_click_handler.handle_if_button_pressed(&event);

        // Recalculate if necessary
        if requires_recalculate {
            let buffer = generate_mandelbrot_buffer(&*settings.borrow());
            image = unwrap_image_to_texture(buffer, &mut window);
            requires_recalculate = false;
        }

        // Draw
        window.draw_2d(&event, |context, graphics, _| {
            Image::new().draw(&image, &Default::default(), context.transform, graphics);
        });
    }
}

/// Convert mouse position to mandelbrot coords.
fn mouse_to_screen(mouse_pos: [f64; 2], settings: &MandelbrotSettings) -> [f32; 2] {
    let [x, y] = mouse_pos;
    [
        (x as f32 - settings.width as f32 / 2.) / settings.width as f32 * 4.0 / settings.zoom
            + settings.offset_x,
        (y as f32 - settings.height as f32 / 2.) / settings.height as f32 * 4.0 / settings.zoom
            + settings.offset_y,
    ]
}

/// Convert an image to a texture for displaying.
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

/// Generate a mandelbrot image given settings.
fn generate_mandelbrot_buffer(settings: &MandelbrotSettings) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(settings.width, settings.height);
    let columns = img.width() as usize;

    // Cache some values to avoid recalculation
    let width_64 = settings.width as f32;
    let height_64 = settings.height as f32;
    let width_scale = 4. / settings.zoom / width_64;
    let height_scale = 4. / settings.zoom / height_64;
    let half_width = width_64 / 2.;
    let half_height = height_64 / 2.;

    // Iterate over the image in parallel
    img.as_mut()
        .par_chunks_mut(columns * 4) // Split the image into rows. *4 is used because each pixel has 4 channels
        .enumerate() // Enumerate the rows in parallel
        .for_each(|(y, row)| {
            let yi = (y as f32 - half_height) * height_scale + settings.offset_y; // Y coord
            for (x, pixel) in row.chunks_mut(4).enumerate() {
                let xi = (x as f32 - half_width) * width_scale + settings.offset_x; // X coord

                // Iterate the mandelbrot function: z = z^2 + c
                let c = Complex::<f32>::new(xi, yi);
                let mut z = Complex::<f32>::new(xi, yi);
                let mut i = 0;
                while i < settings.max_iterations && z.norm_sqr() <= 4. {
                    z = z * z + c;
                    i += 1;
                }

                let lum =
                    ((i as f32 / settings.max_iterations as f32).powf(settings.gamma) * 255.0) as u8; // scale final value and correct gamma
                pixel.copy_from_slice(&[lum, lum, lum, 255]); // set pixel colour
            }
        });

    img
}
