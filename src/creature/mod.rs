pub mod art;
pub mod persistence;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main Tui creature that accompanies the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub name: String,
    pub species: CreatureSpecies,
    pub level: u32,
    pub experience: u64,
    pub points: u32,
    pub stats: CreatureStats,
    pub appearance: CreatureAppearance,
    pub unlocked_skills: Vec<String>,
    pub active_skills: Vec<String>,
    pub unlocked_outfits: Vec<String>,
    pub equipped_outfit: Option<String>,
    pub unlocked_emotes: Vec<String>,
    pub mood: CreatureMood,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub total_sessions: u64,
    pub total_time_seconds: u64,
}

impl Default for Creature {
    fn default() -> Self {
        Self {
            name: "Tui".to_string(),
            species: CreatureSpecies::default(),
            level: 1,
            experience: 0,
            points: 0,
            stats: CreatureStats::default(),
            appearance: CreatureAppearance::default(),
            unlocked_skills: vec!["greeting".to_string()],
            active_skills: vec!["greeting".to_string()],
            unlocked_outfits: vec!["default".to_string()],
            equipped_outfit: Some("default".to_string()),
            unlocked_emotes: vec!["wave".to_string(), "happy".to_string()],
            mood: CreatureMood::Happy,
            created_at: Utc::now(),
            last_seen: Utc::now(),
            total_sessions: 0,
            total_time_seconds: 0,
        }
    }
}

impl Creature {
    pub fn new(name: String, species: CreatureSpecies) -> Self {
        Self {
            name,
            species,
            ..Default::default()
        }
    }

    /// Calculate XP needed for next level (exponential curve)
    pub fn xp_for_level(level: u32) -> u64 {
        // Base 100 XP, grows exponentially
        (100.0 * (1.5_f64).powi(level as i32 - 1)) as u64
    }

    /// Get XP needed to reach the next level
    pub fn xp_to_next_level(&self) -> u64 {
        Self::xp_for_level(self.level + 1).saturating_sub(self.experience)
    }

    /// Get total XP needed for current level
    pub fn xp_for_current_level(&self) -> u64 {
        if self.level == 1 {
            0
        } else {
            Self::xp_for_level(self.level)
        }
    }

    /// Get progress percentage to next level (0.0 - 1.0)
    pub fn level_progress(&self) -> f64 {
        let current_level_xp = self.xp_for_current_level();
        let next_level_xp = Self::xp_for_level(self.level + 1);
        let xp_in_level = self.experience.saturating_sub(current_level_xp);
        let xp_needed = next_level_xp - current_level_xp;
        (xp_in_level as f64) / (xp_needed as f64)
    }

    /// Add experience and handle level ups
    pub fn add_experience(&mut self, xp: u64) -> Vec<LevelUpReward> {
        self.experience += xp;
        let mut rewards = Vec::new();

        while self.experience >= Self::xp_for_level(self.level + 1) {
            self.level += 1;
            let reward = self.calculate_level_reward();
            self.points += reward.points;

            // Unlock any skills/outfits/emotes for this level
            for skill in &reward.unlocked_skills {
                if !self.unlocked_skills.contains(skill) {
                    self.unlocked_skills.push(skill.clone());
                }
            }
            for outfit in &reward.unlocked_outfits {
                if !self.unlocked_outfits.contains(outfit) {
                    self.unlocked_outfits.push(outfit.clone());
                }
            }
            for emote in &reward.unlocked_emotes {
                if !self.unlocked_emotes.contains(emote) {
                    self.unlocked_emotes.push(emote.clone());
                }
            }

            rewards.push(reward);
        }

        rewards
    }

