//! Saved progress: XP, levels, streaks, shells, badges, quiz results and code drafts.
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
    /// Day number (days since 1970-01-01, local time) of the last active day.
    pub last_day: Option<i64>,
    pub badges: BTreeSet<String>,

    /// Lessons whose reading cards have been read through once.
    pub learned: BTreeSet<String>,
    /// Quizzes already answered correctly, as "lesson-id#index".
    pub quizzes_done: BTreeSet<String>,
    /// Current run of correct first answers, and the best ever.
    pub combo: u32,
    pub best_combo: u32,

    /// Shells are the in-game currency for the shop.
    pub shells: u32,
    pub owned: BTreeSet<String>,
    pub hat: Option<String>,
    pub color: Option<String>,

    /// Day the Daily Tide was last finished.
    pub daily_day: Option<i64>,
    pub stats: Stats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub runs: u32,
    pub compile_errors: u32,
    pub quizzes_right: u32,
    pub quizzes_answered: u32,
    pub bosses_defeated: u32,
    pub dailies: u32,
    pub shells_earned: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Completion {
    pub xp_earned: u32,
    pub attempts: u32,
    pub hints: u32,
    pub day: i64,
}

/// What happened when something was won, for the celebration screen.
#[derive(Debug, Clone, Default)]
pub struct Reward {
    pub xp: u32,
    pub shells: u32,
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

macro_rules! badges {
    ($($id:literal, $name:literal, $desc:literal;)*) => {
        pub const BADGES: &[Badge] = &[$(Badge { id: $id, name: $name, desc: $desc }),*];
    };
}

badges! {
    "first-steps", "First Steps", "Pass your first lesson";
    "flawless", "Flawless", "Pass a lesson on the very first try";
    "no-peeking", "No Peeking", "Pass 3 lessons without any tips";
    "persistent", "Persistent Pincer", "Pass a lesson after 5 or more tries";
    "on-a-roll", "On a Roll", "Keep a 3-day streak";
    "week-warrior", "Week Warrior", "Keep a 7-day streak";
    "borrow-boss", "Borrow Boss", "Conquer the ownership lesson";
    "boss-slayer", "Boss Slayer", "Defeat your first boss";
    "monster-hunter", "Monster Hunter", "Defeat 3 bosses";
    "quiz-whiz", "Quiz Whiz", "Answer 10 quizzes right on the first try";
    "combo-5", "Combo Crab", "Reach a 5x quiz combo";
    "combo-10", "Unstoppable", "Reach a 10x quiz combo";
    "daily-diver", "Daily Diver", "Finish 3 Daily Tides";
    "perfect-tide", "Perfect Tide", "Get every Daily Tide question right";
    "fashionista", "Fashionista", "Buy something in the Shell Shop";
    "collector", "Collector", "Own 5 shop items";
    "error-veteran", "Error Veteran", "Survive 25 compiler errors (they make you stronger!)";
    "halfway", "Halfway Reef", "Pass half of the lessons";
    "graduate", "Rustacean Graduate", "Pass every lesson";
}

pub fn badge(id: &str) -> Option<&'static Badge> {
    BADGES.iter().find(|b| b.id == id)
}

