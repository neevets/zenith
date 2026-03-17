use flate2::read::GzDecoder;
use std::env;
use std::fs;
use std::io::Cursor;
use std::process::Command;
use tar::Archive;

const PHP_VERSION: &str = "8.4";
const BASE_URL: &str = "https://github.com/pmmp/PHP-Binaries/releases/download/pm5-php-8.4-latest/";

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
        println!("Binary not found at expected path, searching recursively...");
        let mut found_path = None;
        for entry in walkdir::WalkDir::new(&local_bin)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "php" || entry.file_name() == "php.exe" {
                if entry.path().is_file() {
                    let p = entry.path().to_path_buf();
                    // Basic sanity check to ensure it's the right one (PMMP binaries are in a bin dir)
                    if p.to_string_lossy().contains("/bin/") {
                         found_path = Some(p);
                         break;
                    }
                }
            }
        }

        if let Some(path) = found_path {
            println!("Found PHP binary at {:?}", path);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&path, perms)?;
                
                // Create a symlink at the expected local_php location
                let _ = std::os::unix::fs::symlink(&path, &local_php);
            }
            return Ok(path.to_string_lossy().to_string());
        } else {
            return Err(anyhow::anyhow!(
                "Extraction succeeded but binary 'php' not found in {:?}",
                local_bin
            ));
        }
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
    let (os_name, ext) = match env::consts::OS {
        "linux" => ("Linux", "tar.gz"),
        "macos" => ("macOS", "tar.gz"),
        "windows" => ("Windows", "zip"),
        _ => return Err(anyhow::anyhow!("Unsupported OS: {}", env::consts::OS)),
    };

    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported Architecture: {}", env::consts::ARCH)),
    };

    // PMMP Format: PHP-8.4-Linux-x86_64-PM5.tar.gz
    Ok(format!(
        "{}PHP-{}-{}-{}-PM5.{}",
        BASE_URL,
        PHP_VERSION,
        os_name,
        arch,
        ext
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
