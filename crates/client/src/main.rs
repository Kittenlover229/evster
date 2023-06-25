use std::rc::Rc;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use engine::{Actor, ActorPrototype, Atlas, FrameBuilder, Grid, Instance, Renderer, Tile};

pub fn frame_from_world<'a>(
    renderer: &'a mut Renderer,
    world: &Grid,
    atlas: &Atlas,
) -> FrameBuilder<'a> {
    let mut builder = renderer.begin_frame();

    for Tile {
        position: pos,
        occupier,
        ..
    } in &world.grid
    {
        if let Some(_actor) = occupier {
            let sprite_idx = atlas
                .resolve_sprite_by_name(_actor.as_ref().borrow().prototype().sprite())
                .map_or(0, |x| x.0);

            builder = builder.draw_sprite(
                sprite_idx,
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

// Palette:
// https://lospec.com/palette-list/2bit-demichrome

pub fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Evster")
        .build(&event_loop)
        .unwrap();

    let snek = ActorPrototype::new("Snek", "monster.snek");
    let snek = Rc::new(snek);

    let mut world = Grid::new(16, 16);
    world.put_actor([0, 0], Actor::from(snek.clone()))?;
    world.move_actor([0, 0], [2, 2]);

    let mut renderer = pollster::block_on(Renderer::new(window));
    let atlas = Atlas::default_from_device(
        &renderer.device,
        &renderer.queue,
        &renderer.atlas_bind_layout,
    );

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
            let frame = frame_from_world(&mut renderer, &world, &atlas);

            match frame.end_frame(&atlas) {
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
