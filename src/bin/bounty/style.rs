//! Terminal styling utilities

use std::io::Write;

pub fn style_cyan(s: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", s)
}

pub fn style_green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

pub fn style_red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

pub fn style_yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

pub fn style_dim(s: &str) -> String {
    format!("\x1b[2m{}\x1b[0m", s)
}

pub fn style_bold(s: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", s)
}

pub fn print_success(msg: &str) {
    println!("{} {}", style_green("✓"), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", style_red("✗"), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", style_yellow("⚠"), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", style_cyan("ℹ"), msg);
}

pub fn print_header(title: &str) {
    println!();
    println!("{}", style_bold(title));
    println!("{}", "─".repeat(title.len()));
}
