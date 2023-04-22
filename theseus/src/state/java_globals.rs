use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::prelude::JavaVersion;
use crate::util::jre;

// All stored Java versions, chosen by the user
// A wrapper over a Hashmap connecting key -> java version
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaGlobals(HashMap<String, JavaVersion>);

impl JavaGlobals {
    pub fn new() -> JavaGlobals {
        JavaGlobals(HashMap::new())
    }

    pub fn insert(&mut self, key: String, java: JavaVersion) {
        self.0.insert(key, java);
    }

    pub fn remove(&mut self, key: &String) {
        self.0.remove(key);
    }

    pub fn get(&self, key: &String) -> Option<&JavaVersion> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut JavaVersion> {
        self.0.get_mut(key)
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    // Validates that every path here is a valid Java version and that the version matches the version stored here
    // If false, when checked, the user should be prompted to reselect the Java version
    pub async fn is_all_valid(&self) -> bool {
        for (_, java) in self.0.iter() {
            let jre = jre::check_java_at_filepath(
                PathBuf::from(&java.path).as_path(),
            )
            .await;
            if let Some(jre) = jre {
                if jre.version != java.version {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl Default for JavaGlobals {
    fn default() -> Self {
        Self::new()
    }
}
