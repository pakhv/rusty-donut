use std::{f32::consts::PI, thread, time::Duration};

use crate::drawer::Drawer;

pub const VIEWPORT: usize = 80;

pub struct Donut {
    points: [[char; VIEWPORT]; VIEWPORT],
    a_angle: f32,
    b_angle: f32,
}

impl Donut {
    const THETA_SPACING: f32 = 0.04;
    const PHI_SPACING: f32 = 0.01;

    const A_SPACING: f32 = 0.07;
    const B_SPACING: f32 = 0.02;

    const DONUT_THICKNESS: usize = 1;
    const DONUT_RADIUS: usize = 2;
    const DISTANCE_TO_DONUT: usize = 5;
    const PROJECTION: usize =
        VIEWPORT * Self::DISTANCE_TO_DONUT * 3 / (8 * (Self::DONUT_THICKNESS + Self::DONUT_RADIUS));

    const LUMENANCE: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

    pub fn new() -> Self {
        Self {
            points: [[' '; VIEWPORT]; VIEWPORT],
            a_angle: 0.0,
            b_angle: 0.0,
        }
    }

    pub fn run(&mut self) {
        let mut drawer = Drawer::new();
        //_ = drawer.prepare_screen();

        loop {
            increment_angle(&mut self.a_angle, Self::A_SPACING);
            increment_angle(&mut self.b_angle, Self::B_SPACING);

            self.points = [[' '; VIEWPORT]; VIEWPORT];
            self.calculate_frame();
            _ = drawer.draw(&self.points);

            thread::sleep(Duration::from_millis(50));
        }

        //_ = drawer.reset_screen();
    }

    fn calculate_frame(&mut self) {
        let mut cur_theta = 0.0;
        let mut z_buffer = [[0.0; VIEWPORT]; VIEWPORT];

        loop {
            match cur_theta {
                theta if theta < 2.0 * PI => {
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
                phi if phi < 2.0 * PI => {
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
}

fn increment_angle(angle: &mut f32, spacing: f32) {
    match angle {
        angle if *angle >= 2.0 * PI => {
            *angle -= 2.0 * PI;
        }
        _ => {
            *angle += spacing;
        }
    }
}
