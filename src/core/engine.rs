use crate::codegen::transpiler::Transpiler;
use crate::core::analyzer::Analyzer;
use crate::core::ast::StatementKind;
use crate::core::cache::Cache;
use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use crate::core::system;
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

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
        self.transpile_recursive(filename, &mut HashMap::new(), false)
    }

    pub fn transpile_test(&self, filename: &str) -> anyhow::Result<String> {
        self.transpile_recursive(filename, &mut HashMap::new(), true)
    }

    fn transpile_recursive(
        &self,
        filename: &str,
        module_map: &mut HashMap<String, String>,
        is_test: bool,
    ) -> anyhow::Result<String> {
        let abs_path = if filename.starts_with("http") {
            PathBuf::from(filename)
        } else {
            std::fs::canonicalize(filename)?
        };

        let input = if filename.starts_with("http") {
            let cm = Cache::new()?;
            let local_path = cm.get(filename)?;
            std::fs::read_to_string(local_path)?
        } else {
            std::fs::read_to_string(&abs_path)?
        };

        let cm = Cache::new().ok();
        let mut source_hash = String::new();
        if let Some(ref c) = cm {
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            source_hash = format!("{:x}", hasher.finalize());
            if let Some(cached_php) = c.get_transpiled(&source_hash) {
                if filename.starts_with("http") {
                    module_map.insert(
                        filename.to_string(),
                        c.get_transpiled_path(&source_hash)
                            .to_string_lossy()
                            .to_string(),
                    );
                }
                return Ok(cached_php);
            }
        }

        let l = Lexer::new(&input);
        let mut p = Parser::new(l);
        let program = p.parse_program();

        if !p.errors.is_empty() {
            use crate::core::diagnostics::Diagnostic;
            for err in p.errors {
                let mut diag = Diagnostic::new_error(&err.message, filename, err.span);
                if let Some(label) = err.label.as_deref() {
                    diag = diag.with_label(label);
                }
                if let Some(help) = err.help.as_deref() {
                    diag = diag.with_help(help);
                }
                diag.render(&input);
            }
            return Err(anyhow::anyhow!(
                "Transpilation failed. See diagnostics above."
            ));
        }

        let mut a = Analyzer::new();
        let lc_map = a.analyze(&program);

        if !lc_map.errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Quantum Shield blocked execution in {}:\n{}",
                filename,
                lc_map.errors.join("\n")
            ));
        }

        
        for import_stmt in &program.imports {
            if let StatementKind::Import(path) = &import_stmt.kind {
                if path.starts_with("http") && !module_map.contains_key(path) {
                    let _ = self.transpile_recursive(path, module_map, is_test)?;
                } else if !module_map.contains_key(path) {
                    let _ = self.transpile_recursive(path, module_map, is_test)?;
                }
            }
        }

        let mut t = Transpiler::new();
        t.is_test_mode = is_test;
        t.set_module_map(module_map.clone());
        t.set_lifecycle_map(lc_map.clone());

        let header = t.get_php_header();
        let mut php_code = String::new();

        if let Some(ref c) = cm {
            let runtime_path = c.save_runtime(&header)?;
            php_code.push_str("<?php\n\ndeclare(strict_types=1);\n\n");
            php_code.push_str(&format!("require_once '{}';\n", runtime_path.to_string_lossy()));
        } else {
            php_code.push_str(&header);
        }

        if is_test {
            php_code.push_str("error_reporting(E_ALL & ~E_WARNING & ~E_NOTICE);\n");
        }

        let composer_path = std::path::Path::new("vendor/autoload.php");
        if composer_path.exists() {
            php_code.push_str("require_once __DIR__ . '/vendor/autoload.php';\n");
        }

        php_code.push_str(&t.transpile(&program));

        if is_test {
            php_code.push_str(&t.get_test_runner());
        }

        if let Some(ref c) = cm {
            if !source_hash.is_empty() {
                c.save_transpiled(&source_hash, &php_code)?;
                if filename.starts_with("http") {
                    module_map.insert(
                        filename.to_string(),
                        c.get_transpiled_path(&source_hash)
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            }
        }

        Ok(php_code)
    }

    pub fn execute(&self, php_code: &str) -> anyhow::Result<String> {
        self.execute_with_context(php_code, "index.zen", "")
    }

    pub fn execute_with_context(
        &self,
        php_code: &str,
        filename: &str,
        zenith_source: &str,
    ) -> anyhow::Result<String> {
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
            let disabled_funcs =
                "curl_init,curl_exec,fsockopen,pfsockopen,stream_socket_client,socket_create";
            args.push("-d".to_string());
            args.push(format!("disable_functions={}", disabled_funcs));
        }

        args.push(tmp_path.to_string_lossy().to_string());

        let output = Command::new(php_bin).args(args).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            let re = regex::Regex::new(r"PHP (.*?) error: (.*?) in (.*?) on line (\d+)").unwrap();
            if let Some(caps) = re.captures(&stderr) {
                let msg = &caps[2];
                let line: usize = caps[4].parse().unwrap_or(0);

                use crate::core::diagnostics::Diagnostic;
                let mut diag = Diagnostic::new_error(msg, filename, 0..1);
                diag = diag.with_help("This error occurred in the generated PHP runner.");

                if !zenith_source.is_empty() {
                    diag.render(zenith_source);
                } else {
                    println!("error: {}", msg.red().bold());
                    println!("  --> {}:{}", filename, line);
                }

                if !stdout.is_empty() {
                    return Ok(stdout);
                }
                return Err(anyhow::anyhow!("Execution failed."));
            }

            if !stdout.is_empty() {
                return Ok(stdout);
            }
            return Err(anyhow::anyhow!("PHP Execution Error:\n{}", stderr));
        }

        Ok(stdout)
    }
}
