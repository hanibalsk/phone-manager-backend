//! Policy resolution service for managed device settings.
//!
//! Story 13.6: Policy Resolution Algorithm Implementation
//!
//! This service resolves effective settings for a device based on policy hierarchy:
//! 1. Organization defaults
//! 2. Group policy
//! 3. Device policy
//! 4. Device custom settings (only non-locked keys)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Resolved settings for a device with locked key information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedSettings {
    /// Merged settings from all policy levels
    pub settings: HashMap<String, serde_json::Value>,
    /// Combined set of locked keys from all policy levels
    pub locked_keys: HashSet<String>,
    /// Source information for each setting
    pub sources: HashMap<String, SettingSource>,
}

/// Source of a setting value for debugging/auditing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettingSource {
    OrganizationDefault,
    GroupPolicy,
    DevicePolicy,
    DeviceCustom,
    DefaultValue,
}

impl std::fmt::Display for SettingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrganizationDefault => write!(f, "organization_default"),
            Self::GroupPolicy => write!(f, "group_policy"),
            Self::DevicePolicy => write!(f, "device_policy"),
            Self::DeviceCustom => write!(f, "device_custom"),
            Self::DefaultValue => write!(f, "default_value"),
        }
    }
}

/// Input for policy resolution.
#[derive(Debug, Clone, Default)]
pub struct PolicyResolutionInput {
    /// Organization default settings (if device is managed)
    pub organization_defaults: Option<HashMap<String, serde_json::Value>>,
    /// Group policy settings and locked keys
    pub group_policy: Option<PolicySettings>,
    /// Device-specific policy settings and locked keys
    pub device_policy: Option<PolicySettings>,
    /// Device custom settings
    pub device_settings: HashMap<String, serde_json::Value>,
    /// Default setting values from setting definitions
    pub setting_defaults: HashMap<String, serde_json::Value>,
}

/// Policy settings with locked keys.
#[derive(Debug, Clone, Default)]
pub struct PolicySettings {
    pub settings: HashMap<String, serde_json::Value>,
    pub locked_keys: Vec<String>,
}

impl ResolvedSettings {
    /// Create a new empty ResolvedSettings.
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
            locked_keys: HashSet::new(),
            sources: HashMap::new(),
        }
    }

    /// Check if a key is locked.
    pub fn is_locked(&self, key: &str) -> bool {
        self.locked_keys.contains(key)
    }

    /// Get a setting value.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.settings.get(key)
    }

    /// Get the source of a setting.
    pub fn get_source(&self, key: &str) -> Option<&SettingSource> {
        self.sources.get(key)
    }
}

impl Default for ResolvedSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve effective settings for a device based on policy hierarchy.
///
/// Priority order (lowest to highest):
/// 1. Setting defaults
/// 2. Organization defaults
/// 3. Group policy
/// 4. Device policy
/// 5. Device custom settings (only non-locked keys)
///
/// Locked keys are accumulated from all policy levels.
pub fn resolve_effective_settings(input: PolicyResolutionInput) -> ResolvedSettings {
    let mut result = ResolvedSettings::new();

    // 1. Start with setting defaults
    for (key, value) in input.setting_defaults {
        result.settings.insert(key.clone(), value);
        result.sources.insert(key, SettingSource::DefaultValue);
    }

    // 2. Apply organization defaults
    if let Some(org_defaults) = input.organization_defaults {
        for (key, value) in org_defaults {
            result.settings.insert(key.clone(), value);
            result
                .sources
                .insert(key, SettingSource::OrganizationDefault);
        }
    }

    // 3. Apply group policy
    if let Some(group_policy) = input.group_policy {
        for (key, value) in group_policy.settings {
            result.settings.insert(key.clone(), value);
            result.sources.insert(key, SettingSource::GroupPolicy);
        }
        for key in group_policy.locked_keys {
            result.locked_keys.insert(key);
        }
    }

    // 4. Apply device policy
    if let Some(device_policy) = input.device_policy {
        for (key, value) in device_policy.settings {
            result.settings.insert(key.clone(), value);
            result.sources.insert(key, SettingSource::DevicePolicy);
        }
        for key in device_policy.locked_keys {
            result.locked_keys.insert(key);
        }
    }

    // 5. Apply device custom settings (only non-locked keys)
    for (key, value) in input.device_settings {
        if !result.locked_keys.contains(&key) {
            result.settings.insert(key.clone(), value);
            result.sources.insert(key, SettingSource::DeviceCustom);
        }
        // Locked keys are silently ignored - the policy value takes precedence
    }

    result
}

