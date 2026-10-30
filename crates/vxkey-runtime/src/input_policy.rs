use crate::config::Config;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub app_id: String,
    pub pid: u32,
    pub generation: u64,
}
#[derive(Clone, Debug)]
pub struct InputPolicy {
    pub target: Option<Target>,
    pub suspended: bool,
    pub secure: bool,
    pub global_mode: String,
}
impl Default for InputPolicy {
    fn default() -> Self {
        Self {
            target: None,
            suspended: false,
            secure: false,
            global_mode: "vi".into(),
        }
    }
}
impl InputPolicy {
    pub fn blocked(&self, config: &Config) -> bool {
        self.suspended
            || self.secure
            || self
                .target
                .as_ref()
                .and_then(|t| config.apps.policies.get(&t.app_id))
                .is_some_and(|p| p.input_bypass)
    }
    pub fn mode(&self, config: &Config) -> String {
        if config.apps.remember_per_app {
            if let Some(mode) = self
                .target
                .as_ref()
                .and_then(|t| config.apps.language_memory.get(&t.app_id))
            {
                return mode.clone();
            }
        }
        self.global_mode.clone()
    }
    pub fn validate_target(&self, t: &Target) -> bool {
        self.target.as_ref() == Some(t)
    }
}
