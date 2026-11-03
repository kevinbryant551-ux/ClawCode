//! Saved progress: XP, levels, streaks, badges, and code drafts.
//! Stored as JSON in ~/.local/share/clawcode/progress.json (XDG data dir).
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub xp: u32,
    pub completed: BTreeMap<String, Completion>,
    /// Number of check attempts per lesson (kept even after passing).
    pub attempts: BTreeMap<String, u32>,
    /// How many hints the learner has revealed per lesson.
    pub hints_used: BTreeMap<String, u32>,
    /// Lessons where the learner peeked at the solution.
    pub solution_seen: BTreeSet<String>,
    /// The learner's unfinished code, so nothing is lost on quit.
    pub drafts: BTreeMap<String, String>,
    pub streak: u32,
    pub best_streak: u32,
    /// Day number (days since 1970-01-01, local time) of the last passed lesson.
    pub last_day: Option<i64>,
    pub badges: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Completion {
    pub xp_earned: u32,
    pub attempts: u32,
    pub hints: u32,
    pub day: i64,
}

/// What happened when a lesson was passed, for the celebration screen.
#[derive(Debug, Clone, Default)]
pub struct Reward {
    pub xp: u32,
    pub breakdown: Vec<(String, i64)>,
    pub first_time: bool,
    pub leveled_up: Option<u32>,
    pub new_badges: Vec<&'static Badge>,
    pub streak: u32,
}

#[derive(Debug)]
pub struct Badge {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const BADGES: &[Badge] = &[
    Badge {
        id: "first-steps",
        name: "First Steps",
        desc: "Pass your first lesson",
    },
    Badge {
        id: "flawless",
        name: "Flawless",
        desc: "Pass a lesson on the very first try",
    },
    Badge {
        id: "no-peeking",
        name: "No Peeking",
        desc: "Pass 3 lessons without any hints",
    },
    Badge {
        id: "persistent",
        name: "Persistent Pincer",
        desc: "Pass a lesson after 5 or more tries",
    },
    Badge {
        id: "on-a-roll",
        name: "On a Roll",
        desc: "Keep a 3-day streak",
    },
    Badge {
        id: "week-warrior",
        name: "Week Warrior",
        desc: "Keep a 7-day streak",
    },
    Badge {
        id: "borrow-boss",
        name: "Borrow Boss",
        desc: "Conquer the ownership lesson",
    },
    Badge {
        id: "halfway",
        name: "Halfway Reef",
        desc: "Pass half of the lessons",
    },
    Badge {
        id: "graduate",
        name: "Rustacean Graduate",
        desc: "Pass every lesson",
    },
];

pub fn badge(id: &str) -> Option<&'static Badge> {
    BADGES.iter().find(|b| b.id == id)
}

/// XP needed to *reach* each level. Level 1 starts at 0.
const LEVEL_XP: &[u32] = &[0, 100, 250, 450, 700, 1000, 1400, 1900, 2500];
const LEVEL_TITLES: &[&str] = &[
    "Hatchling",
    "Tide-pool Tinkerer",
    "Shell Shaper",
    "Reef Ranger",
    "Borrow Buddy",
    "Ownership Oracle",
    "Trait Tamer",
    "Deep-sea Debugger",
    "Rustacean",
];

pub fn level_for(xp: u32) -> u32 {
    LEVEL_XP.iter().rposition(|&t| xp >= t).unwrap_or(0) as u32 + 1
}

pub fn level_title(level: u32) -> &'static str {
    LEVEL_TITLES[(level as usize - 1).min(LEVEL_TITLES.len() - 1)]
}

/// (xp into this level, xp this level spans). At max level the span is 0.
pub fn level_progress(xp: u32) -> (u32, u32) {
    let lvl = level_for(xp) as usize;
    let start = LEVEL_XP[lvl - 1];
    match LEVEL_XP.get(lvl) {
        Some(&next) => (xp - start, next - start),
        None => (xp - start, 0),
    }
}

