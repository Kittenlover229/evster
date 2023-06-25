use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use engine::{Actor, FrameBuilder, Instance, Renderer, Tile, World};

pub fn frame_from_world<'a>(renderer: &'a mut Renderer, world: &'a World) -> FrameBuilder<'a> {
    let mut builder = renderer.begin_frame();

    for Tile {
        position: pos,
        occupier,
        ..
    } in &world.grid
    {
        if let Some(_actor) = occupier {
            builder = builder.draw_sprite(
                0,
                Instance {
                    size: 1.0,
                    pos: [pos.x as f32, pos.y as f32].into(),
                    layer: 1,
                    angle: 0.0,
                    tint: [255; 3],
                },
            );
        }
    }

    builder
}

pub fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Evster")
        .build(&event_loop)
        .unwrap();

    let mut world = World::new(16, 16);
    world.put_actor([0, 0].into(), Actor {})?;

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
            let frame = frame_from_world(&mut renderer, &world);

            match frame.end_frame() {
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
