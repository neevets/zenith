use colored::*;

pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub file: String,
    pub span: logos::Span,
    pub label: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new_error(message: &str, file: &str, span: logos::Span) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.to_string(),
            file: file.to_string(),
            span,
            label: None,
            help: None,
        }
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn render(&self, source: &str) {
        let level_str = match self.level {
            DiagnosticLevel::Error => "error".red().bold(),
            DiagnosticLevel::Warning => "warning".yellow().bold(),
            DiagnosticLevel::Note => "note".blue().bold(),
        };

        // Convert span to line/col
        let mut line_num = 1;
        let mut col_num = 1;
        let mut line_start = 0;
        
        let bytes = source.as_bytes();
        for i in 0..self.span.start.min(bytes.len()) {
            if bytes[i] == b'\n' {
                line_num += 1;
                col_num = 1;
                line_start = i + 1;
            } else {
                col_num += 1;
            }
        }

        println!("{}: {}", level_str, self.message.white().bold());
        println!("  {} {}:{}:{}", "-->".blue().bold(), self.file, line_num, col_num);
        println!("   {}", "|".blue().bold());

        // Get the current line
        let line_end = source[line_start..].find('\n').map(|n| line_start + n).unwrap_or(source.len());
        let current_line = &source[line_start..line_end];

        println!("{:2} {} {}", line_num.to_string().blue().bold(), "|".blue().bold(), current_line);
        
        let pointer_space = " ".repeat(col_num - 1);
        let pointer_len = (self.span.end - self.span.start).max(1);
        let pointer = "^".repeat(pointer_len);
        
        let label_str = if let Some(ref l) = self.label {
            format!(" {}", l.red().bold())
        } else {
            "".to_string()
        };
        
        println!("   {} {}{}{}", "|".blue().bold(), pointer_space, pointer.red().bold(), label_str);

        if let Some(ref h) = self.help {
            println!("   {} {} {}", "|".blue().bold(), "=".blue().bold(), format!("{}: {}", "help".white().bold(), h));
        }
        println!("   {}", "|".blue().bold());
        println!();
    }
}