/// Today's day number in local time. Uses the `date` command for the UTC offset
/// (so no time-zone library is needed); falls back to UTC.
pub fn today() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let offset = std::process::Command::new("date")
        .arg("+%z")
        .output()
        .ok()
        .and_then(|o| parse_offset(String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or(0);
    (secs + offset).div_euclid(86_400)
}

fn parse_offset(s: &str) -> Option<i64> {
    // "+0130" or "-0500"
    if s.len() != 5 {
        return None;
    }
    let sign = if s.starts_with('-') { -1 } else { 1 };
    let h: i64 = s[1..3].parse().ok()?;
    let m: i64 = s[3..5].parse().ok()?;
    Some(sign * (h * 3600 + m * 60))
}

impl Progress {
    pub fn path() -> PathBuf {
        if let Ok(p) = std::env::var("CLAWCODE_PROGRESS") {
            return PathBuf::from(p);
        }
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clawcode")
            .join("progress.json")
    }

    pub fn load() -> Progress {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write to a temp file then rename, so a crash never leaves half a file.
        let tmp = p.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(tmp, p)
    }

    pub fn is_done(&self, id: &str) -> bool {
        self.completed.contains_key(id)
    }

    pub fn level(&self) -> u32 {
        level_for(self.xp)
    }

    /// Records a pass and computes the XP reward.
    ///
    /// Rules: base XP, +50% for a first-try pass, -10% of base per hint (the
    /// first-try bonus still counts), half XP if the solution was revealed,
    /// +5 XP per streak day (max +25). Replaying a finished lesson gives no XP.
    pub fn record_pass(
        &mut self,
        id: &str,
        base_xp: u32,
        total_lessons: usize,
        day: i64,
    ) -> Reward {
        let attempts = *self.attempts.get(id).unwrap_or(&1);
        let hints = *self.hints_used.get(id).unwrap_or(&0);
        let mut reward = Reward {
            streak: self.streak,
            ..Default::default()
        };

        // Streak: counts days in a row on which you passed at least one lesson.
        match self.last_day {
            Some(d) if d == day => {}
            Some(d) if d == day - 1 => self.streak += 1,
            _ => self.streak = 1,
        }
        self.last_day = Some(day);
        self.best_streak = self.best_streak.max(self.streak);
        reward.streak = self.streak;

        if self.is_done(id) {
            return reward; // replay: no XP, but the streak still counts
        }
        reward.first_time = true;

        let base = base_xp as i64;
        let mut parts: Vec<(String, i64)> = vec![("Lesson cleared".into(), base)];
        if attempts == 1 {
            parts.push(("First-try bonus".into(), base / 2));
        }
        if hints > 0 {
            parts.push((format!("Hints used ({hints})"), -(base / 10) * hints as i64));
        }
        if self.solution_seen.contains(id) {
            parts.push(("Peeked at solution".into(), -(base / 2)));
        }
        if self.streak > 1 {
            parts.push((
                format!("{}-day streak", self.streak),
                (5 * self.streak as i64).min(25),
            ));
        }
        let total: i64 = parts.iter().map(|(_, v)| v).sum();
        let earned = total.max(base / 4) as u32;

        let before = self.level();
        self.xp += earned;
        let after = self.level();
        if after > before {
            reward.leveled_up = Some(after);
        }
        reward.xp = earned;
        reward.breakdown = parts;
        self.completed.insert(
            id.to_string(),
            Completion {
                xp_earned: earned,
                attempts,
                hints,
                day,
            },
        );
        self.drafts.remove(id);

        // Badges
        let no_hint_passes = self.completed.values().filter(|c| c.hints == 0).count();
        let mut award = |b: &str, cond: bool| {
            if cond
                && self.badges.insert(b.to_string())
                && let Some(b) = badge(b)
            {
                reward.new_badges.push(b);
            }
        };
        let done = self.completed.len();
        award("first-steps", done >= 1);
        award("flawless", attempts == 1 && hints == 0);
        award("no-peeking", no_hint_passes >= 3);
        award("persistent", attempts >= 5);
        award("on-a-roll", self.streak >= 3);
        award("week-warrior", self.streak >= 7);
        award("borrow-boss", id.contains("ownership"));
        award("halfway", done * 2 >= total_lessons);
        award("graduate", done >= total_lessons);
        reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels() {
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(99), 1);
        assert_eq!(level_for(100), 2);
        assert_eq!(level_for(99_999), LEVEL_XP.len() as u32);
        assert_eq!(level_progress(150), (50, 150));
    }

    #[test]
    fn first_try_bonus_and_badges() {
        let mut p = Progress::default();
        p.attempts.insert("01-hello".into(), 1);
        let r = p.record_pass("01-hello", 50, 10, 100);
        assert_eq!(r.xp, 75);
        assert!(r.first_time);
        assert!(p.badges.contains("first-steps") && p.badges.contains("flawless"));
        // Replay gives nothing.
        assert_eq!(p.record_pass("01-hello", 50, 10, 100).xp, 0);
    }

    #[test]
    fn hints_reduce_xp_and_streaks_grow() {
        let mut p = Progress::default();
        p.attempts.insert("a".into(), 3);
        p.hints_used.insert("a".into(), 2);
        assert_eq!(p.record_pass("a", 100, 10, 10).xp, 80);
        p.attempts.insert("b".into(), 2);
        let r = p.record_pass("b", 100, 10, 11);
        assert_eq!(r.streak, 2);
        assert_eq!(r.xp, 110);
        p.attempts.insert("c".into(), 2);
        assert_eq!(
            p.record_pass("c", 100, 10, 20).streak,
            1,
            "a gap resets the streak"
        );
    }

    #[test]
    fn offset_parsing() {
        assert_eq!(parse_offset("-0500"), Some(-18000));
        assert_eq!(parse_offset("+0130"), Some(5400));
        assert_eq!(parse_offset("bad"), None);
    }
}
