use dialoguer::{Confirm, Input, Select};
use eyre::Result;
use std::{borrow::Cow, path::Path};
use tabled::{Table, Tabled};

// TODO: make primarily async to avoid copies

// Prompting helpers
pub fn prompt(prompt: &str, default: Option<String>) -> Result<String> {
    let prompt = match default.as_deref() {
        Some("") => Cow::Owned(format!("{prompt} (optional)")),
        Some(default) => Cow::Owned(format!("{prompt} (default: {default})")),
        None => Cow::Borrowed(prompt),
    };
    print_prompt(&prompt);

    let mut input = Input::<String>::new();
    input.with_prompt("").show_default(false);

    if let Some(default) = default {
        input.default(default);
    }

    Ok(input.interact_text()?.trim().to_owned())
}

pub async fn prompt_async(
    text: String,
    default: Option<String>,
) -> Result<String> {
    tokio::task::spawn_blocking(move || prompt(&text, default)).await?
}

// Selection helpers
pub fn select(prompt: &str, choices: &[&str]) -> Result<usize> {
    print_prompt(prompt);

    let res = Select::new().items(choices).default(0).interact()?;
    eprintln!("> {}", choices[res]);
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

// Table helpers
pub fn table<T: Tabled>(rows: impl IntoIterator<Item = T>) -> Table {
    Table::new(rows).with(tabled::Style::psql())
}

pub fn table_path_display(path: &Path) -> String {
    let mut res = path.display().to_string();

    if let Some(home_dir) = dirs::home_dir() {
        res = res.replace(&home_dir.display().to_string(), "~");
    }

    res
}

// Dispatch macros
macro_rules! dispatch {
    ($on:expr, $args:tt => {$($option:path),+}) => {
        match $on {
            $($option (ref cmd) => dispatch!(@apply cmd => $args)),+
        }
    };

    (@apply $cmd:expr => ($($args:expr),*)) => {{
        use tracing_futures::WithSubscriber;
        $cmd.run($($args),*).with_current_subscriber().await
    }};
}

// Internal helpers
fn print_prompt(prompt: &str) {
    println!(
        "{}",
        paris::formatter::colorize_string(format!("<yellow>?</> {prompt}:"))
    );
}
