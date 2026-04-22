//! Themes, palettes, and rendering knobs.
//!
//! A [`Theme`] bundles a [`Palette`] (nine semantic color roles) and a
//! [`Knobs`] struct (animation + behavior tuning). Components pull colors
//! from the palette by role name and never hardcode hex — that's what makes
//! a single component source file render correctly under every theme.
//!
//! Forest is the canonical theme and the only one built by default. Retro,
//! Minimal, and Cyber are feature-gated (`theme-retro`, `theme-minimal`,
//! `theme-cyber`, or `all-themes`).
//!
//! ```no_run
//! use ono::theme::Theme;
//!
//! let theme = Theme::Forest;
//! let palette = theme.palette();
//! let knobs = theme.knobs();
//! // Pass `palette` / `knobs` to components; never branch on `Theme` variants
//! // for visual logic — branch on knob fields instead.
//! ```

use ratatui::style::Color;

/// A bundled palette + knobs preset.
///
/// Use [`Theme::palette`] and [`Theme::knobs`] to get references into the
/// static preset. Feature-gated variants are only constructable when their
/// corresponding cargo feature is enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    /// Canonical theme. Calm greens; slow breathing animation; no scanline,
    /// no glitch. Always available.
    Forest,
    /// Amber-phosphor retro CRT — scanline on, flicker on. `theme-retro`.
    #[cfg(feature = "theme-retro")]
    Retro,
    /// Lipgloss-style quiet monochrome with a single violet accent. `theme-minimal`.
    #[cfg(feature = "theme-minimal")]
    Minimal,
    /// Neon cyberpunk — scanline + glitch on, high-contrast. `theme-cyber`.
    #[cfg(feature = "theme-cyber")]
    Cyber,
}

/// Nine semantic color roles. Every theme fills every role; there are no
/// `Option<Color>` fallbacks. Adding a role is a breaking change and is
/// locked by semver once v0.1.0 ships.
pub struct Palette {
    /// Terminal background. Components should rarely paint this — leave the
    /// user's terminal background showing through by default.
    pub bg: Color,
    /// Card/panel background, one shade above [`Palette::bg`]. Used to lift
    /// a region without borders.
    pub surface: Color,
    /// Default border color for boxes, tables, dividers.
    pub border: Color,
    /// De-emphasized text — labels, timestamps, secondary prose.
    pub dim: Color,
    /// Default body-text color and the dominant hue.
    pub primary: Color,
    /// Highlights, stat values, status dots, "OK" markers.
    pub bright: Color,
    /// Gradient endpoint, progress fill, selection — the hero accent.
    pub accent: Color,
    /// Second accent for two-hue compositions (gradient start, alt badge).
    pub secondary: Color,
    /// Error and warn state only. Do not use as a generic accent.
    pub warn: Color,
}

/// Canonical palette role names. Classes in specs map to these names; the list
/// is kept in sync with `Palette`'s fields and the spec validator checks
/// against it.
#[allow(dead_code)]
pub(crate) const PALETTE_ROLES: &[&str] = &[
    "bg", "surface", "border", "dim", "primary", "bright", "accent", "secondary", "warn",
];

impl Palette {
    #[allow(dead_code)]
    pub(crate) fn role(&self, name: &str) -> Option<Color> {
        Some(match name {
            "bg" => self.bg,
            "surface" => self.surface,
            "border" => self.border,
            "dim" => self.dim,
            "primary" => self.primary,
            "bright" => self.bright,
            "accent" => self.accent,
            "secondary" => self.secondary,
            "warn" => self.warn,
            _ => return None,
        })
    }
}

/// Animation and behavior tuning. Branch on these (not on [`Theme`] variants)
/// when a component needs to vary visual behavior per-theme.
pub struct Knobs {
    /// Seconds per full gradient sweep. Larger = slower breathing.
    pub gradient_period_secs: f32,
    /// Brightness pulse amplitude (0.0 disables).
    pub pulse_amplitude: f32,
    /// If true, components may draw a horizontal scanline overlay.
    pub scanline: bool,
    /// Rows per second the scanline traverses.
    pub scanline_speed_rows_per_sec: f32,
    /// Multiplicative brightness boost applied to pixels on the scanline row.
    pub scanline_boost: f32,
    /// If true, components may inject periodic glitch frames.
    pub glitch: bool,
    /// Minimum seconds between glitch bursts.
    pub glitch_interval_min: f32,
    /// Maximum seconds between glitch bursts.
    pub glitch_interval_max: f32,
    /// Duration of a single glitch burst in milliseconds.
    pub glitch_duration_ms: u64,
    /// If true, idle text may flicker subtly (CRT feel).
    pub idle_flicker: bool,
    /// Typewriter reveal pacing in milliseconds per character.
    pub reveal_ms_per_char: u64,
    /// Spinner glyph sequence, cycled on each tick.
    pub spinner: &'static [char],
    /// Caret blink rate for input-like widgets.
    pub cursor_blink_hz: f32,
    /// If true, components prefer Unicode block glyphs over ASCII for gauges.
    pub gauge_unicode: bool,
}

