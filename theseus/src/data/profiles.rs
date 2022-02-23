use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub struct Profiles {}

// TODO: possibly add defaults to some of these values

#[records::record]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Profile {
    java: JavaSettings,
    memory: MemorySettings,
    resolution: Option<WindowSettings>,
    hooks: ProfileHooks,
}

#[records::record]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JavaSettings {
    install: PathBuf,
    extra_arguments: Option<Vec<String>>,
}

#[records::record]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MemorySettings {
    minimum: Option<String>,
    maximum: String,
}

#[records::record]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WindowSettings {
    width: u16,
    height: u16,
}

#[records::record]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProfileHooks {
    pre_launch: Option<Vec<String>>,
    wrapper: Option<String>,
    post_exit: Option<Vec<String>>,
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
	    "resolution": {
		"width": 1920u16,
		"height": 1080u16,
	    },
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
