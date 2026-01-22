use pixels::{Pixels, SurfaceTexture};
use core::time::Duration;
use std::sync::Arc;
use std::time::Instant; // Added for timing
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use clap::Parser;

use crate::draw::ASEDrawing;


mod draw;

struct App<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    drawing: ASEDrawing<'a>,
    start_time: Instant, // Store when the app started
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (width, height) = self.drawing.size();
        let scaling = 8;
        let (window_width, window_height) = (width * scaling, height * scaling);
        let window_attributes = Window::default_attributes()
            .with_title("Animated Raw Pixels")
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height));
        
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let surface_texture = SurfaceTexture::new(window_width, window_height, Arc::clone(&window));
        let pixels = Pixels::new(width, height, surface_texture).unwrap();
        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let (Some(pixels), Some(window)) = (&mut self.pixels, &self.window) {
                    
                    self.drawing.draw(pixels, self.start_time.elapsed());

                    if pixels.render().is_err() {
                        event_loop.exit();
                        return;
                    }
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn main() {

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    struct Args {
        /// .ase or .aseprite file to open. See tinyase/tests/anim_idle.ase for an example file.
        aseprite_file: String,
    }

    let args = Args::parse();


    let event_loop = EventLoop::new().unwrap();
    // Use Poll to ensure the loop runs as fast as possible for smooth animation
    event_loop.set_control_flow(ControlFlow::Poll);

    let file_buf = std::fs::read(&args.aseprite_file).expect("Failed to find aseprite file");
    let header_reader = tinyase::parser::HeaderReader::new(&file_buf);
    let drawing = ASEDrawing {
        reader: header_reader,
    };
    
    let mut app = App {
        window: None,
        pixels: None,
        drawing: drawing,
        start_time: Instant::now(), // Initialize timer here
    };
    
    event_loop.run_app(&mut app).unwrap();
}