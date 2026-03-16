use crate::core::engine::{Engine, Options as EngineOptions};
use crate::core::server;
use actix_rt;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zenith")]
#[command(about = "The modern PHP programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        file: String,
        #[arg(short, long)]
        php: bool,
        #[arg(short, long)]
        watch: bool,
    },
    Serve {
        #[arg(default_value = "8080")]
        port: String,
    },
    Cache {
        #[arg(short, long)]
        reload: bool,
    },
    #[command(alias = "tests")]
    Test {
        file: Option<String>,
        #[arg(short, long)]
        php: bool,
    },
    Install,
}

pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file, php, watch } => {
            if *watch {
                println!("[Zenith] Watching for changes in {}...", file);
                let mut last_modified = std::fs::metadata(file).and_then(|m| m.modified()).ok();

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let current_modified = std::fs::metadata(file).and_then(|m| m.modified()).ok();

                    if current_modified != last_modified {
                        last_modified = current_modified;
                        println!("[Zenith] Change detected, restarting...");
                        let _ = execute_run(file, *php);
                    }
                }
            } else {
                if let Err(e) = execute_run(file, *php) {
                    println!("{}", e);
                }
            }
        }
        Commands::Serve { port } => {
            let rt = actix_rt::System::new();
            if let Err(e) = rt.block_on(server::start(port)) {
                println!("[!] Server Error: {}", e);
            }
        }
        Commands::Cache { reload } => {
            if *reload {
                let home = dirs::home_dir().unwrap();
                let cache_dir = home.join(".zenith").join("cache");
                if cache_dir.exists() {
                    let _ = std::fs::remove_dir_all(&cache_dir);
                    println!("[Zenith] Cache cleared.");
                }
            } else {
                println!("[Zenith] Cache is active.");
            }
        }
        Commands::Test { file, php } => {
            if let Err(e) = execute_test(file.as_ref(), *php) {
                println!("{}", e);
            }
        }
        Commands::Install => {
            println!("[Zenith] Installing dependencies...");
            if std::path::Path::new("composer.json").exists() {
                println!("[Zenith] Detected composer.json, running composer install...");
                let _ = std::process::Command::new("composer")
                    .arg("install")
                    .status();
            }
            println!("[Zenith] Done.");
        }
    }
}

fn execute_run(file: &str, show_php: bool) -> anyhow::Result<()> {
    let engine = Engine::new(EngineOptions {
        allow_read: true,
        allow_net: true,
        allow_env: true,
    });

    match engine.transpile(file) {
        Ok(php_code) => {
            if show_php {
                println!("{}", php_code);
            } else {
                let source = std::fs::read_to_string(file).unwrap_or_default();
                match engine.execute_with_context(&php_code, file, &source) {
                    Ok(output) => print!("{}", output),
                    Err(e) => println!("[!] Execution Error: {}", e),
                }
            }
        }
        Err(e) => println!("[!] Transpilation Error: {}", e),
    }
    Ok(())
}

fn execute_test(file: Option<&String>, show_php: bool) -> anyhow::Result<()> {
    use colored::Colorize;
    use std::path::Path;
    use walkdir::WalkDir;

    let engine = Engine::new(EngineOptions {
        allow_read: true,
        allow_net: true,
        allow_env: true,
    });

    let mut files = Vec::new();

    let test_regex = regex::Regex::new(r#"(?m)^\s*test\s+"#).unwrap();

    let mut find_zen_files = |dir: &str| {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "zen")
            })
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if test_regex.is_match(&content) {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    };

    if let Some(f) = file {
        let path = Path::new(f);
        if path.is_dir() {
            find_zen_files(f);
        } else {
            files.push(f.clone());
        }
    } else {
        find_zen_files("src");
        if Path::new("test").exists() {
            find_zen_files("test");
        }
        if Path::new("tests").exists() {
            find_zen_files("tests");
        }
    }

    if files.is_empty() {
        println!("[!] No .zen files found for testing.");
        return Ok(());
    }

    println!("\n{}", "Zenith Test Runner".bold().magenta());
    println!("{}", "=".repeat(40));

    for f in files {
        println!("\nFile: {}", f.bold().cyan());
        match engine.transpile_test(&f) {
            Ok(php_code) => {
                if show_php {
                    println!("{}", php_code);
                } else {
                    let source = std::fs::read_to_string(&f).unwrap_or_default();
                    match engine.execute_with_context(&php_code, &f, &source) {
                        Ok(output) => print!("{}", output),
                        Err(e) => println!("[!] Test Execution Error: {}", e),
                    }
                }
            }
            Err(e) => println!("[!] Transpilation Error: {}", e),
        }
    }

    Ok(())
}
