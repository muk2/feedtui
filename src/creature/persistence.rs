use super::Creature;
use anyhow::Result;
use std::path::{Path, PathBuf};

const CREATURE_FILE: &str = "tui.json";

/// Get the default path for creature save file
pub fn default_creature_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".feedtui")
        .join(CREATURE_FILE)
}

/// Save creature state to file
pub fn save_creature(creature: &Creature, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(creature)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load creature state from file
pub fn load_creature(path: &Path) -> Result<Option<Creature>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let creature: Creature = serde_json::from_str(&content)?;
    Ok(Some(creature))
}

/// Load creature or create new one if none exists
pub fn load_or_create_creature(path: &Path) -> Result<Creature> {
    match load_creature(path)? {
        Some(mut creature) => {
            creature.start_session();
            Ok(creature)
        }
        None => {
            let creature = Creature::default();
            save_creature(&creature, path)?;
            Ok(creature)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::CreatureSpecies;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_creature() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_creature.json");

        let creature = Creature::new("TestTui".to_string(), CreatureSpecies::Cat);
        save_creature(&creature, &path).unwrap();

        let loaded = load_creature(&path).unwrap().unwrap();
        assert_eq!(loaded.name, "TestTui");
        assert_eq!(loaded.species, CreatureSpecies::Cat);
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = load_creature(&path).unwrap();
        assert!(result.is_none());
    }
}
