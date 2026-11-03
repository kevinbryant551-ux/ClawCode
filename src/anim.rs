//! Small animation helpers: a tiny random number generator, confetti and bubbles.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

/// xorshift64*: tiny, fast, and good enough for sparkles and daily picks.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn from_time() -> Rng {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Rng::new(t)
    }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// A number in 0..n.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
    /// A float in 0.0..1.0.
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
}

const CONFETTI: &[char] = &['*', '+', '•', '✦', '░', '▪', '~', 'o'];
const COLORS: &[Color] = &[
    Color::LightRed,
    Color::Yellow,
    Color::LightCyan,
    Color::LightMagenta,
    Color::LightGreen,
    Color::White,
];

#[derive(Debug, Clone)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ch: char,
    color: Color,
    life: u32,
}

#[derive(Debug, Default)]
pub struct Confetti {
    parts: Vec<Particle>,
}

impl Confetti {
    /// Bursts confetti from a few points along the top of `area`.
    pub fn burst(&mut self, area: Rect, rng: &mut Rng, amount: usize) {
        for i in 0..amount {
            let origin = area.x as f32 + area.width as f32 * ((i % 5) as f32 + 0.5) / 5.0;
            self.parts.push(Particle {
                x: origin,
                y: area.y as f32 + area.height as f32 * 0.35,
                vx: (rng.unit() - 0.5) * 3.0,
                vy: -rng.unit() * 1.6 - 0.3,
                ch: CONFETTI[rng.below(CONFETTI.len() as u64) as usize],
                color: COLORS[rng.below(COLORS.len() as u64) as usize],
                life: 40 + rng.below(50) as u32,
            });
        }
    }

    pub fn tick(&mut self) {
        for p in &mut self.parts {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.08; // gravity
            p.vx *= 0.97; // air
            p.life = p.life.saturating_sub(1);
        }
        self.parts.retain(|p| p.life > 0);
    }

    #[cfg(test)]
    pub fn active(&self) -> bool {
        !self.parts.is_empty()
    }

    pub fn clear(&mut self) {
        self.parts.clear();
    }

    pub fn draw(&self, buf: &mut Buffer, area: Rect) {
        for p in &self.parts {
            if p.x < area.x as f32 || p.y < area.y as f32 {
                continue;
            }
            let (x, y) = (p.x as u16, p.y as u16);
            if x < area.right()
                && y < area.bottom()
                && let Some(cell) = buf.cell_mut((x, y))
            {
                cell.set_char(p.ch).set_style(Style::new().fg(p.color));
            }
        }
    }
}

/// Bubbles that drift up behind the title screen.
#[derive(Debug, Default)]
pub struct Bubbles {
    parts: Vec<(f32, f32, f32, char)>,
}

impl Bubbles {
    pub fn tick(&mut self, area: Rect, rng: &mut Rng) {
        if rng.below(3) == 0 && area.width > 0 {
            let ch = ['o', '°', '.', 'O'][rng.below(4) as usize];
            self.parts.push((
                area.x as f32 + rng.below(area.width as u64) as f32,
                area.bottom() as f32 - 1.0,
                0.15 + rng.unit() * 0.35,
                ch,
            ));
        }
        for b in &mut self.parts {
            b.1 -= b.2;
            b.0 += (b.1 * 0.7).sin() * 0.15;
        }
        self.parts.retain(|b| b.1 > area.y as f32);
    }

    pub fn draw(&self, buf: &mut Buffer, area: Rect) {
        for (x, y, _, ch) in &self.parts {
            let (x, y) = (*x as u16, *y as u16);
            if x >= area.x
                && x < area.right()
                && y >= area.y
                && y < area.bottom()
                && let Some(cell) = buf.cell_mut((x, y))
                && cell.symbol() == " "
            {
                cell.set_char(*ch)
                    .set_style(Style::new().fg(Color::Rgb(70, 110, 160)));
            }
        }
    }
}

/// Picks `n` distinct items out of `0..len`, the same way for the same seed.
pub fn pick(len: usize, n: usize, seed: u64) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..len).collect();
    let mut rng = Rng::new(seed);
    for i in (1..idx.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        idx.swap(i, j);
    }
    idx.truncate(n);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_is_stable_and_distinct() {
        let a = pick(20, 5, 7);
        assert_eq!(a, pick(20, 5, 7));
        let mut s = a.clone();
        s.sort();
        s.dedup();
        assert_eq!(s.len(), 5);
        assert_eq!(pick(3, 5, 1).len(), 3);
    }

    #[test]
    fn confetti_falls_and_ends() {
        let mut c = Confetti::default();
        let mut r = Rng::new(1);
        c.burst(Rect::new(0, 0, 80, 24), &mut r, 50);
        assert!(c.active());
        for _ in 0..200 {
            c.tick();
        }
        assert!(!c.active());
    }
}