pub(crate) const SPINNER_FOREST: &[char] = &['◐', '◓', '◑', '◒'];
#[cfg(feature = "theme-retro")]
pub(crate) const SPINNER_BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
#[cfg(feature = "theme-minimal")]
pub(crate) const SPINNER_MINIMAL: &[char] = &['·', '◦', '○', '◦'];
#[cfg(feature = "theme-cyber")]
pub(crate) const SPINNER_CYBER: &[char] = &['╋', '═', '║', '╂', '╀', '┼', '╳', '╱'];

#[cfg(feature = "theme-retro")]
const PALETTE_RETRO: Palette = Palette {
    bg: Color::Rgb(0x0A, 0x07, 0x00),
    surface: Color::Rgb(0x0A, 0x07, 0x00),
    border: Color::Rgb(0x3A, 0x2A, 0x00),
    dim: Color::Rgb(0x8E, 0x6A, 0x14),
    primary: Color::Rgb(0xE8, 0x9A, 0x00),
    bright: Color::Rgb(0xFF, 0xB8, 0x1C),
    accent: Color::Rgb(0xE0, 0x5A, 0x00),
    secondary: Color::Rgb(0xE0, 0x5A, 0x00),
    warn: Color::Rgb(0xE0, 0x5A, 0x00),
};

#[cfg(feature = "theme-minimal")]
const PALETTE_MINIMAL: Palette = Palette {
    bg: Color::Rgb(0x0D, 0x0D, 0x0F),
    surface: Color::Rgb(0x1A, 0x1A, 0x1D),
    border: Color::Rgb(0x2A, 0x2A, 0x2F),
    dim: Color::Rgb(0x88, 0x88, 0x92),
    primary: Color::Rgb(0xB8, 0xB8, 0xC0),
    bright: Color::Rgb(0xD8, 0xD8, 0xDE),
    accent: Color::Rgb(0x6B, 0x4A, 0xE8),
    secondary: Color::Rgb(0x6B, 0x4A, 0xE8),
    warn: Color::Rgb(0xD6, 0x88, 0x0A),
};

#[cfg(feature = "theme-cyber")]
const PALETTE_CYBER: Palette = Palette {
    bg: Color::Rgb(0x05, 0x05, 0x0A),
    surface: Color::Rgb(0x0F, 0x0A, 0x1F),
    border: Color::Rgb(0x2A, 0x1F, 0x4A),
    dim: Color::Rgb(0x7A, 0x6A, 0xA4),
    primary: Color::Rgb(0xB8, 0xB8, 0xD8),
    bright: Color::Rgb(0xE8, 0x1E, 0x74),
    accent: Color::Rgb(0x00, 0xC8, 0xE0),
    secondary: Color::Rgb(0xE8, 0x1E, 0x74),
    warn: Color::Rgb(0xE0, 0x00, 0x2A),
};

const PALETTE_FOREST: Palette = Palette {
    bg: Color::Rgb(0x05, 0x0B, 0x07),
    surface: Color::Rgb(0x0C, 0x16, 0x0F),
    border: Color::Rgb(0x1F, 0x34, 0x22),
    dim: Color::Rgb(0x62, 0x88, 0x64),
    primary: Color::Rgb(0x6F, 0xA8, 0x5E),
    bright: Color::Rgb(0xB6, 0xE0, 0x7A),
    accent: Color::Rgb(0xD8, 0xA7, 0x3A),
    secondary: Color::Rgb(0x3E, 0x8A, 0x5A),
    warn: Color::Rgb(0xC8, 0x6A, 0x1E),
};

