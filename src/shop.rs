//! The Shell Shop: cosmetic hats and shell colours for Pinch.
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Hat,
    Color,
}

#[derive(Debug)]
pub struct Item {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub price: u32,
    pub desc: &'static str,
}

pub const ITEMS: &[Item] = &[
    Item {
        id: "color-red",
        name: "Classic Red",
        kind: Kind::Color,
        price: 0,
        desc: "Pinch's natural shell. Always free.",
    },
    Item {
        id: "hat-party",
        name: "Party Hat",
        kind: Kind::Hat,
        price: 30,
        desc: "Every clear is a party.",
    },
    Item {
        id: "color-orange",
        name: "Sunset Orange",
        kind: Kind::Color,
        price: 40,
        desc: "Warm as an evening tide pool.",
    },
    Item {
        id: "hat-sailor",
        name: "Sailor Cap",
        kind: Kind::Hat,
        price: 50,
        desc: "Ready to cargo-ship some code.",
    },
    Item {
        id: "color-teal",
        name: "Ocean Teal",
        kind: Kind::Color,
        price: 60,
        desc: "Blends right into the reef.",
    },
    Item {
        id: "color-purple",
        name: "Deep Purple",
        kind: Kind::Color,
        price: 60,
        desc: "Mysterious deep-sea vibes.",
    },
    Item {
        id: "hat-top",
        name: "Top Hat",
        kind: Kind::Hat,
        price: 80,
        desc: "Very distinguished. Very crustacean.",
    },
    Item {
        id: "hat-pirate",
        name: "Pirate Hat",
        kind: Kind::Hat,
        price: 120,
        desc: "Arr! Borrow, never steal.",
    },
    Item {
        id: "hat-wizard",
        name: "Wizard Hat",
        kind: Kind::Hat,
        price: 160,
        desc: "For mastering the arcane borrow checker.",
    },
    Item {
        id: "color-gold",
        name: "Golden Shell",
        kind: Kind::Color,
        price: 200,
        desc: "Shiny. Show-offy. Earned.",
    },
    Item {
        id: "hat-crown",
        name: "Crown",
        kind: Kind::Hat,
        price: 250,
        desc: "Ruler of the reef.",
    },
];

/// The lines drawn above Pinch's head (each 13 columns wide).
pub fn hat_art(id: Option<&str>) -> &'static [&'static str] {
    match id {
        Some("hat-party") => &["      *      ", "     /_\\     "],
        Some("hat-sailor") => &["    _____    ", "   [_____]   "],
        Some("hat-top") => &["    _____    ", "  __|___|__  "],
        Some("hat-pirate") => &["   _______   ", "  /__ x __\\  "],
        Some("hat-wizard") => &["      /\\  *  ", "   * /__\\    "],
        Some("hat-crown") => &["   .^.^.^.   ", "   |_____|   "],
        _ => &[],
    }
}

pub fn shell_color(id: Option<&str>) -> Color {
    match id {
        Some("color-orange") => Color::Rgb(255, 140, 60),
        Some("color-teal") => Color::Rgb(40, 200, 190),
        Some("color-purple") => Color::Rgb(170, 110, 240),
        Some("color-gold") => Color::Rgb(255, 210, 60),
        _ => Color::LightRed,
    }
}
