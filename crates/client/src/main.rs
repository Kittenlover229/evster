use std::rc::Rc;

use nalgebra_glm::{Vec2, Vec3};
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use engine::{
    Action, Actor, ActorTemplate, Atlas, AxialInput2D, FrameBuilder, Grid, InputHandler, Instance,
    Position, Renderer, Tile, World,
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    main().unwrap()
}

pub fn main() -> anyhow::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            pretty_env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Evster")
        .build(&event_loop)
        .unwrap();
    window.set_cursor_visible(false);
    let mut input_handler = InputHandler::new_with_filter(
        {
            use VirtualKeyCode::*;
            vec![Space, Escape, Numpad8, Numpad4, Numpad6, Numpad2]
        }
        .into_iter(),
        [{
            use VirtualKeyCode::*;
            AxialInput2D {
                normalize: false,
                up: W,
                down: S,
                right: D,
                left: A,
            }
        }]
        .into_iter(),
    );

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(800, 800));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("evster-attach-here")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let snek = ActorTemplate::new("Snek", "creature.snek");
    let snek = Rc::new(snek);

    let player = ActorTemplate::new("Player", "creature.player");
    let player = Rc::new(player);

    let mut world = World::new(16, 16);

    let (_, player) = world.grid.put_actor([0, 0], Actor::from(player))?;
    world.grid.put_actor([2, 2], Actor::from(snek))?;

    let mut renderer = pollster::block_on(Renderer::new(window));

    let atlas = Atlas::default_from_device(
        &renderer.device,
        &renderer.queue,
        &renderer.atlas_bind_layout,
    );

    let mut cursor_pos = PhysicalPosition::default();
    let mut camera_inputs = Vec2::new(0., 0.);
    let camera_speed = 12.;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => match event {
            WindowEvent::KeyboardInput { input, .. } => {
                input_handler.handle_input(input);

                use VirtualKeyCode::*;

                if input_handler.is_pressed(Escape) {
                    *control_flow = ControlFlow::Exit;
                }

                camera_inputs = input_handler.get_axial(0);

                if input_handler.is_pressed(Numpad8) {
                    let player_pos = player.borrow().position().unwrap();
                    world.submit_action(Action::move_actor(
                        player_pos,
                        player_pos + Position::new(0, 1),
                    ));
                };
            }

            WindowEvent::CursorMoved { position, .. } => {
                cursor_pos = *position;
            }

            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }

            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size);
            }

            _ => {}
        },

        Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
            input_handler.flush();

            renderer.camera.borrow_mut().position += renderer.delta_time
                * camera_speed
                * Vec3::new(camera_inputs.x as _, camera_inputs.y as _, 0.);
            renderer.refresh_camera();
            let cursor_pos = renderer.window_space_to_world(&cursor_pos);

            let mut frame = frame_from_world(&mut renderer, &world.grid, &atlas);
            frame = frame.draw_sprite(
                7,
                Instance {
                    size: 1.0,
                    pos: cursor_pos,
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