/// Check if a device needs policy resolution (is managed).
pub fn needs_resolution(organization_id: Option<uuid::Uuid>, is_managed: bool) -> bool {
    organization_id.is_some() && is_managed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_empty_input() {
        let input = PolicyResolutionInput::default();
        let result = resolve_effective_settings(input);
        assert!(result.settings.is_empty());
        assert!(result.locked_keys.is_empty());
    }

    #[test]
    fn test_resolve_with_setting_defaults() {
        let mut defaults = HashMap::new();
        defaults.insert("tracking_enabled".to_string(), json!(true));
        defaults.insert("tracking_interval".to_string(), json!(300));

        let input = PolicyResolutionInput {
            setting_defaults: defaults,
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        assert_eq!(result.get("tracking_enabled"), Some(&json!(true)));
        assert_eq!(result.get("tracking_interval"), Some(&json!(300)));
        assert_eq!(
            result.get_source("tracking_enabled"),
            Some(&SettingSource::DefaultValue)
        );
    }

    #[test]
    fn test_resolve_org_defaults_override_setting_defaults() {
        let mut setting_defaults = HashMap::new();
        setting_defaults.insert("tracking_enabled".to_string(), json!(true));

        let mut org_defaults = HashMap::new();
        org_defaults.insert("tracking_enabled".to_string(), json!(false));

        let input = PolicyResolutionInput {
            setting_defaults,
            organization_defaults: Some(org_defaults),
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        assert_eq!(result.get("tracking_enabled"), Some(&json!(false)));
        assert_eq!(
            result.get_source("tracking_enabled"),
            Some(&SettingSource::OrganizationDefault)
        );
    }

    #[test]
    fn test_resolve_group_policy_override() {
        let mut org_defaults = HashMap::new();
        org_defaults.insert("tracking_enabled".to_string(), json!(false));
        org_defaults.insert("secret_mode".to_string(), json!(false));

        let mut group_settings = HashMap::new();
        group_settings.insert("tracking_enabled".to_string(), json!(true));

        let input = PolicyResolutionInput {
            organization_defaults: Some(org_defaults),
            group_policy: Some(PolicySettings {
                settings: group_settings,
                locked_keys: vec!["tracking_enabled".to_string()],
            }),
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        assert_eq!(result.get("tracking_enabled"), Some(&json!(true)));
        assert_eq!(result.get("secret_mode"), Some(&json!(false)));
        assert!(result.is_locked("tracking_enabled"));
        assert!(!result.is_locked("secret_mode"));
        assert_eq!(
            result.get_source("tracking_enabled"),
            Some(&SettingSource::GroupPolicy)
        );
    }

    #[test]
    fn test_resolve_device_policy_override() {
        let mut group_settings = HashMap::new();
        group_settings.insert("tracking_enabled".to_string(), json!(true));
        group_settings.insert("tracking_interval".to_string(), json!(300));

        let mut device_policy_settings = HashMap::new();
        device_policy_settings.insert("tracking_interval".to_string(), json!(60));

        let input = PolicyResolutionInput {
            group_policy: Some(PolicySettings {
                settings: group_settings,
                locked_keys: vec!["tracking_enabled".to_string()],
            }),
            device_policy: Some(PolicySettings {
                settings: device_policy_settings,
                locked_keys: vec!["tracking_interval".to_string()],
            }),
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        assert_eq!(result.get("tracking_enabled"), Some(&json!(true)));
        assert_eq!(result.get("tracking_interval"), Some(&json!(60)));
        assert!(result.is_locked("tracking_enabled"));
        assert!(result.is_locked("tracking_interval"));
        assert_eq!(
            result.get_source("tracking_interval"),
            Some(&SettingSource::DevicePolicy)
        );
    }

    #[test]
    fn test_resolve_device_custom_settings() {
        let mut group_settings = HashMap::new();
        group_settings.insert("tracking_enabled".to_string(), json!(true));

        let mut device_settings = HashMap::new();
        device_settings.insert("display_name".to_string(), json!("My Device"));
        device_settings.insert("tracking_enabled".to_string(), json!(false)); // Should be ignored

        let input = PolicyResolutionInput {
            group_policy: Some(PolicySettings {
                settings: group_settings,
                locked_keys: vec!["tracking_enabled".to_string()],
            }),
            device_settings,
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        // tracking_enabled should remain true from group policy (locked)
        assert_eq!(result.get("tracking_enabled"), Some(&json!(true)));
        // display_name should be from device custom
        assert_eq!(result.get("display_name"), Some(&json!("My Device")));
        assert_eq!(
            result.get_source("display_name"),
            Some(&SettingSource::DeviceCustom)
        );
    }

    #[test]
    fn test_resolve_locked_keys_accumulate() {
        let input = PolicyResolutionInput {
            group_policy: Some(PolicySettings {
                settings: HashMap::new(),
                locked_keys: vec!["key1".to_string(), "key2".to_string()],
            }),
            device_policy: Some(PolicySettings {
                settings: HashMap::new(),
                locked_keys: vec!["key3".to_string(), "key2".to_string()], // key2 duplicate
            }),
            ..Default::default()
        };

        let result = resolve_effective_settings(input);
        assert!(result.is_locked("key1"));
        assert!(result.is_locked("key2"));
        assert!(result.is_locked("key3"));
        assert!(!result.is_locked("key4"));
        assert_eq!(result.locked_keys.len(), 3); // No duplicates
    }

    #[test]
    fn test_resolve_full_hierarchy() {
        let mut setting_defaults = HashMap::new();
        setting_defaults.insert("a".to_string(), json!(1));
        setting_defaults.insert("b".to_string(), json!(1));
        setting_defaults.insert("c".to_string(), json!(1));
        setting_defaults.insert("d".to_string(), json!(1));
        setting_defaults.insert("e".to_string(), json!(1));

        let mut org_defaults = HashMap::new();
        org_defaults.insert("b".to_string(), json!(2));
        org_defaults.insert("c".to_string(), json!(2));
        org_defaults.insert("d".to_string(), json!(2));
        org_defaults.insert("e".to_string(), json!(2));

        let mut group_settings = HashMap::new();
        group_settings.insert("c".to_string(), json!(3));
        group_settings.insert("d".to_string(), json!(3));
        group_settings.insert("e".to_string(), json!(3));

        let mut device_policy_settings = HashMap::new();
        device_policy_settings.insert("d".to_string(), json!(4));
        device_policy_settings.insert("e".to_string(), json!(4));

        let mut device_settings = HashMap::new();
        device_settings.insert("e".to_string(), json!(5));

        let input = PolicyResolutionInput {
            setting_defaults,
            organization_defaults: Some(org_defaults),
            group_policy: Some(PolicySettings {
                settings: group_settings,
                locked_keys: vec!["c".to_string()],
            }),
            device_policy: Some(PolicySettings {
                settings: device_policy_settings,
                locked_keys: vec!["d".to_string()],
            }),
            device_settings,
        };

        let result = resolve_effective_settings(input);

        // a: only in defaults
        assert_eq!(result.get("a"), Some(&json!(1)));
        assert_eq!(result.get_source("a"), Some(&SettingSource::DefaultValue));

        // b: overridden by org defaults
        assert_eq!(result.get("b"), Some(&json!(2)));
        assert_eq!(
            result.get_source("b"),
            Some(&SettingSource::OrganizationDefault)
        );

        // c: overridden by group policy, locked
        assert_eq!(result.get("c"), Some(&json!(3)));
        assert_eq!(result.get_source("c"), Some(&SettingSource::GroupPolicy));
        assert!(result.is_locked("c"));

        // d: overridden by device policy, locked
        assert_eq!(result.get("d"), Some(&json!(4)));
        assert_eq!(result.get_source("d"), Some(&SettingSource::DevicePolicy));
        assert!(result.is_locked("d"));

        // e: would be device custom, but not locked so it takes effect
        assert_eq!(result.get("e"), Some(&json!(5)));
        assert_eq!(result.get_source("e"), Some(&SettingSource::DeviceCustom));
        assert!(!result.is_locked("e"));
    }

    #[test]
    fn test_needs_resolution() {
        // Not managed
        assert!(!needs_resolution(None, false));
        assert!(!needs_resolution(Some(uuid::Uuid::new_v4()), false));

        // No org
        assert!(!needs_resolution(None, true));

        // Managed with org
        assert!(needs_resolution(Some(uuid::Uuid::new_v4()), true));
    }

    #[test]
    fn test_setting_source_display() {
        assert_eq!(
            SettingSource::OrganizationDefault.to_string(),
            "organization_default"
        );
        assert_eq!(SettingSource::GroupPolicy.to_string(), "group_policy");
        assert_eq!(SettingSource::DevicePolicy.to_string(), "device_policy");
        assert_eq!(SettingSource::DeviceCustom.to_string(), "device_custom");
        assert_eq!(SettingSource::DefaultValue.to_string(), "default_value");
    }

    #[test]
    fn test_resolved_settings_methods() {
        let mut result = ResolvedSettings::new();
        result.settings.insert("key1".to_string(), json!(42));
        result.locked_keys.insert("key2".to_string());
        result
            .sources
            .insert("key1".to_string(), SettingSource::DeviceCustom);

        assert_eq!(result.get("key1"), Some(&json!(42)));
        assert_eq!(result.get("nonexistent"), None);
        assert!(!result.is_locked("key1"));
        assert!(result.is_locked("key2"));
        assert_eq!(
            result.get_source("key1"),
            Some(&SettingSource::DeviceCustom)
        );
    }
}
