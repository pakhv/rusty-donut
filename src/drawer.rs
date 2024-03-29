use std::io::{stdout, Result, Stdout, Write};

use crossterm::{
    cursor, execute, queue, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::donut::VIEWPORT;

pub struct Drawer(Stdout);

impl Drawer {
    pub fn new() -> Self {
        Self(stdout())
    }

    pub fn draw(&mut self, points: &[[char; VIEWPORT]; VIEWPORT]) -> Result<()> {
        queue!(self.0, terminal::Clear(terminal::ClearType::All))?;

        for (i, row) in points.iter().enumerate() {
            for (j, _) in row.iter().enumerate() {
                queue!(
                    self.0,
                    cursor::MoveTo((2 * i + 1) as u16, j as u16),
                    style::Print(points[i][j])
                )?;
                queue!(
                    self.0,
                    cursor::MoveTo(2 * i as u16, j as u16),
                    style::Print(points[i][j])
                )?;
            }
        }

        self.0.flush()
    }

    pub fn prepare_screen(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.0, cursor::Hide, terminal::EnterAlternateScreen)
    }

    pub fn reset_screen(&mut self) -> Result<()> {
        execute!(
            self.0,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        disable_raw_mode()
    }
}
