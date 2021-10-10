use std::iter;
use std::time::Instant;

use chrono::Timelike;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

/// A custom event type for the winit app.
enum Event {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal();//std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        //self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::with_user_event();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("egui-wgpu_winit example")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        //wgpu_subscriber::initialize_default_subscriber(None);
        // Temporarily avoid srgb formats for the swapchain on the web

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            //run(event_loop, window, wgpu::TextureFormat::Bgra8UnormSrgb).await;
            run(event_loop, window).await;
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let query_string = web_sys::window().unwrap().location().search().unwrap();
        let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
            .map(|x| x.parse().ok())
            .flatten()
            .unwrap_or(log::Level::Error);
        console_log::init_with_level(level).expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        use wasm_bindgen::{prelude::*, JsCast};
        wasm_bindgen_futures::spawn_local(async move {
            let run = run(event_loop, window).await;
          });
    }
}

/// A simple egui + wgpu + winit based example.
async fn run(event_loop: winit::event_loop::EventLoop<Event>, window: winit::window::Window) {

    let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let instance = wgpu::Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter =
        wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

    let (mut device, mut queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            label: None,
        },
        None,
    ).await.expect("Unable to find a suitable GPU adapter!");

    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal{});//std::sync::Mutex::new(
    //    event_loop.create_proxy(),
    //)));

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    // Display the demo application that ships with egui.
    let mut demo_app = egui_demo_lib::WrapApp::default();

    //let start_time = Instant::now();
    //let mut previous_frame_time = None;
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                //platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame.
                //let egui_start = Instant::now();
                platform.begin_frame();
                let mut app_output = epi::backend::AppOutput::default();

                let mut frame = epi::backend::FrameBuilder {
                    info: epi::IntegrationInfo {
                        web_info: None,
                        cpu_usage: None,
                        seconds_since_midnight: None,
                        native_pixels_per_point: Some(window.scale_factor() as _),
                        prefer_dark_mode: None,
                    },
                    tex_allocator: &mut egui_rpass,
                    output: &mut app_output,
                    repaint_signal: repaint_signal.clone(),
                }
                .build();

                // Draw the demo application.
                demo_app.update(&platform.context(), &mut frame);

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let (_output, paint_commands) = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(paint_commands);

                //let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
                //previous_frame_time = Some(frame_time);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };
                egui_rpass.update_texture(&device, &queue, &platform.context().texture());
                egui_rpass.update_user_textures(&device, &queue);
                egui_rpass.update_buffers(&mut device, &mut queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();
                // Submit the commands.
                queue.submit(iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();

                // Suppport reactive on windows only, but not on linux.
                // if _output.needs_repaint {
                //     *control_flow = ControlFlow::Poll;
                // } else {
                //     *control_flow = ControlFlow::Wait;
                // }
            }
            MainEventsCleared | UserEvent(Event::RequestRedraw) => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}

/// Time of day as seconds since midnight. Used for clock in demo app.
//pub fn seconds_since_midnight() -> f64 {
//    let time = chrono::Local::now().time();
//    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
//}

#[cfg(target_arch = "wasm32")]
/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}
