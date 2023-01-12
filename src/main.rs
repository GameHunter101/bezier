//! Example that just prints out all the input events.

use ggez::event::MouseButton;
use ggez::graphics::{self, Color, DrawMode, FillOptions, StrokeOptions};
use ggez::mint::Point2;
use ggez::{conf, event};
use ggez::{glam::*};
use ggez::{Context, GameResult};

struct Bezier {
    control_points: Vec<Point2<f32>>,
    bezier_points: Vec<Vec<Point2<f32>>>,
    groups: Vec<Vec<usize>>,
}

impl Bezier {
    fn calculate_bezier(&mut self, points: &[f32], t: f32) -> f32 {
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
            let group_points = self.indices_to_points(group_point_indices);
            let (x_vals, y_vals) = self.seperate_points_coords(group_points);
            let bezier_resolution = 16;
            let mut new_bezier_group: Vec<Point2<f32>> = vec![];
            for i in 0..=bezier_resolution {
                let bezier_point = Point2 {
                    x: self.calculate_bezier(&x_vals, (i as f32) / (bezier_resolution as f32)),
                    y: self.calculate_bezier(&y_vals, (i as f32) / (bezier_resolution as f32)),
                };
                new_bezier_group.push(bezier_point);
            }
            if self.bezier_points.len() == 0 || self.bezier_points.get(group_index) == None {
                self.bezier_points.push(new_bezier_group);
            } else {
                self.bezier_points[group_index] = new_bezier_group;
            }
        }
    }

    fn indices_to_points(&self, points_indices: &Vec<usize>) -> Vec<&Point2<f32>> {
        let points_refs: Vec<&Point2<f32>> = points_indices
            .iter()
            .map(|i| &self.control_points[*i])
            .collect();
        points_refs
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
        // If control_points length is 0 set groups and bezier_points to empty vector
        // remove any instances of the last index in groups
        // remove groups that are smaller than 2 elements
        self.control_points.remove(point);
        if self.control_points.len() == 0 {
            self.groups = vec![];
            self.bezier_points = vec![];
            return;
        }
        let len = self.groups.len() - 1;
        self.groups[len].pop();
        let mut reversed_groups:Vec<usize> = self.groups.iter().enumerate().map(|(i,_g)| i).collect();
        reversed_groups.reverse();
        for group in reversed_groups {
            if self.groups[group].len() < 2 && group != 0 {
                self.groups.remove(group);
                self.bezier_points.remove(group);
            }
        }
        self.calculate_bezier_points(if point == self.control_points.len() {
            point - 1
        } else {
            point
        });
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
