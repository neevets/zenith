use flate2::read::GzDecoder;
use std::env;
use std::fs;
use std::io::Cursor;
use std::process::Command;
use tar::Archive;

const PHP_VERSION: &str = "8.5.0";
const BASE_URL: &str = "https://github.com/shivammathur/php-bin/releases/download/";

pub fn ensure_php() -> anyhow::Result<String> {
    if let Ok(path) = which::which("php") {
        let path_s = path.to_string_lossy().to_string();
        if is_php_compatible(&path_s) {
            return Ok(path_s);
        }

        println!(
            "System PHP found at '{}' but version is lower than {}. Falling back to managed runtime.",
            path_s, PHP_VERSION
        );
    }

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let local_bin = home.join(".zenith").join("bin");
    let mut local_php = local_bin.join("php");
    if cfg!(windows) {
        local_php.set_extension("exe");
    }

    if local_php.exists() {
        let local_path = local_php.to_string_lossy().to_string();
        if is_php_compatible(&local_path) {
            return Ok(local_path);
        }

        println!(
            "Managed PHP found at '{}' but version is lower than {}. Reinstalling runtime.",
            local_path, PHP_VERSION
        );
    }

    println!(
        "PHP {} not found. Downloading PHP {} for {}/{}...",
        PHP_VERSION,
        PHP_VERSION,
        env::consts::OS,
        env::consts::ARCH
    );
    fs::create_dir_all(&local_bin)?;

    let url = get_download_url()?;
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download PHP: status {}",
            response.status()
        ));
    }

    let content = response.bytes()?;
    let tar_gz = GzDecoder::new(Cursor::new(content));
    let mut archive = Archive::new(tar_gz);

    println!("Extracting PHP runtime...");
    archive.unpack(&local_bin)?;

    if !local_php.exists() {
        return Err(anyhow::anyhow!(
            "Extraction succeeded but binary not found at {:?}",
            local_php
        ));
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

    Ok(format!(
        "{}{}/php-{}-{}-{}.tar.gz",
        BASE_URL,
        PHP_VERSION,
        PHP_VERSION,
        os_name,
        env::consts::ARCH
    ))
}

fn is_php_compatible(binary_path: &str) -> bool {
    let output = Command::new(binary_path)
        .arg("-r")
        .arg("echo PHP_VERSION;")
        .output();

    let Ok(output) = output else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    version_at_least(&version, PHP_VERSION)
}

fn version_at_least(current: &str, min: &str) -> bool {
    let c = parse_semver(current);
    let m = parse_semver(min);
    c >= m
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .next()
        .unwrap_or("0.0.0");

    let mut parts = clean.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}
