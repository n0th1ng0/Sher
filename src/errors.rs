use crate::token::Span;

pub struct Diagnostic;

impl Diagnostic {
    pub fn report(
        source: &str,
        file_path: &str,
        span: Span,
        category: &str,
        message: &str,
        hint: Option<&str>,
    ) {
        // Red bold header
        eprintln!("\x1b[1;31merror[{}]\x1b[0m: \x1b[1m{}\x1b[0m", category, message);

        // Location link
        eprintln!(
            "\x1b[1;36m  -->\x1b[0m {}:{}:{}",
            file_path, span.line, span.column
        );

        let lines: Vec<&str> = source.lines().collect();
        if span.line > 0 && span.line <= lines.len() {
            let code_line = lines[span.line - 1];
            let line_num_str = format!("{}", span.line);
            let pad = " ".repeat(line_num_str.len());

            eprintln!("\x1b[1;36m{} |\x1b[0m", pad);
            eprintln!("\x1b[1;36m{} |\x1b[0m {}", line_num_str, code_line);

            let col_pad = " ".repeat(span.column.saturating_sub(1));
            eprintln!(
                "\x1b[1;36m{} |\x1b[0m {}\x1b[1;31m^\x1b[0m",
                pad, col_pad
            );
            eprintln!("\x1b[1;36m{} |\x1b[0m", pad);
        }

        if let Some(h) = hint {
            eprintln!("\x1b[1;33m  = help:\x1b[0m {}", h);
        }
        eprintln!();
    }
}