    fn calculate_level_reward(&self) -> LevelUpReward {
        let points = 5 + (self.level / 5) * 2; // More points at higher levels

        let mut unlocked_skills = Vec::new();
        let mut unlocked_outfits = Vec::new();
        let mut unlocked_emotes = Vec::new();

        // Level-based unlocks
        match self.level {
            2 => unlocked_emotes.push("excited".to_string()),
            3 => unlocked_skills.push("news_digest".to_string()),
            5 => {
                unlocked_outfits.push("hacker".to_string());
                unlocked_emotes.push("cool".to_string());
            }
            7 => unlocked_skills.push("stock_alert".to_string()),
            10 => {
                unlocked_outfits.push("wizard".to_string());
                unlocked_skills.push("speed_read".to_string());
            }
            15 => {
                unlocked_outfits.push("ninja".to_string());
                unlocked_emotes.push("stealth".to_string());
            }
            20 => {
                unlocked_outfits.push("astronaut".to_string());
                unlocked_skills.push("cosmic_insight".to_string());
            }
            25 => unlocked_outfits.push("robot".to_string()),
            30 => {
                unlocked_outfits.push("dragon".to_string());
                unlocked_skills.push("fire_breath".to_string());
            }
            50 => {
                unlocked_outfits.push("legendary".to_string());
                unlocked_skills.push("omniscience".to_string());
            }
            _ => {}
        }

        LevelUpReward {
            level: self.level,
            points,
            unlocked_skills,
            unlocked_outfits,
            unlocked_emotes,
        }
    }

    /// Record a session start
    pub fn start_session(&mut self) {
        self.total_sessions += 1;
        self.last_seen = Utc::now();

        // Mood based on absence
        let hours_away = (Utc::now() - self.last_seen).num_hours();
        self.mood = if hours_away > 168 {
            // Week+
            CreatureMood::Lonely
        } else if hours_away > 24 {
            CreatureMood::Sleepy
        } else {
            CreatureMood::Happy
        };
    }

    /// Update session time and grant XP
    pub fn tick_session(&mut self, seconds: u64) -> u64 {
        self.total_time_seconds += seconds;
        // 1 XP per 10 seconds of usage
        let xp_gained = seconds / 10;
        xp_gained
    }

    /// Check if a skill can be purchased
    pub fn can_purchase_skill(&self, skill: &Skill) -> bool {
        self.points >= skill.cost
            && !self.unlocked_skills.contains(&skill.id)
            && skill
                .prerequisites
                .iter()
                .all(|p| self.unlocked_skills.contains(p))
    }

    /// Purchase a skill with points
    pub fn purchase_skill(&mut self, skill: &Skill) -> bool {
        if self.can_purchase_skill(skill) {
            self.points -= skill.cost;
            self.unlocked_skills.push(skill.id.clone());
            true
        } else {
            false
        }
    }

    /// Equip an outfit
    pub fn equip_outfit(&mut self, outfit_id: &str) -> bool {
        if self.unlocked_outfits.contains(&outfit_id.to_string()) {
            self.equipped_outfit = Some(outfit_id.to_string());
            true
        } else {
            false
        }
    }

    /// Toggle a skill active/inactive
    pub fn toggle_skill(&mut self, skill_id: &str) -> bool {
        if !self.unlocked_skills.contains(&skill_id.to_string()) {
            return false;
        }

        if self.active_skills.contains(&skill_id.to_string()) {
            self.active_skills.retain(|s| s != skill_id);
        } else {
            self.active_skills.push(skill_id.to_string());
        }
        true
    }
}

/// Available creature species to choose from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreatureSpecies {
    Blob,    // Friendly slime creature
    Bird,    // Chirpy bird
    Cat,     // Classic cat companion
    Dragon,  // Mini dragon
    Fox,     // Clever fox
    Owl,     // Wise owl
    Penguin, // Cute penguin
    Robot,   // Friendly robot
    Spirit,  // Mystical spirit
    Octopus, // Multi-tasking octopus
}

impl Default for CreatureSpecies {
    fn default() -> Self {
        CreatureSpecies::Blob
    }
}

