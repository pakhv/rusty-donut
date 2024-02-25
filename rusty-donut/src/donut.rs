use std::{
    f32::consts::PI,
    io::Result,
    time::{Duration, SystemTime},
};

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};

use crate::drawer::Drawer;

pub const VIEWPORT: usize = 80;

#[derive(PartialEq, Eq)]
enum UserInput {
    Stop,
    Resume,
    Left,
    Right,
    Up,
    Down,
    Quit,
}

pub struct Donut {
    user_input: UserInput,
    last_render: SystemTime,
    points: [[char; VIEWPORT]; VIEWPORT],
    a_angle: f32,
    b_angle: f32,
}

impl Donut {
    const MS_PER_RENDER: u128 = 50;

    const THETA_SPACING: f32 = 0.04;
    const PHI_SPACING: f32 = 0.01;

    const A_SPACING: f32 = 0.07;
    const B_SPACING: f32 = 0.07;

    const DONUT_THICKNESS: usize = 1;
    const DONUT_RADIUS: usize = 2;
    const DISTANCE_TO_DONUT: usize = 15;
    const PROJECTION: usize =
        VIEWPORT * Self::DISTANCE_TO_DONUT * 3 / (8 * (Self::DONUT_THICKNESS + Self::DONUT_RADIUS));

    const LUMENANCE: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

    const FULL_ANGLE_RANGE: f32 = 2.0 * PI;

    pub fn new() -> Self {
        Self {
            user_input: UserInput::Resume,
            last_render: SystemTime::now(),
            points: [[' '; VIEWPORT]; VIEWPORT],
            a_angle: 0.0,
            b_angle: 0.0,
        }
    }

    pub fn run(&mut self) {
        let mut drawer = Drawer::new();
        drawer.prepare_screen().expect("well, that's unlucky");

        match self.calculate_and_render(&mut drawer) {
            Ok(_) => (),
            Err(err) => println!("{err}"),
        }

        _ = drawer.reset_screen().expect("what did you do?");
    }

    fn calculate_and_render(&mut self, drawer: &mut Drawer) -> Result<()> {
        loop {
            self.read_user_input()?;

            if self.user_input == UserInput::Quit {
                break Ok(());
            }

            self.increment_angle();

            self.points = [[' '; VIEWPORT]; VIEWPORT];
            self.calculate_frame();

            if self.last_render.elapsed().unwrap().as_millis() >= Self::MS_PER_RENDER {
                drawer.draw(&self.points)?;
            }
        }
    }

    fn calculate_frame(&mut self) {
        let mut cur_theta = 0.0;
        let mut z_buffer = [[0.0; VIEWPORT]; VIEWPORT];

        loop {
            match cur_theta {
                theta if theta < Self::FULL_ANGLE_RANGE => {
                    self.calculate_donut_points(theta, &mut z_buffer);
                    cur_theta += Self::THETA_SPACING;
                }
                _ => break,
            }
        }
    }

    fn calculate_donut_points(&mut self, theta: f32, z_buffer: &mut [[f32; VIEWPORT]; VIEWPORT]) {
        let (cos_a, sin_a) = (f32::cos(self.a_angle), f32::sin(self.a_angle));
        let (cos_b, sin_b) = (f32::cos(self.b_angle), f32::sin(self.b_angle));
        let (cos_theta, sin_theta) = (f32::cos(theta), f32::sin(theta));

        let mut cur_phi = 0.0;

        loop {
            match cur_phi {
                phi if phi < Self::FULL_ANGLE_RANGE => {
                    let (cos_phi, sin_phi) = (f32::cos(phi), f32::sin(phi));

                    let circle_x =
                        Self::DONUT_RADIUS as f32 + (Self::DONUT_THICKNESS as f32) * cos_theta;
                    let circle_y = Self::DONUT_THICKNESS as f32 * sin_theta;

                    let x = circle_x * (cos_b * cos_phi + sin_a * sin_b * sin_phi)
                        - circle_y * cos_a * sin_b;
                    let y = circle_x * (sin_b * cos_phi - sin_a * cos_b * sin_phi)
                        + circle_y * cos_a * cos_b;
                    let z = Self::DISTANCE_TO_DONUT as f32
                        + cos_a * circle_x * sin_phi
                        + circle_y * sin_a;
                    let ooz = 1.0 / z;

                    let x_p = (VIEWPORT as f32 / 2.0 + Self::PROJECTION as f32 * ooz * x) as usize;
                    let y_p = (VIEWPORT as f32 / 2.0 - Self::PROJECTION as f32 * ooz * y) as usize;

                    let luminance = cos_phi * cos_theta * sin_b
                        - cos_a * cos_theta * sin_phi
                        - sin_a * sin_theta
                        + cos_b * (cos_a * sin_theta - cos_theta * sin_a * sin_phi);

                    if luminance > 0.0 && ooz > z_buffer[x_p][y_p] {
                        z_buffer[x_p][y_p] = ooz;
                        let lumenance_idx = (luminance * 8.0) as usize;

                        self.points[x_p][y_p] = Self::LUMENANCE[lumenance_idx];
                    }

                    cur_phi += Self::PHI_SPACING;
                }
                _ => break,
            }
        }
    }

    fn read_user_input(&mut self) -> Result<()> {
        if poll(Duration::from_millis(Self::MS_PER_RENDER as u64))? {
            let input = match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => UserInput::Left,
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => UserInput::Right,
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                }) => UserInput::Up,
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => UserInput::Down,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(ch),
                    ..
                }) => match ch {
                    'q' => UserInput::Quit,
                    ' ' => match self.user_input {
                        UserInput::Resume => UserInput::Stop,
                        _ => UserInput::Resume,
                    },
                    _ => return Ok(()),
                },
                _ => return Ok(()),
            };

            self.user_input = input;
            return Ok(());
        }

        Ok(())
    }

    fn increment_angle(&mut self) {
        let (a_inc, b_inc) = match self.user_input {
            UserInput::Stop => (0.0, 0.0),
            UserInput::Resume => (Self::A_SPACING, Self::B_SPACING),
            UserInput::Down => {
                self.user_input = UserInput::Stop;
                (-Self::A_SPACING, 0.0)
            }
            UserInput::Up => {
                self.user_input = UserInput::Stop;
                (Self::A_SPACING, 0.0)
            }
            UserInput::Left => {
                self.user_input = UserInput::Stop;
                (0.0, Self::B_SPACING)
            }
            UserInput::Right => {
                self.user_input = UserInput::Stop;
                (0.0, -Self::B_SPACING)
            }
            _ => return,
        };

        self.a_angle += a_inc;
        self.b_angle += b_inc;

        if self.a_angle >= Self::FULL_ANGLE_RANGE {
            self.a_angle -= Self::FULL_ANGLE_RANGE;
        }

        if self.b_angle >= Self::FULL_ANGLE_RANGE {
            self.b_angle -= Self::FULL_ANGLE_RANGE;
        }
    }
}
