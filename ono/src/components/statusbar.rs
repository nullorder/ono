//! Component: statusbar — one-line status bar.
//!
//! Matches `specs/components/statusbar.toml`. Composition anchor: wires
//! `spinner`, `progress`, and `percentage` elements together with typed
//! params. Self-contained — imports only ratatui, sibling elements, and
//! the local `Palette`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;

use super::super::elements::percentage::Percentage;
use super::super::elements::progress::{Progress, ProgressStyle};
use super::super::elements::spinner::Spinner;
use super::super::theme::Palette;

pub struct Statusbar<'a> {
    label: &'a str,
    percent: f32,
    width: u16,
    show_spinner: bool,
    show_progress: bool,
    show_percent: bool,
    tick: u64,
    palette: &'a Palette,
}

impl<'a> Statusbar<'a> {
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            label: "Working",
            percent: 0.0,
            width: 60,
            show_spinner: false,
            show_progress: true,
            show_percent: true,
            tick: 0,
            palette,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn percent(mut self, percent: f32) -> Self {
        self.percent = percent.clamp(0.0, 1.0);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width.max(10);
        self
    }

    pub fn show_spinner(mut self, show: bool) -> Self {
        self.show_spinner = show;
        self
    }

    pub fn show_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    pub fn show_percent(mut self, show: bool) -> Self {
        self.show_percent = show;
        self
    }

    pub fn tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }
}

impl Widget for Statusbar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let y = area.y;
        let max_x = area.x + area.width.min(self.width);
        let mut x = area.x;

        if !self.label.is_empty() && x < max_x {
            let w = (self.label.chars().count() as u16).min(max_x - x);
            Line::from(self.label)
                .style(Style::default().fg(self.palette.primary))
                .render(Rect { x, y, width: w, height: 1 }, buf);
            x = (x + w + 1).min(max_x);
        }

        if self.show_spinner && x < max_x {
            Spinner::new(self.palette)
                .tick(self.tick)
                .render(Rect { x, y, width: 1, height: 1 }, buf);
            x = (x + 2).min(max_x);
        }

        if self.show_progress && x < max_x {
            let desired = self.width.saturating_sub(20).max(1);
            let bar_w = desired.min(max_x - x);
            Progress::new(self.percent, self.palette)
                .width(bar_w)
                .style(ProgressStyle::Unicode)
                .render(Rect { x, y, width: bar_w, height: 1 }, buf);
            x = (x + bar_w + 1).min(max_x);
        }

        if self.show_percent && x < max_x {
            let w = 5u16.min(max_x - x);
            Percentage::new(self.percent, self.palette).render(
                Rect { x, y, width: w, height: 1 },
                buf,
            );
        }
    }
}
