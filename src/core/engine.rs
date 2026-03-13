use sha2::{Sha256, Digest};
use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use crate::core::analyzer::Analyzer;
use crate::codegen::transpiler::Transpiler;
use crate::core::cache::Cache;
use crate::core::system;
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

pub struct Options {
    pub allow_read: bool,
    pub allow_net: bool,
    pub allow_env: bool,
}

pub struct Engine {
    opts: Options,
}

impl Engine {
    pub fn new(opts: Options) -> Self {
        Engine { opts }
    }

    pub fn transpile(&self, filename: &str) -> anyhow::Result<String> {
        let abs_path = std::fs::canonicalize(filename)?;
        let input = std::fs::read_to_string(&abs_path)?;

        let cm = Cache::new().ok();
        let mut source_hash = String::new();
        if let Some(ref c) = cm {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            source_hash = format!("{:x}", hasher.finalize());
            if let Some(cached_php) = c.get_transpiled(&source_hash) {
                return Ok(cached_php);
            }
        }

        let l = Lexer::new(&input);
        let mut p = Parser::new(l);
        let program = p.parse_program();

        if !p.errors.is_empty() {
            return Err(anyhow::anyhow!("Parser errors:\n{}", p.errors.join("\n")));
        }

        let mut a = Analyzer::new();
        let lc_map = a.analyze(&program);

        if !lc_map.errors.is_empty() {
            return Err(anyhow::anyhow!("Quantum Shield blocked execution:\n{}", lc_map.errors.join("\n")));
        }

        let t = Transpiler::new();
        // t.set_lifecycle_map(lc_map); // If implemented in transpiler

        let _dir = abs_path.parent().unwrap();
        // Handle imports recursively if needed, but for now we'll assume they're handled by the transpiler
        
        let mut php_code = t.get_php_header();
        php_code.push_str(&t.transpile(&program));

        if let Some(ref c) = cm {
            if !source_hash.is_empty() {
                c.save_transpiled(&source_hash, &php_code)?;
            }
        }

        Ok(php_code)
    }

    pub fn execute(&self, php_code: &str) -> anyhow::Result<String> {
        let mut tmp_file = NamedTempFile::new_in(".")?;
        tmp_file.write_all(php_code.as_bytes())?;
        let tmp_path = tmp_file.path().to_owned();

        let php_bin = system::ensure_php()?;

        let mut args = Vec::new();
        if !self.opts.allow_read {
            args.push("-d".to_string());
            args.push(format!("open_basedir=.:{:?}", tmp_path));
        }

        if !self.opts.allow_net {
            args.push("-d".to_string());
            args.push("allow_url_fopen=Off".to_string());
            let disabled_funcs = "curl_init,curl_exec,fsockopen,pfsockopen,stream_socket_client,socket_create";
            args.push("-d".to_string());
            args.push(format!("disable_functions={}", disabled_funcs));
        }

        args.push(tmp_path.to_string_lossy().to_string());

        let output = Command::new(php_bin)
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("PHP Execution Error:\n{}\n\nGenerated PHP:\n{}", String::from_utf8_lossy(&output.stderr), php_code));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
