use crate::launcher::download::get_os;
use crate::launcher::meta::{OsRule, Rule, RuleAction};
use regex::Regex;

pub fn parse_rules(rules: &[Rule]) -> bool {
    rules.iter().all(|x| parse_rule(x))
}

pub fn parse_rule(rule: &Rule) -> bool {
    let result = if let Some(os) = &rule.os {
        parse_os_rule(os)
    } else if rule.features.is_some() {
        false
    } else {
        true
    };

    match rule.action {
        RuleAction::Allow => result,
        RuleAction::Disallow => !result,
    }
}

pub fn parse_os_rule(rule: &OsRule) -> bool {
    if let Some(arch) = &rule.arch {
        match arch.as_str() {
            "x86" => {
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                return false;
            }
            "arm" => {
                #[cfg(not(target_arch = "arm"))]
                return false;
            }
            _ => {}
        }
    }

    if let Some(name) = &rule.name {
        if &get_os() != name {
            return false;
        }
    }
    if let Some(version) = &rule.version {
        let regex = Regex::new(version.as_str());

        if let Ok(regex) = regex {
            if !regex.is_match(&*sys_info::os_release().unwrap_or_default()) {
                return false;
            }
        }
    }

    true
}
