use std::rc::Rc;

use nalgebra_glm::Vec2;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use engine::{
    Action, Actor, ActorTemplate, Atlas, FrameBuilder, Grid, Instance, Renderer, Tile, World,
};

pub fn frame_from_world<'a>(
    renderer: &'a mut Renderer,
    world: &Grid,
    atlas: &'a Atlas,
) -> FrameBuilder<'a> {
    let mut builder = renderer.begin_frame(atlas);

    for Tile {
        position: pos,
        occupier,
        ..
    } in &world.grid
    {
        if let Some(_actor) = occupier {
            let sprite_idx = atlas
                .resolve_resource(_actor.as_ref().borrow().template().resource_name())
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
    window.set_cursor_visible(false);

    let snek = ActorTemplate::new("Snek", "monster.snek");
    let snek = Rc::new(snek);

    let mut world = World::new(16, 16);

    world.grid.put_actor([0, 0], Actor::from(snek.clone()))?;
    world.grid.move_actor([0, 0], [2, 2]);

    let mut renderer = pollster::block_on(Renderer::new(window));

    let atlas = Atlas::default_from_device(
        &renderer.device,
        &renderer.queue,
        &renderer.atlas_bind_layout,
    );

    let mut thing = Vec2::new(0., 0.);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                world.submit_action(Action::move_actor([2, 2], [0, 0]));
            }

            WindowEvent::CursorMoved { position, .. } => {
                thing = renderer.window_space_to_world(position);
            }

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
            let mut frame = frame_from_world(&mut renderer, &world.grid, &atlas);
            frame = frame.draw_sprite(
                7,
                Instance {
                    size: 1.0,
                    pos: thing,
                    layer: 3,
                    angle: 0.0,
                    tint: [255; 3],
                },
            );

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
