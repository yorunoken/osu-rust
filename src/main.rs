use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics;
use ggez::graphics::Drawable;
use ggez::graphics::{Color, DrawMode, Mesh};
use ggez::{Context, ContextBuilder, GameResult};

use rosu_map::section::hit_objects::{self};
use rosu_map::Beatmap;

struct Circle {
    position: Vec2,
    radius: f32,
    tag: String,
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

        for hit_objects in &map.hit_objects {
            match &hit_objects.kind {
                hit_objects::HitObjectKind::Circle(circle) => {
                    if circle.new_combo {
                        circle_tag = 1;
                    }

                    circles.push(Circle {
                        position: Vec2::new(circle.pos.x, circle.pos.y),
                        radius: CIRCLE_SIZE,
                        tag: circle_tag.to_string(),
                    });

                    circle_tag += 1;
                }
                _ => {}
            }
        }

        let s = MainState { map, circles };
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from_rgb(25, 50, 75));

        for circle in &self.circles {
            let circle_mesh = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                circle.position,
                CIRCLE_SIZE,
                0.1,
                Color::BLACK,
            )?;
            canvas.draw(&circle_mesh, Vec2::new(0.0, 0.0));

            let text = graphics::Text::new(circle.tag.to_string());
            let text_dimensions = text.dimensions(ctx).unwrap();
            let text_x = circle.position.x - text_dimensions.w as f32 / 2.0;
            let text_y = circle.position.y - text_dimensions.h as f32 / 2.0;
            canvas.draw(&text, Vec2::new(text_x, text_y));
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let click_position = Vec2::new(x, y);
        let mut closest_circle_index: Option<usize> = None;
        let mut closest_distance = f32::MAX;

        for (i, circle) in self.circles.iter().enumerate() {
            let distance_from_click = circle.position.distance(click_position);

            if distance_from_click < circle.radius && distance_from_click < closest_distance {
                closest_distance = distance_from_click;
                closest_circle_index = Some(i);
            }
        }

        if let Some(index) = closest_circle_index {
            self.circles.remove(index);
        }

        Ok(())
    }
}

pub fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("circle_game", "Author")
        .build()
        .expect("aieee, could not create ggez context!");

    let map: Beatmap = rosu_map::from_path("./resources/diff.osu").unwrap();

    let state = MainState::new(map)?;
    event::run(ctx, event_loop, state)
}
