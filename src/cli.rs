use clap::{Parser, Subcommand};
use crate::core::engine::{Engine, Options as EngineOptions};
use crate::core::server;
use actix_rt;

#[derive(Parser)]
#[command(name = "zenith")]
#[command(about = "The Modern PHP & Rust Runtime", long_about = None)]
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
                match engine.execute(&php_code) {
                    Ok(output) => print!("{}", output),
                    Err(e) => println!("[!] Execution Error: {}", e),
                }
            }
        }
        Err(e) => println!("[!] Transpilation Error: {}", e),
    }
    Ok(())
}
