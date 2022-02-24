use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub struct Profiles {}

// TODO: possibly add defaults to some of these values

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Profile {
    pub java: JavaSettings,
    pub memory: MemorySettings,
    pub resolution: Option<WindowSize>,
    pub hooks: ProfileHooks,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JavaSettings {
    pub install: PathBuf,
    pub extra_arguments: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MemorySettings {
    pub minimum: Option<String>,
    pub maximum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WindowSize(u16, u16);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProfileHooks {
    pub pre_launch: Option<Vec<String>>,
    pub wrapper: Option<String>,
    pub post_exit: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn profile_test() -> Result<(), serde_json::Error> {
	let profile = Profile {
	    java: JavaSettings::new(PathBuf::from("/usr/bin/java"), None),
	    memory: MemorySettings::new(None, String::from("8192k")),
	    resolution: Some(WindowSettings::new(1920, 1080)),
	    hooks: ProfileHooks::new(None, None, None)
	};
	let json = serde_json::json!({
	    "java": {
		"install": "/usr/bin/java",
		"extra_arguments": None as Option<Vec<String>>,
	    },
	    "memory": {
		"maximum": "8192k",
		"minimum": None as Option<u16>,
	    },
	    "resolution": (1920u16, 1080u16),
	    "hooks": {
		"pre_launch": None as Option<Vec<String>>,
		"wrapper": None as Option<String>,
		"post_exit": None as Option<Vec<String>>,
	    },
	});
	
	assert_eq!(profile.clone(), serde_json::from_value(json.clone())?);
	assert_eq!(serde_json::to_value(profile)?, json);
	Ok(())
    }
}
