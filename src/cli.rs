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
    },
    Serve {
        #[arg(default_value = "8080")]
        port: String,
    },
}

pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file, php } => {
            let engine = Engine::new(EngineOptions {
                allow_read: true,
                allow_net: true,
                allow_env: true,
            });

            match engine.transpile(file) {
                Ok(php_code) => {
                    if *php {
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
        }
        Commands::Serve { port } => {
            let rt = actix_rt::System::new();
            if let Err(e) = rt.block_on(server::start(port)) {
                println!("[!] Server Error: {}", e);
            }
        }
    }
}
