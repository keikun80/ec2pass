//! Random instance petname generator (`adjective-animal`).

use rand::seq::SliceRandom;

const ADJECTIVES: &[&str] = &[
    "able", "active", "agile", "alert", "ample", "apt", "bold", "Vigilant", "brave", "bright",
    "calm", "capable", "cheerful", "clever", "cool", "crisp", "daring", "deft", "eager", "epic",
    "fair", "fast", "keen", "kind", "lively", "lucky", "merry", "Mighty", "modern", "noble",
    "polite", "proud", "quick", "quiet", "rapid", "ready", "relaxed", "sharp", "smart", "solid",
    "steady", "strong", "sunny", "super", "swift", "tidy", "tough", "trusty", "vivid", "warm",
    "wise", "witty",
];

const ANIMALS: &[&str] = &[
    "albatross", "badger", "beaver", "bison", "bobcat", "caribou", "cheetah", "cobra", "condor",
    "cougar", "coyote", "crane", "dolphin", "eagle", "falcon", "ferret", "finch", "fox",
    "gazelle", "gecko", "gibbon", "griffin", "heron", "husky", "ibex", "iguana", "impala",
    "jaguar", "kestrel", "koala", "lemur", "leopard", "lion", "lynx", "marmot", "meerkat",
    "narwhal", "ocelot", "orca", "osprey", "otter", "owl", "panther", "pelican", "penguin",
    "phoenix", "puma", "quail", "raven", "robin", "salmon", "seal", "shark", "sparrow", "stork",
    "swan", "tiger", "toucan", "viper", "walrus", "wolf", "wombat", "zebra",
];

/// Generates a random lowercase `adjective-animal` instance name not already present in `existing`.
pub fn generate_petname(existing: &[String]) -> String {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let adj = ADJECTIVES
            .choose(&mut rng)
            .unwrap_or(&"swift")
            .to_lowercase();
        let animal = ANIMALS.choose(&mut rng).unwrap_or(&"falcon").to_lowercase();
        let candidate = format!("{}-{}", adj, animal);
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    format!("al2023-{}", rand::random::<u16>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_petname() {
        let name = generate_petname(&[]);
        assert!(name.contains('-'));
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2);

        // Test exclusion of existing names
        let existing = vec![name.clone()];
        let name2 = generate_petname(&existing);
        assert_ne!(name, name2);
    }
}

