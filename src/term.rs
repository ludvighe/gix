/// Term: A tui helper
///
/// NOTE:
///  - Term runs in raw mode, meaning all key combinations need to be captured to work. For
///    example Ctrl+C or Ctrl+Z.
///
use crossterm::ExecutableCommand;
use crossterm::cursor::MoveTo;
use crossterm::event::read;
use crossterm::event::{self, Event};
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{ClearType, disable_raw_mode, enable_raw_mode};
use std::fmt::Display;
use std::io::{Stdout, Write, stdout};
use std::ops::{Add, Div, Mul, Sub};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

pub struct Term {
    stdout: Stdout,
}

#[allow(unused)]
impl Term {
    pub fn new() -> Term {
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        stdout
            .execute(crossterm::terminal::EnterAlternateScreen)
            .unwrap();
        stdout.execute(crossterm::cursor::Hide).unwrap();
        Term { stdout }
    }

    pub fn close(&mut self) {
        self.clear_all();
        self.stdout.execute(crossterm::cursor::Show).unwrap();
        self.stdout
            .execute(crossterm::terminal::LeaveAlternateScreen)
            .unwrap();
        disable_raw_mode().unwrap();
    }

    pub fn size() -> Vec2 {
        match crossterm::terminal::size() {
            Ok(value) => Vec2::from(value),
            _ => Vec2::empty(),
        }
    }

    pub fn reset_cursor(&mut self) {
        self.stdout.execute(MoveTo(0, 0)).unwrap();
    }

    pub fn clear_all(&mut self) {
        self.stdout
            .execute(crossterm::terminal::Clear(ClearType::All))
            .unwrap();
        self.stdout.execute(MoveTo(0, 0)).unwrap();
    }

    /// Sets background color for following text until reset_colors is called.
    pub fn set_bg_color(&mut self, color: Color) {
        self.stdout.execute(SetBackgroundColor(color)).unwrap();
    }
    /// Sets foreground color for following text until reset_colors is called.
    pub fn set_fg_color(&mut self, color: Color) {
        self.stdout.execute(SetForegroundColor(color)).unwrap();
    }
    pub fn reset_colors(&mut self) {
        self.stdout.execute(ResetColor).unwrap();
    }

    /// Sets attribute for following text until reset_attributes is called.
    pub fn set_attribute(&mut self, attribute: Attribute) {
        self.stdout.execute(SetAttribute(attribute)).unwrap();
    }
    pub fn reset_attributes(&mut self) {
        self.stdout.execute(SetAttribute(Attribute::Reset)).unwrap();
    }

    pub fn write_text(&mut self, at: Vec2, text: impl std::fmt::Display) {
        self.stdout.execute(MoveTo(at.x, at.y)).unwrap();
        write!(self.stdout, "{}", text).unwrap();
        self.stdout.flush().unwrap();
    }
    pub fn write_bold_text(&mut self, at: Vec2, text: impl std::fmt::Display) {
        self.stdout.execute(MoveTo(at.x, at.y)).unwrap();
        self.set_attribute(Attribute::Bold);
        write!(self.stdout, "{}", text).unwrap();
        self.reset_attributes();
        self.stdout.flush().unwrap();
    }

    pub fn set_pixel(
        &mut self,
        at: Vec2,
        bg_color: Option<Color>,
        fg_color: Option<Color>,
        ch: Option<&str>,
    ) {
        self.stdout.execute(MoveTo(at.x, at.y)).unwrap();
        if let Some(bg) = bg_color {
            self.stdout.execute(SetBackgroundColor(bg)).unwrap();
        }
        if let Some(fg) = fg_color {
            self.stdout.execute(SetForegroundColor(fg)).unwrap();
        }
        write!(self.stdout, "{}", ch.unwrap_or(" ")).unwrap();
        self.stdout.execute(ResetColor).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn draw_text_bubble(&mut self, at: Vec2, text: impl std::fmt::Display) {
        let string = text.to_string();
        let lines: Vec<&str> = string.lines().collect();
        let max_len = string.lines().map(|l| l.len()).max().unwrap_or(0);
        let padding: u16 = 0;
        let outline_color = Some(Color::AnsiValue(22));

        let size = Vec2::new(
            max_len as u16 + (padding * 2) + 2,
            lines.len() as u16 + (padding * 2) + 1,
        );

        self.set_pixel(at, None, outline_color, Some("┏"));
        self.set_pixel(at + Vec2::new(size.x, 0), None, outline_color, Some("┓"));
        self.set_pixel(at + Vec2::new(0, size.y), None, outline_color, Some("┗"));
        self.set_pixel(at + size, None, outline_color, Some("┛"));

        for x in 1..size.x {
            self.set_pixel(at + Vec2::new(x, 0), None, outline_color, Some("━"));
            self.set_pixel(at + Vec2::new(x, size.y), None, outline_color, Some("━"));
        }

        for y in 1..size.y {
            self.set_pixel(at + Vec2::new(0, y), None, outline_color, Some("┃"));
            self.set_pixel(at + Vec2::new(size.x, y), None, outline_color, Some("┃"));
        }

        for (i, line) in lines.iter().enumerate() {
            self.write_bold_text(at + Vec2::new(1, i as u16 + 1), *line);
        }
    }

    pub fn set_pixel_bg(&mut self, at: Vec2, color: Color) {
        self.stdout.execute(MoveTo(at.x, at.y)).unwrap();
        self.stdout.execute(SetBackgroundColor(color)).unwrap();
        write!(self.stdout, " ").unwrap();
        self.stdout.execute(ResetColor).unwrap();
    }

    pub fn draw(&mut self, at: Vec2, graphic: &str, color: Color) {
        for (y, line) in graphic.lines().enumerate() {
            for (x, c) in line.graphemes(true).enumerate() {
                if c == " " {
                    continue;
                }

                self.set_pixel(
                    Vec2::new(at.x + x as u16, at.y + y as u16),
                    None,
                    Some(color),
                    Some(c),
                );
            }
        }
    }

    pub fn read_event(&self, timeout_ms: u64) -> Option<Event> {
        if event::poll(Duration::from_millis(timeout_ms)).ok()? {
            Some(read().unwrap())
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: u16,
    pub y: u16,
}

#[allow(unused)]
impl Vec2 {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
    pub fn empty() -> Self {
        Self::new(0, 0)
    }
    pub fn x(&self) -> Self {
        Vec2::new(self.x, 0)
    }
    pub fn y(&self) -> Self {
        Vec2::new(0, self.y)
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from(value: (u16, u16)) -> Self {
        Self::new(value.0, value.1)
    }
}
impl From<(usize, usize)> for Vec2 {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0 as u16, value.1 as u16)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<u16> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: u16) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<u16> for Vec2 {
    type Output = Self;

    fn div(self, scalar: u16) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
