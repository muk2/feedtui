use super::{CreatureMood, CreatureSpecies};

/// Get ASCII art for a creature based on species, mood, and outfit
pub fn get_creature_art(
    species: &CreatureSpecies,
    mood: &CreatureMood,
    outfit: Option<&str>,
    frame: usize,
) -> Vec<String> {
    // Get base art for species
    let base_art = get_species_art(species, mood, frame);

    // Apply outfit modifications if applicable
    if let Some(outfit_id) = outfit {
        apply_outfit(outfit_id, base_art)
    } else {
        base_art
    }
}

fn get_species_art(species: &CreatureSpecies, mood: &CreatureMood, frame: usize) -> Vec<String> {
    match species {
        CreatureSpecies::Blob => get_blob_art(mood, frame),
        CreatureSpecies::Bird => get_bird_art(mood, frame),
        CreatureSpecies::Cat => get_cat_art(mood, frame),
        CreatureSpecies::Dragon => get_dragon_art(mood, frame),
        CreatureSpecies::Fox => get_fox_art(mood, frame),
        CreatureSpecies::Owl => get_owl_art(mood, frame),
        CreatureSpecies::Penguin => get_penguin_art(mood, frame),
        CreatureSpecies::Robot => get_robot_art(mood, frame),
        CreatureSpecies::Spirit => get_spirit_art(mood, frame),
        CreatureSpecies::Octopus => get_octopus_art(mood, frame),
    }
}

fn get_blob_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "  .-~~~-.".to_string(),
            " /       \\".to_string(),
            format!("|   {}   |", face),
            " \\       /".to_string(),
            "  '~---~'".to_string(),
        ],
        _ => vec![
            "  .~~~~~.".to_string(),
            " /       \\".to_string(),
            format!("|   {}   |", face),
            " \\       /".to_string(),
            "  '-----'".to_string(),
        ],
    }
}

fn get_bird_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "   __".to_string(),
            format!("  ({})", face),
            " >(  )>".to_string(),
            "   ^^".to_string(),
        ],
        _ => vec![
            "   __".to_string(),
            format!("  ({})", face),
            " <(  )<".to_string(),
            "   ^^".to_string(),
        ],
    }
}

fn get_cat_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "  /\\_/\\".to_string(),
            format!(" ( {} )", face),
            "  > ^ <".to_string(),
            " /|   |\\".to_string(),
            "(_|   |_)".to_string(),
        ],
        _ => vec![
            "  /\\_/\\".to_string(),
            format!(" ( {} )", face),
            "  > ^ <".to_string(),
            "  |   |".to_string(),
            " (_   _)".to_string(),
        ],
    }
}

fn get_dragon_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "    ____ ".to_string(),
            format!("   ( {} )", face),
            " /\\/    \\/\\".to_string(),
            "<<  ~~~~  >>".to_string(),
            "   \\    /".to_string(),
            "    ^^^^".to_string(),
        ],
        _ => vec![
            "    ____".to_string(),
            format!("   ( {} )~", face),
            " /\\/    \\/\\".to_string(),
            "<<  ~~~~  >>".to_string(),
            "   \\    /".to_string(),
            "    ^^^^".to_string(),
        ],
    }
}

fn get_fox_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "  /\\   /\\".to_string(),
            " /  \\ /  \\".to_string(),
            format!("|   {}   |", face),
            " \\  w  /".to_string(),
            "  \\___/".to_string(),
            "   | |".to_string(),
        ],
        _ => vec![
            "  /\\   /\\".to_string(),
            " /  \\ /  \\".to_string(),
            format!("|   {}   |", face),
            " \\  w  /".to_string(),
            "  \\___/".to_string(),
            "  |   |".to_string(),
        ],
    }
}

fn get_owl_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "  ,___,".to_string(),
            " (o   o)".to_string(),
            format!("  ( {} )", face),
            "  /| |\\".to_string(),
            " (_| |_)".to_string(),
        ],
        _ => vec![
            "  ,___,".to_string(),
            " (O   O)".to_string(),
            format!("  ( {} )", face),
            "  /| |\\".to_string(),
            " (_| |_)".to_string(),
        ],
    }
}

fn get_penguin_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "   __".to_string(),
            "  /  \\".to_string(),
            format!(" | {} |", face),
            " /|  |\\".to_string(),
            "(_|  |_)".to_string(),
            "   \\/".to_string(),
        ],
        _ => vec![
            "   __".to_string(),
            "  /  \\".to_string(),
            format!(" | {} |", face),
            "  |  |".to_string(),
            " /|  |\\".to_string(),
            "(_|__|_)".to_string(),
        ],
    }
}

