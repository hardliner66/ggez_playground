#![windows_subsystem = "windows"]

/// Based on the bunnymark example from [`tetra`](https://crates.io/crates/tetra)
/// which is based on https://github.com/openfl/openfl-samples/tree/master/demos/BunnyMark
/// Original BunnyMark (and sprite) by Iain Lobb
use std::env;
use std::path;

use nalgebra as na;
use rand::rngs::ThreadRng;
use rand::{self, Rng};

use conf::WindowSetup;
use event::{quit, KeyCode, KeyMods};
use ggez::graphics::{spritebatch::SpriteBatch, Color, Image};
use ggez::Context;
use ggez::*;
use input::mouse::position;

// NOTE: Using a high number here yields worse performance than adding more bunnies over
// time - I think this is due to all of the RNG being run on the same tick...
const INITIAL_BUNNIES: usize = 100;
const WIDTH: u16 = 1280;
const HEIGHT: u16 = 720;
const GRAVITY: f32 = 0.5;

struct Bunny {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
}

impl Bunny {
    fn new(x: f32, y: f32, rng: &mut ThreadRng) -> Bunny {
        let x_vel = rng.gen::<f32>() * 5.0;
        let y_vel = (rng.gen::<f32>() * 5.0) - 2.5;

        Bunny {
            position: na::Point2::new(x, y),
            velocity: na::Vector2::new(x_vel, y_vel),
        }
    }
}

struct GameState {
    rng: ThreadRng,
    texture: Image,
    bunnies: Vec<Bunny>,
    max_x: f32,
    max_y: f32,

    delete_held: bool,
    add_held: bool,
    ctrl_held: bool,

    click_timer: i32,
    bunnybatch: SpriteBatch,
    batched_drawing: bool,
}

impl GameState {
    fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        let mut rng = rand::thread_rng();

        let img =
            image::load_from_memory(include_bytes!("../resources/wabbit_alpha.png"))?.to_rgba();
        let (width, height) = img.dimensions();

        let texture = Image::from_rgba8(ctx, width as u16, height as u16, &img)?;
        let mut bunnies = Vec::with_capacity(INITIAL_BUNNIES);
        let max_x = (WIDTH - texture.width()) as f32;
        let max_y = (HEIGHT - texture.height()) as f32;

        for _ in 0..INITIAL_BUNNIES {
            bunnies.push(Bunny::new(0.0, 0.0, &mut rng));
        }

        let bunnybatch = SpriteBatch::new(texture.clone());

        Ok(GameState {
            rng,
            texture,
            bunnies,
            max_x,
            max_y,

            delete_held: false,
            add_held: false,
            ctrl_held: false,

            click_timer: 0,
            bunnybatch,
            batched_drawing: true,
        })
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            if self.click_timer > 0 {
                self.click_timer -= 1;
            }

            if self.add_held {
                if self.ctrl_held {
                    let (width, _) = ggez::graphics::size(ctx);
                    for _ in 0..INITIAL_BUNNIES {
                        let width = self.rng.gen_range(0.0, width);
                        self.bunnies.push(Bunny::new(width, 0.0, &mut self.rng));
                    }
                } else {
                    let pos = position(ctx);

                    self.bunnies.push(Bunny::new(pos.x, pos.y, &mut self.rng));
                }
            }

            let bunny_count = self.bunnies.len();
            if self.delete_held {
                if self.ctrl_held {
                    let count = if bunny_count > INITIAL_BUNNIES {
                        INITIAL_BUNNIES
                    } else {
                        bunny_count
                    };
                    for _ in 0..count {
                        let index = self.rng.gen_range(0, self.bunnies.len());
                        self.bunnies.remove(index);
                    }
                } else {
                    if bunny_count > 0 {
                        let index = self.rng.gen_range(0, bunny_count);
                        self.bunnies.remove(index);
                    }
                }
            }

            for bunny in &mut self.bunnies {
                bunny.position += bunny.velocity;
                bunny.velocity.y += GRAVITY;

                if bunny.position.x > self.max_x {
                    bunny.velocity.x *= -1.0;
                    bunny.position.x = self.max_x;
                } else if bunny.position.x < 0.0 {
                    bunny.velocity.x *= -1.0;
                    bunny.position.x = 0.0;
                }

                if bunny.position.y > self.max_y {
                    bunny.velocity.y *= -0.8;
                    bunny.position.y = self.max_y;

                    if self.rng.gen::<bool>() {
                        bunny.velocity.y -= 3.0 + (self.rng.gen::<f32>() * 4.0);
                    }
                } else if bunny.position.y < 0.0 {
                    bunny.velocity.y = 0.0;
                    bunny.position.y = 0.0;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from((0.392, 0.584, 0.929)));

        if self.batched_drawing {
            self.bunnybatch.clear();
            for bunny in &self.bunnies {
                self.bunnybatch.add((bunny.position,));
            }
            graphics::draw(ctx, &self.bunnybatch, (na::Point2::new(0.0, 0.0),))?;
        } else {
            for bunny in &self.bunnies {
                graphics::draw(ctx, &self.texture, (bunny.position,))?;
            }
        }

        graphics::set_window_title(
            ctx,
            &format!(
                "BunnyMark - {} bunnies - {:.0} FPS - batched drawing: {}",
                self.bunnies.len(),
                timer::fps(ctx),
                self.batched_drawing
            ),
        );
        graphics::present(ctx)?;

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Space {
            self.batched_drawing = !self.batched_drawing;
        }
        if keycode == KeyCode::Back {
            self.bunnies.clear();
        }
        if keycode == KeyCode::Escape {
            quit(ctx);
        }

        if keycode == KeyCode::LControl || keycode == KeyCode::RControl {
            self.ctrl_held = true;
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if keycode == KeyCode::LControl || keycode == KeyCode::RControl {
            self.ctrl_held = false;
        }
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: input::mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == input::mouse::MouseButton::Left {
            self.add_held = true;
        }
        if button == input::mouse::MouseButton::Right {
            self.delete_held = true;
        }
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: input::mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == input::mouse::MouseButton::Right {
            self.delete_held = false;
        }
        if button == input::mouse::MouseButton::Left {
            self.add_held = false;
        }
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("bunnymark", "ggez")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup {
            vsync: false,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
