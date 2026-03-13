use std::fs;
use std::env;
use std::io::Cursor;
use tar::Archive;
use flate2::read::GzDecoder;

const PHP_VERSION: &str = "8.2.0";
const BASE_URL: &str = "https://github.com/shivammathur/php-bin/releases/download/";

pub fn ensure_php() -> anyhow::Result<String> {
    if let Ok(path) = which::which("php") {
        return Ok(path.to_string_lossy().to_string());
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let local_bin = home.join(".zenith").join("bin");
    let mut local_php = local_bin.join("php");
    if cfg!(windows) {
        local_php.set_extension("exe");
    }

    if local_php.exists() {
        return Ok(local_php.to_string_lossy().to_string());
    }

    println!("PHP not found in system. Downloading PHP {} for {}/{}...", PHP_VERSION, env::consts::OS, env::consts::ARCH);
    fs::create_dir_all(&local_bin)?;

    let url = get_download_url()?;
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download PHP: status {}", response.status()));
    }

    let content = response.bytes()?;
    let tar_gz = GzDecoder::new(Cursor::new(content));
    let mut archive = Archive::new(tar_gz);

    println!("Extracting PHP runtime...");
    archive.unpack(&local_bin)?;

    if !local_php.exists() {
        return Err(anyhow::anyhow!("Extraction succeeded but binary not found at {:?}", local_php));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&local_php)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&local_php, perms)?;
    }

    Ok(local_php.to_string_lossy().to_string())
}

fn get_download_url() -> anyhow::Result<String> {
    let os_name = match env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "win",
        _ => return Err(anyhow::anyhow!("Unsupported OS: {}", env::consts::OS)),
    };

    Ok(format!("{}{}/php-{}-{}-{}.tar.gz", BASE_URL, PHP_VERSION, PHP_VERSION, os_name, env::consts::ARCH))
}
