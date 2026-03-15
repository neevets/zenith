use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub struct Cache {
    pub base_dir: PathBuf,
}

impl Cache {
    pub fn new() -> anyhow::Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let base = home.join(".zenith").join("cache");
        fs::create_dir_all(&base)?;
        Ok(Cache { base_dir: base })
    }

    pub fn get(&self, url: &str) -> anyhow::Result<String> {
        if !url.starts_with("http") {
            return Ok(url.to_string());
        }

        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let local_path = self.base_dir.join(format!("{}.zen", hash));

        if local_path.exists() {
            return Ok(local_path.to_string_lossy().to_string());
        }

        println!("Downloading {}...", url);
        let mut response = reqwest::blocking::get(url)?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download: status {}",
                response.status()
            ));
        }

        let mut out = fs::File::create(&local_path)?;
        response.copy_to(&mut out)?;

        Ok(local_path.to_string_lossy().to_string())
    }

    pub fn get_transpiled(&self, source_hash: &str) -> Option<String> {
        let local_path = self
            .base_dir
            .join("transpiled")
            .join(format!("{}.php", source_hash));
        if local_path.exists() {
            fs::read_to_string(local_path).ok()
        } else {
            None
        }
    }

    pub fn save_transpiled(&self, source_hash: &str, php_code: &str) -> anyhow::Result<()> {
        let dir = self.base_dir.join("transpiled");
        fs::create_dir_all(&dir)?;
        let local_path = dir.join(format!("{}.php", source_hash));
        fs::write(local_path, php_code)?;
        Ok(())
    }

    pub fn get_transpiled_path(&self, source_hash: &str) -> PathBuf {
        self.base_dir
            .join("transpiled")
            .join(format!("{}.php", source_hash))
    }

    pub fn save_runtime(&self, php_code: &str) -> anyhow::Result<PathBuf> {
        let path = self.base_dir.join("runtime.php");
        fs::write(&path, php_code)?;
        Ok(path)
    }
}
