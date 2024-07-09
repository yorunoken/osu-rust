use std::path::Path;

use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::Drawable;
use ggez::graphics::{self, Image};
use ggez::graphics::{Color, DrawMode, Mesh};
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult};

use rodio::{Decoder, OutputStream, Sink};

use std::fs::File;
use std::io::BufReader;

use rosu_map::section::hit_objects::{self};
use rosu_map::Beatmap;

struct Circle {
    position: Vec2,
    radius: f32,
    tag: String,
    color: Color,
    start_time: f32,
    active: bool,
}

struct ApproachCircle {
    radius: f32,
    closing_speed: f32,
}

struct MainState {
    map: Beatmap,
    circles: Vec<Circle>,
    approach_circle: ApproachCircle,
    start_time: std::time::Instant,

    _stream: OutputStream,
    sink: Sink,
}

const CIRCLE_SIZE: f32 = 4.0 * 10.0;

impl MainState {
    fn new(ctx: &mut Context, map: Beatmap) -> GameResult<MainState> {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = BufReader::new(
            File::open("/home/yoru/Projects/rust/osu-rust/src/resources/audio.mp3").unwrap(),
        );
        let source = Decoder::new(file).unwrap();
        sink.append(source);
        sink.set_volume(0.1);

        let mut circles = Vec::new();
        let mut circle_tag = 1;
        let colors = &map.custom_combo_colors;
        let mut color_index = 0;

        for hit_object in &map.hit_objects {
            match &hit_object.kind {
                hit_objects::HitObjectKind::Circle(circle) => {
                    if circle.new_combo {
                        circle_tag = 1;
                        color_index += 1;
                        if color_index >= colors.len() {
                            color_index = 0;
                        }
                    }

                    let color = colors[color_index];

                    circles.push(Circle {
                        position: Vec2::new(circle.pos.x, circle.pos.y),
                        radius: CIRCLE_SIZE,
                        tag: circle_tag.to_string(),
                        color: Color::from_rgb(color.red(), color.green(), color.blue()),
                        start_time: hit_object.start_time as f32,
                        active: false,
                    });

                    circle_tag += 1;
                }
                _ => {}
            }
        }

        let s = MainState {
            map,
            circles,
            approach_circle: ApproachCircle {
                closing_speed: 10.0,
                radius: CIRCLE_SIZE + 60.0,
            },
            start_time: std::time::Instant::now(),
            _stream,
            sink,
        };
        Ok(s)
    }
    fn detect_circle(&mut self, position: Vec2) {
        let mut closest_circle_index: Option<usize> = None;

        for (index, circle) in self.circles.iter().enumerate().rev() {
            let distance_from_click = circle.position.distance(position);

            // Check if the click is within the circle's radius
            if distance_from_click < circle.radius {
                // Check if this circle is the top-most visible one at this click position
                if closest_circle_index.is_none() || index < closest_circle_index.unwrap() {
                    closest_circle_index = Some(index);
                }
            }
        }

        if let Some(index) = closest_circle_index {
            self.circle_detected(index)
        }
    }

    fn circle_detected(&mut self, circle_index: usize) {
        self.circles.remove(circle_index);
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let elapsed_time = self.start_time.elapsed().as_secs_f32() * 1000.0;

        for circle in &mut self.circles {
            if elapsed_time >= circle.start_time {
                circle.active = true;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(25, 50, 75));
        canvas.draw(
            &Image::from_path(
                ctx,
                "/yoru/home/Projects/rust/osu-rust/src/resources/sillyalbumcoverxd.jpg",
            )?,
            Vec2::new(0.0, 0.0),
        );

        let mut previous_circle: Option<&Circle> = None;

        // I don't know how to make the new circles appear below the older ones,
        // so I just reversed the array, idk what I'll do with it.
        for circle in self.circles.iter().rev() {
            if circle.active {
                if self.approach_circle.radius > CIRCLE_SIZE {
                    self.approach_circle.radius -=
                        self.approach_circle.closing_speed * ctx.time.delta().as_secs_f32();
                }

                let mut pos_x = circle.position.x;
                let mut pos_y = circle.position.y;

                if let Some(prev) = previous_circle {
                    if pos_x == prev.position.x && pos_y == prev.position.y {
                        pos_x -= 8.0;
                        pos_y -= 8.0;
                    }
                }

                let outer_mesh = Mesh::new_circle(
                    ctx,
                    DrawMode::stroke(0.5),
                    Vec2::new(pos_x, pos_y),
                    CIRCLE_SIZE + 0.5,
                    0.1,
                    Color::BLACK,
                )?;
                canvas.draw(&outer_mesh, Vec2::new(0.0, 0.0));

                let inner_mesh = Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    Vec2::new(pos_x, pos_y),
                    CIRCLE_SIZE,
                    0.1,
                    circle.color,
                )?;
                canvas.draw(&inner_mesh, Vec2::new(0.0, 0.0));

                let approach_circle_mesh = Mesh::new_circle(
                    ctx,
                    DrawMode::stroke(1.0),
                    Vec2::new(pos_x, pos_y),
                    self.approach_circle.radius,
                    0.0002,
                    Color::GREEN,
                )?;
                canvas.draw(&approach_circle_mesh, Vec2::new(0.0, 0.0));

                let text_fragment = graphics::TextFragment::new(circle.tag.to_string())
                    .color(Color::from_rgb(0, 0, 0));

                let text = graphics::Text::new(text_fragment);
                let text_dimensions = text.dimensions(ctx).unwrap();
                let text_x = pos_x - text_dimensions.w as f32 / 2.0;
                let text_y = pos_y - text_dimensions.h as f32 / 2.0;
                canvas.draw(&text, Vec2::new(text_x, text_y));

                previous_circle = Some(&circle);
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(key_input) = input.keycode {
            match key_input {
                keyboard::KeyCode::Escape => ctx.request_quit(),
                keyboard::KeyCode::A | keyboard::KeyCode::D => {
                    let click_position = Vec2::new(ctx.mouse.position().x, ctx.mouse.position().y);
                    self.detect_circle(click_position);
                }
                _ => {}
            }
        }

        Ok(())
    }

    // fn mouse_button_down_event(
    //     &mut self,
    //     _ctx: &mut Context,
    //     _button: ggez::event::MouseButton,
    //     x: f32,
    //     y: f32,
    // ) -> GameResult {
    //     let click_position = Vec2::new(x, y);
    //     self.detect_circle(click_position);

    //     Ok(())
    // }
}

pub fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("circle_game", "Author")
        .window_mode(conf::WindowMode::default().dimensions(800.0, 600.0))
        .window_setup(conf::WindowSetup::default().title("osu-rust"))
        .build()
        .expect("aieee, could not create ggez context!");

    let map: Beatmap = rosu_map::from_path("./src/resources/diff.osu").unwrap();

    let state = MainState::new(&mut ctx, map)?;

    state.sink.play();
    event::run(ctx, event_loop, state);
}