fn get_robot_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "  ___".to_string(),
            " [___]".to_string(),
            format!(" |{}|", face),
            " |___|".to_string(),
            " /| |\\".to_string(),
            "/_| |_\\".to_string(),
        ],
        _ => vec![
            "  _*_".to_string(),
            " [___]".to_string(),
            format!(" |{}|", face),
            " |___|".to_string(),
            " /| |\\".to_string(),
            "/_| |_\\".to_string(),
        ],
    }
}

fn get_spirit_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 3 {
        0 => vec![
            "    *".to_string(),
            "  .oOo.".to_string(),
            format!(" ( {} )", face),
            "  '~'~'".to_string(),
            "   ~~~".to_string(),
        ],
        1 => vec![
            "   *".to_string(),
            "  .oOo.".to_string(),
            format!(" ( {} )", face),
            "  '~~~'".to_string(),
            "   ~~~".to_string(),
        ],
        _ => vec![
            "  *".to_string(),
            "  .oOo.".to_string(),
            format!(" ( {} )", face),
            "  '~~~'".to_string(),
            "    ~~".to_string(),
        ],
    }
}

fn get_octopus_art(mood: &CreatureMood, frame: usize) -> Vec<String> {
    let face = mood_to_face(mood);
    match frame % 2 {
        0 => vec![
            "   ___".to_string(),
            "  /   \\".to_string(),
            format!(" ( {} )", face),
            "  /|\\|\\".to_string(),
            " / | | \\".to_string(),
        ],
        _ => vec![
            "   ___".to_string(),
            "  /   \\".to_string(),
            format!(" ( {} )", face),
            "  \\|/|/".to_string(),
            "   | |".to_string(),
        ],
    }
}

fn mood_to_face(mood: &CreatureMood) -> &'static str {
    match mood {
        CreatureMood::Happy => "^_^",
        CreatureMood::Excited => "^o^",
        CreatureMood::Sleepy => "-_-",
        CreatureMood::Thinking => "o.o",
        CreatureMood::Proud => "^v^",
        CreatureMood::Lonely => ";_;",
        CreatureMood::Curious => "?.?",
    }
}

fn apply_outfit(outfit_id: &str, base_art: Vec<String>) -> Vec<String> {
    // Add accessories on top of base art based on outfit
    match outfit_id {
        "hacker" => {
            let mut art = vec!["  [===]  ".to_string()]; // sunglasses
            art.extend(base_art);
            art
        }
        "wizard" => {
            let mut art = vec![
                "   /\\".to_string(),
                "  /  \\".to_string(),
                "  ----".to_string(),
            ]; // wizard hat
            art.extend(base_art);
            art
        }
        "ninja" => {
            let mut art = vec!["  ~~~~~".to_string()]; // headband
            art.extend(base_art);
            art
        }
        "astronaut" => {
            let mut art = vec!["  /===\\".to_string(), " |     |".to_string()]; // helmet
            art.extend(base_art);
            art
        }
        "robot" => {
            let mut art = vec!["  [|||]".to_string()]; // antenna
            art.extend(base_art);
            art
        }
        "dragon" => {
            let mut art = vec!["  ^^^".to_string()]; // horns
            art.extend(base_art);
            art
        }
        "legendary" => {
            let mut art = vec!["  *****".to_string(), "  *   *".to_string()]; // crown
            art.extend(base_art);
            art
        }
        _ => base_art,
    }
}

/// Get a greeting message based on the creature's mood
pub fn get_greeting(mood: &CreatureMood, name: &str) -> String {
    match mood {
        CreatureMood::Happy => format!("{}: Hi there! Ready to browse?", name),
        CreatureMood::Excited => format!("{}: Woohoo! Let's see what's new!", name),
        CreatureMood::Sleepy => format!("{}: *yawn* Good to see you...", name),
        CreatureMood::Thinking => format!("{}: Hmm, interesting times...", name),
        CreatureMood::Proud => format!("{}: Look how much we've grown!", name),
        CreatureMood::Lonely => format!("{}: I missed you! Where were you?", name),
        CreatureMood::Curious => format!("{}: What shall we discover today?", name),
    }
}

/// Get an idle animation frame
pub fn get_idle_message(frame: usize) -> &'static str {
    match frame % 8 {
        0 => "...",
        1 => " ..",
        2 => "  .",
        3 => "   ",
        4 => ".  ",
        5 => ".. ",
        6 => "...",
        7 => " . ",
        _ => "...",
    }
}

/// Get level up celebration art
pub fn get_level_up_art() -> Vec<&'static str> {
    vec![
        "  *  LEVEL UP!  *",
        " *** ******** ***",
        "*******************",
        " *** ******** ***",
        "  *            *",
    ]
}

/// Get XP bar visualization
pub fn get_xp_bar(progress: f64, width: usize) -> String {
    let filled = (progress * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}
