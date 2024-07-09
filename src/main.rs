use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics;
use ggez::graphics::Drawable;
use ggez::graphics::{Color, DrawMode, Mesh};
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult};

use rosu_map::section::hit_objects::{self};
use rosu_map::Beatmap;

struct Circle {
    position: Vec2,
    radius: f32,
    tag: String,
    color: Color,
}

struct MainState {
    map: Beatmap,
    circles: Vec<Circle>,
}

const CIRCLE_SIZE: f32 = 4.0 * 10.0;

impl MainState {
    fn new(map: Beatmap) -> GameResult<MainState> {
        let mut circles = Vec::new();
        let mut circle_tag = 1;
        let colors = &map.custom_combo_colors;
        let mut color_index = 0;

        for hit_objects in &map.hit_objects {
            match &hit_objects.kind {
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
                    });

                    circle_tag += 1;
                }
                _ => {}
            }
        }

        let s = MainState { map, circles };
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
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(25, 50, 75));

        let mut previous_circle: Option<&Circle> = None;

        // I don't know how to make the new circles appear below the older ones,
        // so I just reversed the array, idk what I'll do with it.
        for circle in self.circles.iter().rev() {
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
                DrawMode::fill(),
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

            let text_fragment =
                graphics::TextFragment::new(circle.tag.to_string()).color(Color::from_rgb(0, 0, 0));

            let text = graphics::Text::new(text_fragment);
            let text_dimensions = text.dimensions(ctx).unwrap();
            let text_x = pos_x - text_dimensions.w as f32 / 2.0;
            let text_y = pos_y - text_dimensions.h as f32 / 2.0;
            canvas.draw(&text, Vec2::new(text_x, text_y));

            previous_circle = Some(&circle);
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
    let (ctx, event_loop) = ContextBuilder::new("circle_game", "Author")
        .build()
        .expect("aieee, could not create ggez context!");

    let map: Beatmap = rosu_map::from_path("./resources/diff.osu").unwrap();

    let state = MainState::new(map)?;
    event::run(ctx, event_loop, state)
}
