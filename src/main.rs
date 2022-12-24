//! Example that just prints out all the input events.

use ggez::conf;
use ggez::event::{self, MouseButton};
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawMode, FillOptions, StrokeOptions};
use ggez::mint::Point2;
use ggez::{Context, GameResult};

#[derive(PartialEq, Debug, Clone, Copy)]
struct ControlPoint {
    pos_x: f32,
    pos_y: f32,
}

struct MainState {
    stroke_width: f32,
    bezier_resolution: u32,
    max_bezier: usize,
    selected: Option<usize>,
    point_rad: f32,
    mouse_down: bool,
    control_points: Vec<ControlPoint>,
    groups: Vec<Vec<usize>>,
    bezier_points: Vec<Vec<Point2<f32>>>,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            stroke_width: 2.0,
            bezier_resolution: 16,
            max_bezier: 4,
            selected: None,
            point_rad: 5.0,
            mouse_down: false,
            control_points: vec![],
            groups: vec![vec![]],
            bezier_points: vec![vec![]],
        }
    }

    fn bezier(points: &[f32], t: f32) -> f32 {
        if points.as_ref().len() == 1 {
            return points.as_ref()[0];
        } else {
            return (1.0 - t) * Self::bezier(&points.as_ref()[0..points.as_ref().len() - 1], t)
                + t * Self::bezier(&points.as_ref()[1..points.as_ref().len()], t);
        }
    }
    fn recalculate_bezier(&mut self, update_point: &usize) {
        let (point_group_x, point_group_y): (Vec<f32>, Vec<f32>) = self
            .control_points
            .iter()
            .map(|p| (p.pos_x, p.pos_y))
            .unzip();
        let mut selected: Vec<usize> = vec![];
        let _filter = self
            .groups
            .iter()
            .enumerate()
            .filter(|(i, g)| {
                if (**g).contains(&update_point) {
                    selected.push(*i);
                    return true;
                }
                return false;
            })
            .collect::<Vec<_>>();

        self.bezier_points
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                if selected.contains(&i) {
                    *p = vec![];
                }
            });
        for selected_group in selected {
            let mut group_controls_x: Vec<f32> = vec![];
            let mut group_controls_y: Vec<f32> = vec![];
            for i in &self.groups[selected_group] {
                group_controls_x.push(point_group_x[*i]);
                group_controls_y.push(point_group_y[*i]);
            }
            let mut group_bezier = vec![];
            for i in 0..=self.bezier_resolution {
                let bezier_point_x = MainState::bezier(
                    &group_controls_x,
                    (i as f32) / (self.bezier_resolution as f32),
                );
                let bezier_point_y = MainState::bezier(
                    &group_controls_y,
                    (i as f32) / (self.bezier_resolution as f32),
                );
                let bezier_point = Point2 {
                    x: bezier_point_x,
                    y: bezier_point_y,
                };
                group_bezier.push(bezier_point);
            }
            self.bezier_points[selected_group] = group_bezier;
        }
    }

    fn draw_control_points(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        Ok(for control in &self.control_points {
            let point_rad = 10.0;
            let point = graphics::Mesh::new_circle(
                ctx,
                DrawMode::Stroke(StrokeOptions::default().with_line_width(self.stroke_width)),
                [control.pos_x, control.pos_y],
                point_rad,
                0.5,
                Color::BLACK,
            )?;
            canvas.draw(&point, Vec2::new(0.0, 0.0));
        })
    }

    fn draw_control_lines(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        let control_len = self.control_points.len();
        Ok(for i in 0..control_len {
            if i + 1 != control_len {
                let current_point = Point2 {
                    x: (&self.control_points)[i].pos_x,
                    y: (&self.control_points)[i].pos_y,
                };
                let next_point = Point2 {
                    x: (&self.control_points)[i + 1].pos_x,
                    y: (&self.control_points)[i + 1].pos_y,
                };
                let line = graphics::Mesh::new_line(
                    ctx,
                    &[current_point, next_point],
                    self.stroke_width,
                    Color::BLACK,
                )?;
                canvas.draw(&line, Vec2::new(0.0, 0.0));
            }
        })
    }

    fn draw_bezier_circles(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        Ok(
            for point in self.bezier_points.iter().flatten().collect::<Vec<_>>() {
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::Fill(FillOptions::default()),
                    *point,
                    self.point_rad,
                    0.5,
                    Color::RED,
                )?;
                canvas.draw(&circle, Vec2::new(0.0, 0.0));
            },
        )
    }

    fn draw_bezier_max(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        let bounding_box = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(FillOptions::default()),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
            },
            Color::CYAN,
        )?;
        canvas.draw(&bounding_box, Vec2::new(0.0, 0.0));
        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.selected != None {
            MainState::recalculate_bezier(self, &self.selected.unwrap());
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        // Draws bezier max text box
        // self.draw_bezier_max(ctx, &mut canvas)?;

        // Draws control points
        self.draw_control_points(ctx, &mut canvas)?;

        // Draws the lines between control points
        self.draw_control_lines(ctx, &mut canvas)?;

        // Draws the bezier dots
        self.draw_bezier_circles(ctx, &mut canvas)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            self.mouse_down = true;
            for point in &self.control_points {
                if f32::abs(point.pos_x - x) <= 10.0 && f32::abs(point.pos_y - y) <= 10.0 {
                    self.selected =
                        Some(self.control_points.iter().position(|x| x == point).unwrap());
                }
            }
        }
        if button == MouseButton::Right {
            self.mouse_down = true;
            self.control_points.push(ControlPoint {
                pos_x: x,
                pos_y: y
            });
            let last_group_index = &self.groups.len() - 1;
            let new_index = self.control_points.len() - 1;
            if self.groups[last_group_index].len() == self.max_bezier {
                self.groups.push(vec![new_index - 1, new_index]);
                self.bezier_points.push(vec![]);
            } else {
                self.groups[last_group_index].push(new_index);
            }
            self.selected = Some(new_index);
            MainState::recalculate_bezier(self, &new_index);
        }
        if button == MouseButton::Middle {
            let mut deleting_index: Option<usize> = None;
            for i in 0..self.control_points.len() {
                let point = &self.control_points[i];
                if f32::abs(point.pos_x - x) <= 10.0 && f32::abs(point.pos_y - y) <= 10.0 {
                    deleting_index = Some(i);
                }
            }

            if let Some(deleting_index) = deleting_index {
                let deleting_index_groups = self
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_i, g)| g.contains(&deleting_index))
                    .map(|(i, _g)| i)
                    .collect::<Vec<_>>();
                for group in &deleting_index_groups {
                    let updated_group_count = self
                        .groups
                        .iter()
                        .enumerate()
                        .filter(|(_i, g)| g.contains(&deleting_index))
                        .map(|(i, _g)| i)
                        .collect::<Vec<_>>()
                        .len();
                    let group_ref = &mut self.groups[*group];
                    let first_element = group_ref[0];
                    let deleting_element = self.control_points[deleting_index];
                    let last_group_element_index = group_ref[group_ref.len() - 1];
                    let last_group_element = self.control_points[last_group_element_index];
                    self.control_points[last_group_element_index] = deleting_element;
                    self.control_points[deleting_index] = last_group_element;
                    if group_ref.len() >= 2 {
                        group_ref.remove(group_ref.len() - 1);
                    } else {
                        self.groups.remove(*group);
                        self.bezier_points.remove(*group);
                        if self.groups.len() == 0 {
                            self.groups.push(vec![]);
                            self.bezier_points.push(vec![]);
                        }
                    }
                    if updated_group_count == 1 {
                        self.control_points.remove(last_group_element_index);
                    }
                    MainState::recalculate_bezier(self, &first_element);
                }
            }
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> GameResult {
        self.mouse_down = false;
        self.selected = None;
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _xrel: f32,
        _yrel: f32,
    ) -> GameResult {
        if self.mouse_down && self.selected != None {
            let mut clicked = &mut self.control_points[self.selected.unwrap()];
            clicked.pos_x = x;
            clicked.pos_y = y;
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Bezier Demo", "Lior Carmeli")
        .window_mode(
            conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::Windowed)
                .resizable(true),
        )
        .window_setup(
            conf::WindowSetup::default()
                .title("Bezier Demo")
                .samples(conf::NumSamples::Four),
        );
    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new();
    event::run(ctx, event_loop, state)
}