#[cfg(feature = "theme-retro")]
const KNOBS_RETRO: Knobs = Knobs {
    gradient_period_secs: 5.0,
    pulse_amplitude: 0.05,
    scanline: true,
    scanline_speed_rows_per_sec: 2.0,
    scanline_boost: 1.10,
    glitch: false,
    glitch_interval_min: 8.0,
    glitch_interval_max: 12.0,
    glitch_duration_ms: 40,
    idle_flicker: true,
    reveal_ms_per_char: 32,
    spinner: SPINNER_BRAILLE,
    cursor_blink_hz: 1.2,
    gauge_unicode: true,
};

#[cfg(feature = "theme-minimal")]
const KNOBS_MINIMAL: Knobs = Knobs {
    gradient_period_secs: 3.0,
    pulse_amplitude: 0.0,
    scanline: false,
    scanline_speed_rows_per_sec: 0.0,
    scanline_boost: 1.0,
    glitch: false,
    glitch_interval_min: 0.0,
    glitch_interval_max: 0.0,
    glitch_duration_ms: 0,
    idle_flicker: false,
    reveal_ms_per_char: 22,
    spinner: SPINNER_MINIMAL,
    cursor_blink_hz: 1.0,
    gauge_unicode: false,
};

#[cfg(feature = "theme-cyber")]
const KNOBS_CYBER: Knobs = Knobs {
    gradient_period_secs: 1.8,
    pulse_amplitude: 0.10,
    scanline: true,
    scanline_speed_rows_per_sec: 8.0,
    scanline_boost: 1.15,
    glitch: true,
    glitch_interval_min: 3.5,
    glitch_interval_max: 7.0,
    glitch_duration_ms: 110,
    idle_flicker: false,
    reveal_ms_per_char: 18,
    spinner: SPINNER_CYBER,
    cursor_blink_hz: 2.0,
    gauge_unicode: true,
};

const KNOBS_FOREST: Knobs = Knobs {
    gradient_period_secs: 7.0,
    pulse_amplitude: 0.04,
    scanline: false,
    scanline_speed_rows_per_sec: 0.0,
    scanline_boost: 1.0,
    glitch: false,
    glitch_interval_min: 0.0,
    glitch_interval_max: 0.0,
    glitch_duration_ms: 0,
    idle_flicker: true,
    reveal_ms_per_char: 28,
    spinner: SPINNER_FOREST,
    cursor_blink_hz: 0.9,
    gauge_unicode: true,
};

impl Theme {
    /// Return a reference to the theme's palette. The palette is a compile-time
    /// static, so this is a cheap lookup.
    pub fn palette(self) -> &'static Palette {
        match self {
            Theme::Forest => &PALETTE_FOREST,
            #[cfg(feature = "theme-retro")]
            Theme::Retro => &PALETTE_RETRO,
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => &PALETTE_MINIMAL,
            #[cfg(feature = "theme-cyber")]
            Theme::Cyber => &PALETTE_CYBER,
        }
    }

    /// Return a reference to the theme's animation + behavior knobs.
    pub fn knobs(self) -> &'static Knobs {
        match self {
            Theme::Forest => &KNOBS_FOREST,
            #[cfg(feature = "theme-retro")]
            Theme::Retro => &KNOBS_RETRO,
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => &KNOBS_MINIMAL,
            #[cfg(feature = "theme-cyber")]
            Theme::Cyber => &KNOBS_CYBER,
        }
    }

    /// Lowercase theme name (e.g. `"forest"`), for logs, CLI flags, and
    /// generated `theme.rs` comments.
    pub fn name(self) -> &'static str {
        match self {
            Theme::Forest => "forest",
            #[cfg(feature = "theme-retro")]
            Theme::Retro => "retro",
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => "minimal",
            #[cfg(feature = "theme-cyber")]
            Theme::Cyber => "cyber",
        }
    }
}

pub(crate) fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

pub(crate) fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (lerp_u8(a.0, b.0, t), lerp_u8(a.1, b.1, t), lerp_u8(a.2, b.2, t))
}

pub(crate) fn scale_rgb(c: (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    let s = |x: u8| (x as f32 * f).clamp(0.0, 255.0).round() as u8;
    (s(c.0), s(c.1), s(c.2))
}

pub(crate) struct Xorshift(pub(crate) u32);

impl Xorshift {
    pub(crate) fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }
    pub(crate) fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}