/// XP needed to *reach* each level. Level 1 starts at 0.
const LEVEL_XP: &[u32] = &[0, 100, 250, 450, 700, 1000, 1400, 1900, 2500, 3200, 4000];
const LEVEL_TITLES: &[&str] = &[
    "Hatchling",
    "Tide-pool Tinkerer",
    "Shell Shaper",
    "Reef Ranger",
    "Borrow Buddy",
    "Ownership Oracle",
    "Trait Tamer",
    "Iterator Ace",
    "Deep-sea Debugger",
    "Leviathan Wrangler",
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

    /// Streak: counts days in a row with at least one passed lesson or Daily Tide.
    pub fn touch_streak(&mut self, day: i64) {
        match self.last_day {
            Some(d) if d == day => {}
            Some(d) if d == day - 1 => self.streak += 1,
            _ => self.streak = 1,
        }
        self.last_day = Some(day);
        self.best_streak = self.best_streak.max(self.streak);
    }

    /// The streak as it should be shown today (it's 0 if a day was missed).
    pub fn live_streak(&self, day: i64) -> u32 {
        match self.last_day {
            Some(d) if d >= day - 1 => self.streak,
            _ => 0,
        }
    }

    fn add_shells(&mut self, n: u32) {
        self.shells += n;
        self.stats.shells_earned += n;
    }

    fn add_xp(&mut self, n: u32, reward: &mut Reward) {
        let before = self.level();
        self.xp += n;
        if self.level() > before {
            reward.leveled_up = Some(self.level());
        }
    }

    /// Awards a badge if `cond` holds and it's new.
    fn award(&mut self, id: &str, cond: bool, reward: &mut Reward) {
        if cond
            && self.badges.insert(id.to_string())
            && let Some(b) = badge(id)
        {
            reward.new_badges.push(b);
        }
    }

    fn general_badges(&mut self, r: &mut Reward) {
        let s = self.stats.clone();
        self.award("on-a-roll", self.streak >= 3, r);
        self.award("week-warrior", self.streak >= 7, r);
        self.award("quiz-whiz", s.quizzes_right >= 10, r);
        self.award("combo-5", self.best_combo >= 5, r);
        self.award("combo-10", self.best_combo >= 10, r);
        self.award("error-veteran", s.compile_errors >= 25, r);
        self.award("daily-diver", s.dailies >= 3, r);
        self.award("boss-slayer", s.bosses_defeated >= 1, r);
        self.award("monster-hunter", s.bosses_defeated >= 3, r);
        self.award("fashionista", !self.owned.is_empty(), r);
        self.award("collector", self.owned.len() >= 5, r);
    }

    /// Records a pass and computes the reward.
    ///
    /// XP rules: base XP, +50% for a first-try pass, -10% of base per tip (the
    /// first-try bonus still counts), -50% of base if the solution was revealed,
    /// +5 XP per streak day (max +25), never less than 25% of base, +50% of base
    /// for defeating a boss. Shells: a quarter of the XP earned (more for bosses).
    /// Replaying a finished lesson gives no XP or shells.
    pub fn record_pass(
        &mut self,
        id: &str,
        base_xp: u32,
        boss: bool,
        total_lessons: usize,
        day: i64,
    ) -> Reward {
        let attempts = *self.attempts.get(id).unwrap_or(&1);
        let hints = *self.hints_used.get(id).unwrap_or(&0);
        let mut reward = Reward::default();
        self.touch_streak(day);
        reward.streak = self.streak;

        if self.is_done(id) {
            return reward; // replay: no XP, but the streak still counts
        }
        reward.first_time = true;

        let base = base_xp as i64;
        let mut parts: Vec<(String, i64)> = vec![(
            if boss {
                "Boss defeated"
            } else {
                "Lesson cleared"
            }
            .into(),
            base,
        )];
        if attempts == 1 {
            parts.push(("First-try bonus".into(), base / 2));
        }
        if boss {
            parts.push(("Boss bounty".into(), base / 2));
        }
        if hints > 0 {
            parts.push((format!("Tips used ({hints})"), -(base / 10) * hints as i64));
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
        self.add_xp(earned, &mut reward);
        let shells = earned / 4 + if boss { 25 } else { 0 };
        self.add_shells(shells);
        reward.xp = earned;
        reward.shells = shells;
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
        if boss {
            self.stats.bosses_defeated += 1;
        }

        let no_hint_passes = self.completed.values().filter(|c| c.hints == 0).count();
        let done = self.completed.len();
        let r = &mut reward;
        self.award("first-steps", done >= 1, r);
        self.award("flawless", attempts == 1 && hints == 0, r);
        self.award("no-peeking", no_hint_passes >= 3, r);
        self.award("persistent", attempts >= 5, r);
        self.award("borrow-boss", id.contains("ownership"), r);
        self.award("halfway", done * 2 >= total_lessons, r);
        self.award("graduate", done >= total_lessons, r);
        self.general_badges(r);
        reward
    }

    /// Records a quiz answer. Returns (xp, shells) gained. Only the first answer
    /// to a quiz that has never been solved counts for rewards and combos.
    pub fn record_quiz(
        &mut self,
        key: &str,
        first_answer_correct: bool,
        rewardable: bool,
    ) -> (u32, u32, Reward) {
        let mut r = Reward::default();
        if !rewardable || self.quizzes_done.contains(key) {
            return (0, 0, r);
        }
        self.stats.quizzes_answered += 1;
        if first_answer_correct {
            self.stats.quizzes_right += 1;
            self.combo += 1;
            self.best_combo = self.best_combo.max(self.combo);
        } else {
            self.combo = 0;
        }
        let (xp, shells) = if first_answer_correct {
            (10, 3 + self.combo.min(10))
        } else {
            (0, 1)
        };
        self.quizzes_done.insert(key.to_string());
        self.add_xp(xp, &mut r);
        self.add_shells(shells);
        r.xp = xp;
        r.shells = shells;
        self.general_badges(&mut r);
        (xp, shells, r)
    }

    /// Finishes today's Daily Tide.
    pub fn record_daily(&mut self, right: u32, total: u32, day: i64) -> Reward {
        let mut r = Reward {
            first_time: true,
            ..Default::default()
        };
        self.daily_day = Some(day);
        self.touch_streak(day);
        r.streak = self.streak;
        self.stats.dailies += 1;
        let mut parts = vec![
            ("Showed up for the tide".to_string(), 5),
            ("Correct answers".to_string(), 8 * right as i64),
        ];
        if right == total {
            parts.push(("Perfect tide!".into(), 20));
        }
        if self.streak > 1 {
            parts.push((
                format!("{}-day streak", self.streak),
                (5 * self.streak as i64).min(25),
            ));
        }
        let xp: i64 = parts.iter().map(|(_, v)| v).sum();
        self.add_xp(xp as u32, &mut r);
        let shells = 2 + 3 * right + if right == total { 10 } else { 0 };
        self.add_shells(shells);
        r.xp = xp as u32;
        r.shells = shells;
        r.breakdown = parts;
        self.award("perfect-tide", right == total, &mut r);
        self.general_badges(&mut r);
        r
    }

    /// Buys or equips a shop item. Returns a message for Pinch to say.
    pub fn shop_select(&mut self, item: &crate::shop::Item) -> (bool, String, Reward) {
        let mut r = Reward::default();
        if !self.owned.contains(item.id) && item.price > 0 {
            if self.shells < item.price {
                return (
                    false,
                    format!(
                        "You need {} more shells for the {}. Pass lessons and quizzes to earn more!",
                        item.price - self.shells,
                        item.name
                    ),
                    r,
                );
            }
            self.shells -= item.price;
            self.owned.insert(item.id.to_string());
            self.general_badges(&mut r);
        }
        let slot = match item.kind {
            crate::shop::Kind::Hat => &mut self.hat,
            crate::shop::Kind::Color => &mut self.color,
        };
        if slot.as_deref() == Some(item.id) {
            *slot = None;
            (true, format!("Took off the {}.", item.name), r)
        } else {
            *slot = Some(item.id.to_string());
            (true, format!("Looking sharp in the {}!", item.name), r)
        }
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
        let r = p.record_pass("01-hello", 50, false, 10, 100);
        assert_eq!(r.xp, 75);
        assert_eq!(r.shells, 18);
        assert!(r.first_time);
        assert!(p.badges.contains("first-steps") && p.badges.contains("flawless"));
        // Replay gives nothing.
        assert_eq!(p.record_pass("01-hello", 50, false, 10, 100).xp, 0);
    }

    #[test]
    fn hints_reduce_xp_and_streaks_grow() {
        let mut p = Progress::default();
        p.attempts.insert("a".into(), 3);
        p.hints_used.insert("a".into(), 2);
        assert_eq!(p.record_pass("a", 100, false, 10, 10).xp, 80);
        p.attempts.insert("b".into(), 2);
        let r = p.record_pass("b", 100, false, 10, 11);
        assert_eq!(r.streak, 2);
        assert_eq!(r.xp, 110);
        p.attempts.insert("c".into(), 2);
        assert_eq!(
            p.record_pass("c", 100, false, 10, 20).streak,
            1,
            "a gap resets the streak"
        );
        assert_eq!(p.live_streak(22), 0, "missing a day shows a 0 streak");
    }

    #[test]
    fn bosses_pay_more() {
        let mut p = Progress::default();
        p.attempts.insert("boss".into(), 2);
        let r = p.record_pass("boss", 100, true, 10, 1);
        assert_eq!(r.xp, 150);
        assert_eq!(r.shells, 150 / 4 + 25);
        assert!(p.badges.contains("boss-slayer"));
    }

    #[test]
    fn quiz_combos() {
        let mut p = Progress::default();
        for i in 0..5 {
            p.record_quiz(&format!("l#{i}"), true, true);
        }
        assert_eq!(p.combo, 5);
        assert!(p.badges.contains("combo-5"));
        assert_eq!(
            p.record_quiz("l#0", true, true).0,
            0,
            "solved quizzes give nothing again"
        );
        p.record_quiz("l#9", false, true);
        assert_eq!(p.combo, 0);
    }

    #[test]
    fn shop_buy_and_equip() {
        let item = &crate::shop::ITEMS[1];
        let mut p = Progress::default();
        assert!(!p.shop_select(item).0, "can't afford with 0 shells");
        p.shells = 1000;
        assert!(p.shop_select(item).0);
        assert_eq!(p.shells, 1000 - item.price);
        assert!(p.badges.contains("fashionista"));
        p.shop_select(item); // unequip, no charge
        assert_eq!(p.shells, 1000 - item.price);
    }

    #[test]
    fn offset_parsing() {
        assert_eq!(parse_offset("-0500"), Some(-18000));
        assert_eq!(parse_offset("+0130"), Some(5400));
        assert_eq!(parse_offset("bad"), None);
    }
}
