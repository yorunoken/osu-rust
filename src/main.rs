use ggez::glam::*;
use ggez::graphics;
use ggez::graphics::Drawable;
use ggez::{conf, event, Context, ContextBuilder, GameError, GameResult};

use rosu_map::Beatmap;

struct State {
    dt: std::time::Duration,
    beatmap: Beatmap,
}

const CIRCLE_SIZE: f32 = 40.0;

impl State {
    fn new(beatmap: Beatmap) -> Self {
        Self {
            dt: std::time::Duration::new(0, 0),
            beatmap,
        }
    }

    fn draw_circle(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        pos_x: f32,
        pos_y: f32,
        number: String,
    ) -> GameResult {
        // draw the circle
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            CIRCLE_SIZE,
            1.0,
            graphics::Color::BLACK,
        )?;
        canvas.draw(&circle, Vec2::new(pos_x, pos_y));

        // draw the combo in the circle
        let text = graphics::Text::new(number);
        let text_dimensions = text.dimensions(ctx).unwrap();
        let text_x = pos_x - text_dimensions.w as f32 / 2.0;
        let text_y = pos_y - text_dimensions.h as f32 / 2.0;
        canvas.draw(&text, Vec2::new(text_x, text_y));

        Ok(())
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = ctx.time.delta();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from_rgb(25, 50, 75));

        let (screen_width, screen_height) = ctx.gfx.drawable_size();

        let mut hit_number = 1;

        for hit_object in &self.beatmap.hit_objects {
            match &hit_object.kind {
                rosu_map::section::hit_objects::HitObjectKind::Circle(circle) => {
                    let circle_pos_x = circle.pos.x;
                    let circle_pos_y = circle.pos.y;

                    if circle.new_combo {
                        hit_number = 1;
                    }

                    self.draw_circle(
                        ctx,
                        &mut canvas,
                        circle_pos_x,
                        circle_pos_y,
                        hit_number.to_string(),
                    )?;

                    hit_number += 1;
                }
                _ => {}
            }
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("osu-rust", "yoru")
        .window_mode(conf::WindowMode::default().dimensions(800.0, 600.0))
        .window_setup(conf::WindowSetup::default().title("osu-rust"))
        .build()
        .unwrap();

    let map: Beatmap = rosu_map::from_path("./resources/diff.osu").unwrap();

    let state = State::new(map);

    event::run(ctx, event_loop, state)
}
