use dialoguer::{Confirm, Select};
use eyre::Result;
use std::{
    io::{self, Write},
    path::Path,
};

// Prompting helpers
pub fn prompt(prompt: &str) -> Result<String> {
    print_prompt(prompt);
    print!(": ");
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim_end().to_string())
}

pub async fn prompt_async(text: String) -> Result<String> {
    tokio::task::spawn_blocking(move || prompt(&text)).await?
}

// Selection helpers
pub fn select(prompt: &str, choices: &[&str]) -> Result<usize> {
    print_prompt(prompt);

    let res = Select::new().items(choices).default(0).interact()?;
    println!("> {}", choices[res]);
    Ok(res)
}

pub async fn select_async(
    prompt: String,
    choices: &'static [&'static str],
) -> Result<usize> {
    tokio::task::spawn_blocking(move || select(&prompt, choices)).await?
}

// Confirmation helpers
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    print_prompt(prompt);
    Ok(Confirm::new().default(default).interact()?)
}

pub async fn confirm_async(prompt: String, default: bool) -> Result<bool> {
    tokio::task::spawn_blocking(move || confirm(&prompt, default)).await?
}

// Table display helpers
pub fn table_path_display(path: &Path) -> String {
    let mut res = path.display().to_string();

    if let Some(home_dir) = dirs::home_dir() {
        res = res.replace(&home_dir.display().to_string(), "~");
    }

    res
}

// Internal helpers
fn print_prompt(prompt: &str) {
    println!(
        "{}",
        paris::formatter::colorize_string(format!("<yellow>?</> {prompt}:"))
    );
}