impl CreatureSpecies {
    pub fn all() -> Vec<CreatureSpecies> {
        vec![
            CreatureSpecies::Blob,
            CreatureSpecies::Bird,
            CreatureSpecies::Cat,
            CreatureSpecies::Dragon,
            CreatureSpecies::Fox,
            CreatureSpecies::Owl,
            CreatureSpecies::Penguin,
            CreatureSpecies::Robot,
            CreatureSpecies::Spirit,
            CreatureSpecies::Octopus,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            CreatureSpecies::Blob => "Blob",
            CreatureSpecies::Bird => "Bird",
            CreatureSpecies::Cat => "Cat",
            CreatureSpecies::Dragon => "Dragon",
            CreatureSpecies::Fox => "Fox",
            CreatureSpecies::Owl => "Owl",
            CreatureSpecies::Penguin => "Penguin",
            CreatureSpecies::Robot => "Robot",
            CreatureSpecies::Spirit => "Spirit",
            CreatureSpecies::Octopus => "Octopus",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CreatureSpecies::Blob => "A friendly blob that bounces with joy",
            CreatureSpecies::Bird => "A chirpy companion that loves news",
            CreatureSpecies::Cat => "A curious cat always watching the feeds",
            CreatureSpecies::Dragon => "A mini dragon with fiery enthusiasm",
            CreatureSpecies::Fox => "A clever fox with sharp insights",
            CreatureSpecies::Owl => "A wise owl for late-night browsing",
            CreatureSpecies::Penguin => "A cool penguin that slides through data",
            CreatureSpecies::Robot => "A helpful bot that never sleeps",
            CreatureSpecies::Spirit => "A mystical spirit from the terminal realm",
            CreatureSpecies::Octopus => "Multi-tasking master of many feeds",
        }
    }
}

/// Creature stats that can be improved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureStats {
    pub happiness: u8, // 0-100
    pub energy: u8,    // 0-100
    pub knowledge: u8, // 0-100
    pub charisma: u8,  // 0-100
}

impl Default for CreatureStats {
    fn default() -> Self {
        Self {
            happiness: 80,
            energy: 100,
            knowledge: 10,
            charisma: 10,
        }
    }
}

/// Creature appearance customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureAppearance {
    pub primary_color: CreatureColor,
    pub secondary_color: CreatureColor,
    pub accessory: Option<String>,
    pub hat: Option<String>,
    pub background: Option<String>,
}

impl Default for CreatureAppearance {
    fn default() -> Self {
        Self {
            primary_color: CreatureColor::Cyan,
            secondary_color: CreatureColor::White,
            accessory: None,
            hat: None,
            background: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreatureColor {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
    Orange,
    Pink,
    Purple,
}

impl CreatureColor {
    pub fn all() -> Vec<CreatureColor> {
        vec![
            CreatureColor::Red,
            CreatureColor::Green,
            CreatureColor::Blue,
            CreatureColor::Yellow,
            CreatureColor::Magenta,
            CreatureColor::Cyan,
            CreatureColor::White,
            CreatureColor::Orange,
            CreatureColor::Pink,
            CreatureColor::Purple,
        ]
    }

    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            CreatureColor::Red => Color::Red,
            CreatureColor::Green => Color::Green,
            CreatureColor::Blue => Color::Blue,
            CreatureColor::Yellow => Color::Yellow,
            CreatureColor::Magenta => Color::Magenta,
            CreatureColor::Cyan => Color::Cyan,
            CreatureColor::White => Color::White,
            CreatureColor::Orange => Color::Rgb(255, 165, 0),
            CreatureColor::Pink => Color::Rgb(255, 192, 203),
            CreatureColor::Purple => Color::Rgb(128, 0, 128),
        }
    }
}

/// Creature mood affects animations and interactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreatureMood {
    Happy,
    Excited,
    Sleepy,
    Thinking,
    Proud,
    Lonely,
    Curious,
}

impl CreatureMood {
    pub fn emoji(&self) -> &str {
        match self {
            CreatureMood::Happy => ":)",
            CreatureMood::Excited => ":D",
            CreatureMood::Sleepy => "-.-",
            CreatureMood::Thinking => "o.O",
            CreatureMood::Proud => "^_^",
            CreatureMood::Lonely => ":'(",
            CreatureMood::Curious => "?.?",
        }
    }
}

/// A skill that can be unlocked and used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub cost: u32,
    pub prerequisites: Vec<String>,
    pub effects: Vec<SkillEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCategory {
    Passive,  // Always active when equipped
    Active,   // Can be triggered
    Cosmetic, // Visual effects only
    Social,   // Affects emotes/interactions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    XpBoost(f32),               // Multiplier for XP gain
    RefreshBoost,               // Faster feed refresh
    NewsDigest,                 // Summarize news
    StockAlert,                 // Alert on stock changes
    CustomEmote(String),        // Unlock special emote
    ColorUnlock(CreatureColor), // Unlock new color
    Animation(String),          // Special animation
}

