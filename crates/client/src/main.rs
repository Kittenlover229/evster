use nalgebra_glm::vec2;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use engine::{Instance, Renderer};

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Evster").build(&event_loop).unwrap();

    let mut renderer = pollster::block_on(Renderer::new(window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size);
            }
            _ => {}
        },

        Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
            renderer.update();

            let rendered = renderer
                .begin_frame()
                .draw_sprite(
                    5,
                    Instance {
                        size: 4.0,
                        pos: vec2(0.0, 0.0),
                        angle: 45.0,
                        tint: [255, 0, 255],
                        layer: 1,
                    },
                )
                .draw_sprite(
                    0,
                    Instance {
                        size: 2.0,
                        pos: vec2(1.0, -1.0),
                        angle: -10.0,
                        tint: [0, 255, 255],
                        layer: 2,
                    },
                )
                .end_frame();

            match rendered {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }

        Event::MainEventsCleared => {
            renderer.window().request_redraw();
        }
        _ => {}
    });
}
