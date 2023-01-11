//! Example that just prints out all the input events.

use ggez::event::MouseButton;
use ggez::graphics::{self, Color, DrawMode, FillOptions, StrokeOptions};
use ggez::mint::Point2;
use ggez::{conf, event};
use ggez::{glam::*, input};
use ggez::{Context, GameResult};

struct Bezier {
    control_points: Vec<Point2<f32>>,
    bezier_points: Vec<Vec<Point2<f32>>>,
    groups: Vec<Vec<usize>>,
}

impl Bezier {
    fn calculate_bezier(&mut self, points: &[f32], t: f32) -> f32 {
        // println!("{:?}",points);
        if points.as_ref().len() == 1 {
            return points.as_ref()[0];
        } else {
            return (1.0 - t)
                * self.calculate_bezier(&points.as_ref()[0..points.as_ref().len() - 1], t)
                + t * self.calculate_bezier(&points.as_ref()[1..points.as_ref().len()], t);
        }
    }
    fn seperate_points_coords(&self, points: Vec<&Point2<f32>>) -> (Vec<f32>, Vec<f32>) {
        let (x_vals, y_vals): (Vec<f32>, Vec<f32>) = points.iter().map(|p| (p.x, p.y)).unzip();
        (x_vals, y_vals)
    }
    fn calculate_bezier_points(&mut self, point: usize) {
        // Get all the groups that the point is part of
        // loop over every one of the groups
        // calculate_bezier for every point in the group

        let group_contains_point = self.get_groups_of_point(point);
        for group_index in group_contains_point {
            let group_point_indices = &self.groups[group_index];
            let group_points: Vec<&Point2<f32>> = group_point_indices
                .iter()
                .map(|i| &self.control_points[*i])
                .collect();
            let (x_vals, y_vals) = self.seperate_points_coords(group_points);
            let bezier_resolution = 16;
            let mut bezier_group: Vec<Point2<f32>> = vec![];
            for i in 0..=bezier_resolution {
                let bezier_point = Point2 {
                    x: self.calculate_bezier(&x_vals, (i as f32) / (bezier_resolution as f32)),
                    y: self.calculate_bezier(&y_vals, (i as f32) / (bezier_resolution as f32)),
                };
                bezier_group.push(bezier_point);
            }
            if self.bezier_points.len() == 0 || self.bezier_points.get(group_index) == None {
                self.bezier_points.push(bezier_group);
            } else {
                self.bezier_points[group_index] = bezier_group;
            }
        }
    }

    fn get_groups_of_point(&mut self, point: usize) -> Vec<usize> {
        let group_indices: Vec<usize> = self
            .groups
            .iter()
            .enumerate()
            .map(|(i, g)| if g.contains(&point) { Some(i) } else { None })
            .collect::<Vec<Option<usize>>>()
            .iter()
            .filter_map(|x| *x)
            .collect();
        group_indices
    }
    fn add_point(&mut self, x: f32, y: f32) {
        let new_point = Point2 { x, y };
        self.control_points.push(new_point);
        if self.groups.len() > 0 {
            let last_group_index = self.groups.len() - 1;
            let added_index = self.control_points.len() - 1;
            if (&self.groups[last_group_index]).len() < 4 {
                self.groups[last_group_index].push(added_index);
            } else {
                self.groups.push(vec![added_index - 1, added_index]);
            }
        } else {
            self.groups.push(vec![0]);
        };
        self.calculate_bezier_points(self.control_points.len() - 1);
    }
    fn move_point(&mut self, point_index: usize, x: f32, y: f32) {
        let point = &mut self.control_points[point_index];
        point.x = x;
        point.y = y;
    }
    fn delete_point(&mut self, point: usize) {
        self.control_points.remove(point);
        let len = self.groups.len() - 1;
        self.groups[len].pop();
        if point == 0 && self.control_points.len()==point {
            self.bezier_points.remove(0);
            self.groups.remove(0);
        } else {
            self.calculate_bezier_points(if point == self.control_points.len() {
                point - 1
            } else {
                point
            });
        }
    }
}

struct MainState {
    bezier: Bezier,
    mouse_down: bool,
    selected: Option<usize>,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            bezier: Bezier {
                control_points: vec![],
                bezier_points: vec![],
                groups: vec![],
            },
            mouse_down: false,
            selected: None,
        }
    }

    fn draw_control_points(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        Ok(for control in &self.bezier.control_points {
            let point_rad = 10.0;
            let point = graphics::Mesh::new_circle(
                ctx,
                DrawMode::Stroke(StrokeOptions::default().with_line_width(2.0)),
                [control.x, control.y],
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
        let control_len = self.bezier.control_points.len();
        Ok(for i in 0..control_len {
            if i + 1 != control_len {
                let current_point = Point2 {
                    x: (&self.bezier.control_points)[i].x,
                    y: (&self.bezier.control_points)[i].y,
                };
                let next_point = Point2 {
                    x: (&self.bezier.control_points)[i + 1].x,
                    y: (&self.bezier.control_points)[i + 1].y,
                };
                let line =
                    graphics::Mesh::new_line(ctx, &[current_point, next_point], 2.0, Color::BLACK)?;
                canvas.draw(&line, Vec2::new(0.0, 0.0));
            }
        })
    }

    fn draw_bezier_circles(
        &mut self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        let bezier_points = &self.bezier.bezier_points;
        let point_rad = 5.0;
        Ok(for point in bezier_points.iter().flatten() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::Fill(FillOptions::default()),
                *point,
                point_rad,
                0.5,
                Color::RED,
            )?;
            canvas.draw(&circle, Vec2::new(0.0, 0.0));
        })
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
    fn get_clicked_point(&mut self, x: f32, y: f32) {
        for (i, point) in self.bezier.control_points.iter().enumerate() {
            if f32::abs(point.x - x) <= 10.0 && f32::abs(point.y - y) <= 10.0 {
                self.selected = Some(i);
            }
        }
    }
}
impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.2, 0.2, 0.2, 1.0]));

        self.draw_bezier_circles(ctx, &mut canvas)?;
        self.draw_control_points(ctx, &mut canvas)?;
        self.draw_control_lines(ctx, &mut canvas)?;

        canvas.finish(ctx)?;
        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if button == MouseButton::Right {
            self.mouse_down = true;
            self.bezier.add_point(x, y);
            self.get_clicked_point(x, y);
        }
        if button == MouseButton::Left {
            self.mouse_down = true;
            self.get_clicked_point(x, y);
        }
        if button == MouseButton::Middle {
            self.get_clicked_point(x, y);
            if let Some(point) = self.selected {
                self.bezier.delete_point(point);
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
    ) -> Result<(), ggez::GameError> {
        self.mouse_down = false;
        self.selected = None;
        Ok(())
    }
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        if self.mouse_down {
            if let Some(point) = self.selected {
                self.bezier.move_point(point, x, y);
                self.bezier.calculate_bezier_points(point);
            }
        }
        Ok(())
    }
}
/*
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
            self.control_points
                .push(ControlPoint { pos_x: x, pos_y: y });
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
}*/

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