/// Reward for leveling up
#[derive(Debug, Clone)]
pub struct LevelUpReward {
    pub level: u32,
    pub points: u32,
    pub unlocked_skills: Vec<String>,
    pub unlocked_outfits: Vec<String>,
    pub unlocked_emotes: Vec<String>,
}

/// An outfit that changes the creature's appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outfit {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlock_level: Option<u32>,
    pub unlock_cost: Option<u32>,
    pub art_modifier: String,
}

/// An emote the creature can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub id: String,
    pub name: String,
    pub frames: Vec<String>,
    pub duration_ms: u64,
}

/// Get all available skills in the skill tree
pub fn get_skill_tree() -> HashMap<String, Skill> {
    let mut skills = HashMap::new();

    skills.insert(
        "greeting".to_string(),
        Skill {
            id: "greeting".to_string(),
            name: "Greeting".to_string(),
            description: "Tui greets you when you start a session".to_string(),
            category: SkillCategory::Passive,
            cost: 0,
            prerequisites: vec![],
            effects: vec![],
        },
    );

    skills.insert(
        "news_digest".to_string(),
        Skill {
            id: "news_digest".to_string(),
            name: "News Digest".to_string(),
            description: "Tui highlights the most important news".to_string(),
            category: SkillCategory::Passive,
            cost: 10,
            prerequisites: vec!["greeting".to_string()],
            effects: vec![SkillEffect::NewsDigest],
        },
    );

    skills.insert(
        "stock_alert".to_string(),
        Skill {
            id: "stock_alert".to_string(),
            name: "Stock Alert".to_string(),
            description: "Tui alerts you on significant stock movements".to_string(),
            category: SkillCategory::Passive,
            cost: 15,
            prerequisites: vec!["greeting".to_string()],
            effects: vec![SkillEffect::StockAlert],
        },
    );

    skills.insert(
        "speed_read".to_string(),
        Skill {
            id: "speed_read".to_string(),
            name: "Speed Read".to_string(),
            description: "Faster feed refresh rates".to_string(),
            category: SkillCategory::Passive,
            cost: 20,
            prerequisites: vec!["news_digest".to_string()],
            effects: vec![SkillEffect::RefreshBoost],
        },
    );

    skills.insert(
        "xp_boost_1".to_string(),
        Skill {
            id: "xp_boost_1".to_string(),
            name: "Quick Learner".to_string(),
            description: "Gain 10% more XP".to_string(),
            category: SkillCategory::Passive,
            cost: 15,
            prerequisites: vec!["greeting".to_string()],
            effects: vec![SkillEffect::XpBoost(1.1)],
        },
    );

    skills.insert(
        "xp_boost_2".to_string(),
        Skill {
            id: "xp_boost_2".to_string(),
            name: "Fast Learner".to_string(),
            description: "Gain 25% more XP".to_string(),
            category: SkillCategory::Passive,
            cost: 30,
            prerequisites: vec!["xp_boost_1".to_string()],
            effects: vec![SkillEffect::XpBoost(1.25)],
        },
    );

    skills.insert(
        "cosmic_insight".to_string(),
        Skill {
            id: "cosmic_insight".to_string(),
            name: "Cosmic Insight".to_string(),
            description: "Tui gains cosmic wisdom about trending topics".to_string(),
            category: SkillCategory::Passive,
            cost: 50,
            prerequisites: vec!["news_digest".to_string(), "stock_alert".to_string()],
            effects: vec![],
        },
    );

    skills.insert(
        "fire_breath".to_string(),
        Skill {
            id: "fire_breath".to_string(),
            name: "Fire Breath".to_string(),
            description: "Tui breathes fire when excited (cosmetic)".to_string(),
            category: SkillCategory::Cosmetic,
            cost: 40,
            prerequisites: vec![],
            effects: vec![SkillEffect::Animation("fire".to_string())],
        },
    );

    skills.insert(
        "omniscience".to_string(),
        Skill {
            id: "omniscience".to_string(),
            name: "Omniscience".to_string(),
            description: "Tui knows all. Maximum XP boost and insights.".to_string(),
            category: SkillCategory::Passive,
            cost: 100,
            prerequisites: vec!["cosmic_insight".to_string(), "xp_boost_2".to_string()],
            effects: vec![SkillEffect::XpBoost(1.5)],
        },
    );

    skills
}

