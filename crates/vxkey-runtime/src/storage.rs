use crate::config::{Config, Platform};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
    pub config: Config,
}
impl ConfigStore {
    pub fn open(path: impl Into<PathBuf>, platform: Platform) -> Result<Self, String> {
        let path = path.into();
        let config = if path.exists() {
            let mut bytes = Vec::new();
            fs::File::open(&path)
                .map_err(|_| "read_failed")?
                .take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| "read_failed")?;
            if bytes.len() > 1024 * 1024 {
                return Err("config_too_large".into());
            }
            serde_json::from_slice(&bytes).map_err(|_| "invalid_config")?
        } else {
            Config::fresh(platform)
        };
        config.validate()?;
        Ok(Self { path, config })
    }
    pub fn replace(&mut self, mut new: Config, expected: &str) -> Result<(), String> {
        if expected != self.config.revision {
            return Err("revision_conflict".into());
        }
        new.revision = self
            .config
            .counter()?
            .checked_add(1)
            .ok_or("revision_overflow")?
            .to_string();
        new.validate()?;
        let durable = self.write(&new)?;
        // Rename is the commit point. Keep memory aligned even if directory fsync fails.
        self.config = new;
        if durable {
            Ok(())
        } else {
            Err("durability_uncertain".into())
        }
    }
    pub fn save_initial(&self) -> Result<(), String> {
        if !self.path.exists() {
            if self.write(&self.config)? {
                Ok(())
            } else {
                Err("durability_uncertain".into())
            }
        } else {
            Ok(())
        }
    }
    fn write(&self, c: &Config) -> Result<bool, String> {
        let parent = self.path.parent().ok_or("invalid_path")?;
        fs::create_dir_all(parent).map_err(|_| "persist_failed")?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "clock_error")?
            .as_nanos();
        let temp = parent.join(format!("config_tmp_{}_{}", std::process::id(), nonce));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = (|| {
            let mut file = options.open(&temp).map_err(|_| "persist_failed")?;
            file.write_all(&serde_json::to_vec_pretty(c).map_err(|_| "invalid_config")?)
                .map_err(|_| "persist_failed")?;
            file.sync_all().map_err(|_| "persist_failed")?;
            fs::rename(&temp, &self.path).map_err(|_| "persist_failed")?;
            #[cfg(unix)]
            {
                return Ok(fs::File::open(parent).and_then(|f| f.sync_all()).is_ok());
            }
            #[cfg(not(unix))]
            Ok(true)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}
