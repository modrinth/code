use std::fs;

use regex::Regex;

fn main() {
    // Build the Tauri app
    tauri_build::build();

    // Check that all JavaScript 'invoke' Tauri functions have a corresponding tagged Rust function
    // This is to prevent the app from crashing if a JavaScript function is invoked but the corresponding Rust function is not tagged
    // This only allows simple functions, but functions in theseus_gui should be kept simple
    check_invoke_sanity();
}

fn check_invoke_sanity() {
    let js_files = read_js_files("../src/helpers");
    let rust_files = read_rust_files("src");

    let js_function_names = extract_js_function_names(&js_files);
    let rust_function_names = extract_rust_function_names(&rust_files);

    let mut missing_functions = Vec::new();
    for js_fn_name in js_function_names {
        if !rust_function_names.contains(&js_fn_name) {
            missing_functions.push(js_fn_name);
        }
    }
    if !missing_functions.is_empty() {
        panic!(
            "The following invoked Tauri functions do not have corresponding Rust functions with #[tauri::command] attribute :\n{}",
            missing_functions.join("\n")
        );
    }
}

fn read_js_files(directory: &str) -> Vec<String> {
    let mut files = Vec::new();
    read_files_recursively(directory, "js", &mut files);
    files
}

fn read_rust_files(directory: &str) -> Vec<String> {
    let mut files = Vec::new();
    read_files_recursively(directory, "rs", &mut files);
    files
}

// Recursive in case we make the helpers directory more complex
fn read_files_recursively(
    directory: &str,
    extension: &str,
    files: &mut Vec<String>,
) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            read_files_recursively(&path.to_string_lossy(), extension, files);
        } else if path.extension().map_or(false, |ext| ext == extension) {
            let content = fs::read_to_string(path).unwrap();
            files.push(content);
        }
    }
}

fn extract_rust_function_names(rust_files: &[String]) -> Vec<String> {
    // Matches #[tauri::command] attribute
    let re_tauri_command = Regex::new(r"(?m)#\[tauri::command\]").unwrap();
    // Matches function name following the #[tauri::command] attribute
    // Matches up to the first (, to allow for function arguments and comments in that area
    let re_function_name =
        Regex::new(r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    let mut function_names = Vec::new();

    for file in rust_files {
        let mut start = 0;
        while let Some(command_match) = re_tauri_command.find_at(file, start) {
            if let Some(function_name_cap) =
                re_function_name.captures(&file[command_match.end()..])
            {
                function_names.push(function_name_cap[1].to_string());
                start = command_match.start() + 1;
            } else {
                break;
            }
        }
    }

    function_names
}

fn extract_js_function_names(js_files: &[String]) -> Vec<String> {
    // Matches functions of the form: invoke('function_name', { ... }) (or invoke('function_name') )
    let re_invoke = Regex::new(
        r"(?m)invoke\(\s*'([a-zA-Z_][a-zA-Z0-9_]*)'\s*(?:,\s*\{.*?\})?\s*\)",
    )
    .unwrap();
    let mut function_names = Vec::new();

    for file in js_files {
        let mut start = 0;
        while let Some(invoke_match) = re_invoke.find_at(file, start) {
            if let Some(captures) = re_invoke.captures(invoke_match.as_str()) {
                function_names.push(captures[1].to_string());
            }
            start = invoke_match.start() + 1;
        }
    }

    function_names
}