/// Get all available outfits
pub fn get_all_outfits() -> HashMap<String, Outfit> {
    let mut outfits = HashMap::new();

    outfits.insert(
        "default".to_string(),
        Outfit {
            id: "default".to_string(),
            name: "Default".to_string(),
            description: "The classic look".to_string(),
            unlock_level: Some(1),
            unlock_cost: None,
            art_modifier: "default".to_string(),
        },
    );

    outfits.insert(
        "hacker".to_string(),
        Outfit {
            id: "hacker".to_string(),
            name: "Hacker".to_string(),
            description: "Hoodie and sunglasses for the l33t".to_string(),
            unlock_level: Some(5),
            unlock_cost: None,
            art_modifier: "hacker".to_string(),
        },
    );

    outfits.insert(
        "wizard".to_string(),
        Outfit {
            id: "wizard".to_string(),
            name: "Wizard".to_string(),
            description: "Mystical robes and a pointy hat".to_string(),
            unlock_level: Some(10),
            unlock_cost: None,
            art_modifier: "wizard".to_string(),
        },
    );

    outfits.insert(
        "ninja".to_string(),
        Outfit {
            id: "ninja".to_string(),
            name: "Ninja".to_string(),
            description: "Stealthy and swift".to_string(),
            unlock_level: Some(15),
            unlock_cost: None,
            art_modifier: "ninja".to_string(),
        },
    );

    outfits.insert(
        "astronaut".to_string(),
        Outfit {
            id: "astronaut".to_string(),
            name: "Astronaut".to_string(),
            description: "Ready for space exploration".to_string(),
            unlock_level: Some(20),
            unlock_cost: None,
            art_modifier: "astronaut".to_string(),
        },
    );

    outfits.insert(
        "robot".to_string(),
        Outfit {
            id: "robot".to_string(),
            name: "Robot".to_string(),
            description: "Mechanical enhancement suit".to_string(),
            unlock_level: Some(25),
            unlock_cost: None,
            art_modifier: "robot".to_string(),
        },
    );

    outfits.insert(
        "dragon".to_string(),
        Outfit {
            id: "dragon".to_string(),
            name: "Dragon".to_string(),
            description: "Scales and wings of legend".to_string(),
            unlock_level: Some(30),
            unlock_cost: None,
            art_modifier: "dragon".to_string(),
        },
    );

    outfits.insert(
        "legendary".to_string(),
        Outfit {
            id: "legendary".to_string(),
            name: "Legendary".to_string(),
            description: "The ultimate form. Pure energy.".to_string(),
            unlock_level: Some(50),
            unlock_cost: None,
            art_modifier: "legendary".to_string(),
        },
    );

    outfits
}

/// Get all available emotes
pub fn get_all_emotes() -> HashMap<String, Emote> {
    let mut emotes = HashMap::new();

    emotes.insert(
        "wave".to_string(),
        Emote {
            id: "wave".to_string(),
            name: "Wave".to_string(),
            frames: vec!["o/".to_string(), "o-".to_string(), "o\\".to_string()],
            duration_ms: 500,
        },
    );

    emotes.insert(
        "happy".to_string(),
        Emote {
            id: "happy".to_string(),
            name: "Happy".to_string(),
            frames: vec!["^_^".to_string(), "^-^".to_string()],
            duration_ms: 300,
        },
    );

    emotes.insert(
        "excited".to_string(),
        Emote {
            id: "excited".to_string(),
            name: "Excited".to_string(),
            frames: vec!["\\o/".to_string(), "|o|".to_string(), "/o\\".to_string()],
            duration_ms: 200,
        },
    );

    emotes.insert(
        "cool".to_string(),
        Emote {
            id: "cool".to_string(),
            name: "Cool".to_string(),
            frames: vec!["B)".to_string(), "B-)".to_string()],
            duration_ms: 400,
        },
    );

    emotes.insert(
        "stealth".to_string(),
        Emote {
            id: "stealth".to_string(),
            name: "Stealth".to_string(),
            frames: vec![
                "...".to_string(),
                "..".to_string(),
                ".".to_string(),
                "".to_string(),
            ],
            duration_ms: 200,
        },
    );

    emotes
}
