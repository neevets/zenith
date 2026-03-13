use clap::{Parser, Subcommand};
use crate::core::lexer::Lexer;
use crate::core::parser::Parser as ZenithParser;
use crate::core::analyzer::Analyzer;
use crate::codegen::transpiler::Transpiler;
use crate::codegen::native::NativeCompiler;
use inkwell::context::Context;

#[derive(Parser)]
#[command(name = "zenith")]
#[command(about = "The Modern PHP & Native Runtime", long_about = None)]
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
    Bundle {
        file: String,
    },
}

pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file, php } => {
            let input = std::fs::read_to_string(file).expect("Failed to read file");
            let lexer = Lexer::new(&input);
            let mut parser = ZenithParser::new(lexer);
            let program = parser.parse_program();

            let mut analyzer = Analyzer::new();
            let lc_map = analyzer.analyze(&program);

            if !lc_map.errors.is_empty() {
                for err in lc_map.errors {
                    println!("[!] {}", err);
                }
                return;
            }

            if *php {
                let transpiler = Transpiler::new();
                let php_code = transpiler.get_php_header() + &transpiler.transpile(&program);
                println!("{}", php_code);
            } else {
                let context = Context::create();
                let mut native = NativeCompiler::new(&context, "main");
                native.compile(&program);
                native.run_jit();
            }
        }
        Commands::Serve { port } => {
            println!("Starting server on port {}...", port);
        }
        Commands::Bundle { file } => {
            println!("Bundling {}...", file);
        }
    }
}
