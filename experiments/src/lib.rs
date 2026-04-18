use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Cyber,
    #[cfg(feature = "theme-retro")]
    Retro,
    #[cfg(feature = "theme-minimal")]
    Minimal,
    #[cfg(feature = "theme-forest")]
    Forest,
}

pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub border: Color,
    pub dim: Color,
    pub primary: Color,
    pub bright: Color,
    pub accent: Color,
    pub accent2: Color,
    pub warn: Color,
}

pub struct Knobs {
    pub gradient_period_secs: f32,
    pub pulse_amplitude: f32,
    pub scanline: bool,
    pub scanline_speed_rows_per_sec: f32,
    pub scanline_boost: f32,
    pub glitch: bool,
    pub glitch_interval_min: f32,
    pub glitch_interval_max: f32,
    pub glitch_duration_ms: u64,
    pub idle_flicker: bool,
    pub reveal_ms_per_char: u64,
    pub spinner: &'static [char],
    pub cursor_blink_hz: f32,
    pub gauge_unicode: bool,
}

pub const SPINNER_CYBER: &[char] = &['╋', '═', '║', '╂', '╀', '┼', '╳', '╱'];
#[cfg(feature = "theme-retro")]
pub const SPINNER_BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
#[cfg(feature = "theme-minimal")]
pub const SPINNER_MINIMAL: &[char] = &['·', '◦', '○', '◦'];
#[cfg(feature = "theme-forest")]
pub const SPINNER_FOREST: &[char] = &['◐', '◓', '◑', '◒'];

pub const GLITCH_CHARS: &[char] = &['░', '▒', '▓', '█', '▚', '▞', '╳', '¦', '§', '∎', '◈'];

#[cfg(feature = "theme-retro")]
const PALETTE_RETRO: Palette = Palette {
    bg: Color::Rgb(0x0A, 0x07, 0x00),
    surface: Color::Rgb(0x0A, 0x07, 0x00),
    border: Color::Rgb(0x3A, 0x2A, 0x00),
    dim: Color::Rgb(0x8E, 0x6A, 0x14),
    primary: Color::Rgb(0xE8, 0x9A, 0x00),
    bright: Color::Rgb(0xFF, 0xB8, 0x1C),
    accent: Color::Rgb(0xE0, 0x5A, 0x00),
    accent2: Color::Rgb(0xE0, 0x5A, 0x00),
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
    accent2: Color::Rgb(0x6B, 0x4A, 0xE8),
    warn: Color::Rgb(0xD6, 0x88, 0x0A),
};

const PALETTE_CYBER: Palette = Palette {
    bg: Color::Rgb(0x05, 0x05, 0x0A),
    surface: Color::Rgb(0x0F, 0x0A, 0x1F),
    border: Color::Rgb(0x2A, 0x1F, 0x4A),
    dim: Color::Rgb(0x7A, 0x6A, 0xA4),
    primary: Color::Rgb(0xB8, 0xB8, 0xD8),
    bright: Color::Rgb(0xE8, 0x1E, 0x74),
    accent: Color::Rgb(0x00, 0xC8, 0xE0),
    accent2: Color::Rgb(0xE8, 0x1E, 0x74),
    warn: Color::Rgb(0xE0, 0x00, 0x2A),
};

#[cfg(feature = "theme-forest")]
const PALETTE_FOREST: Palette = Palette {
    bg: Color::Rgb(0x05, 0x0B, 0x07),
    surface: Color::Rgb(0x0C, 0x16, 0x0F),
    border: Color::Rgb(0x1F, 0x34, 0x22),
    dim: Color::Rgb(0x62, 0x88, 0x64),
    primary: Color::Rgb(0x6F, 0xA8, 0x5E),
    bright: Color::Rgb(0xB6, 0xE0, 0x7A),
    accent: Color::Rgb(0xD8, 0xA7, 0x3A),
    accent2: Color::Rgb(0x3E, 0x8A, 0x5A),
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

#[cfg(feature = "theme-forest")]
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
    pub fn palette(self) -> &'static Palette {
        match self {
            Theme::Cyber => &PALETTE_CYBER,
            #[cfg(feature = "theme-retro")]
            Theme::Retro => &PALETTE_RETRO,
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => &PALETTE_MINIMAL,
            #[cfg(feature = "theme-forest")]
            Theme::Forest => &PALETTE_FOREST,
        }
    }

    pub fn knobs(self) -> &'static Knobs {
        match self {
            Theme::Cyber => &KNOBS_CYBER,
            #[cfg(feature = "theme-retro")]
            Theme::Retro => &KNOBS_RETRO,
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => &KNOBS_MINIMAL,
            #[cfg(feature = "theme-forest")]
            Theme::Forest => &KNOBS_FOREST,
        }
    }

    pub fn gradient(self, phase: f32) -> (u8, u8, u8) {
        let p = phase.rem_euclid(1.0);
        let tri = if p < 0.5 { p * 2.0 } else { 2.0 - p * 2.0 };
        match self {
            Theme::Cyber => lerp_rgb((0xFF, 0x2E, 0x88), (0x00, 0xF0, 0xFF), tri),
            #[cfg(feature = "theme-retro")]
            Theme::Retro => {
                if tri < 0.5 {
                    lerp_rgb((0xFF, 0x6A, 0x00), (0xFF, 0xB0, 0x00), tri * 2.0)
                } else {
                    lerp_rgb((0xFF, 0xB0, 0x00), (0xFF, 0xD9, 0x66), (tri - 0.5) * 2.0)
                }
            }
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => lerp_rgb((0xE8, 0xE8, 0xEC), (0x7C, 0x5C, 0xFF), tri * 0.18),
            #[cfg(feature = "theme-forest")]
            Theme::Forest => {
                if tri < 0.5 {
                    lerp_rgb((0x3E, 0x8A, 0x5A), (0x6F, 0xA8, 0x5E), tri * 2.0)
                } else {
                    lerp_rgb((0x6F, 0xA8, 0x5E), (0xB6, 0xE0, 0x7A), (tri - 0.5) * 2.0)
                }
            }
        }
    }

    pub fn parse_from_args() -> Self {
        let mut args = std::env::args().skip(1);
        while let Some(a) = args.next() {
            if a == "--theme" {
                if let Some(v) = args.next() {
                    return match v.as_str() {
                        "cyber" => Theme::Cyber,
                        #[cfg(feature = "theme-retro")]
                        "retro" => Theme::Retro,
                        #[cfg(feature = "theme-minimal")]
                        "minimal" => Theme::Minimal,
                        #[cfg(feature = "theme-forest")]
                        "forest" => Theme::Forest,
                        other => {
                            eprintln!(
                                "theme {:?} not enabled in this build. rebuild with `--features theme-{}`. using cyber.",
                                other, other
                            );
                            Theme::Cyber
                        }
                    };
                }
            }
        }
        Theme::Cyber
    }

    pub fn name(self) -> &'static str {
        match self {
            Theme::Cyber => "cyber",
            #[cfg(feature = "theme-retro")]
            Theme::Retro => "retro",
            #[cfg(feature = "theme-minimal")]
            Theme::Minimal => "minimal",
            #[cfg(feature = "theme-forest")]
            Theme::Forest => "forest",
        }
    }
}

pub fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

pub fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (lerp_u8(a.0, b.0, t), lerp_u8(a.1, b.1, t), lerp_u8(a.2, b.2, t))
}

pub fn scale_rgb(c: (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    let s = |x: u8| (x as f32 * f).clamp(0.0, 255.0).round() as u8;
    (s(c.0), s(c.1), s(c.2))
}

pub fn rgb(c: (u8, u8, u8)) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

pub fn color_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

pub struct Xorshift(pub u32);

impl Xorshift {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    pub fn unit(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}
