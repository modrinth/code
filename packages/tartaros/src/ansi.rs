#[derive(Debug, PartialEq, Clone)]
pub enum AnsiControl {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl AnsiControl {
    pub fn to_color(&self) -> String {
        match self {
            AnsiControl::Black => "black".to_string(),
            AnsiControl::Red => "red".to_string(),
            AnsiControl::Green => "green".to_string(),
            AnsiControl::Yellow => "orange".to_string(),
            AnsiControl::Blue => "blue".to_string(),
            AnsiControl::Magenta => "magenta".to_string(),
            AnsiControl::Cyan => "cyan".to_string(),
            AnsiControl::White => "gray".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnsiCommand {
    ModifyStyle(AnsiControl),
    RenderText(String),
    Reset,
}

pub struct AnsiParser {
    input: String,
}

impl AnsiParser {
    pub fn new(input: String) -> Self {
        AnsiParser { input }
    }

    pub fn parse(&self) -> Vec<AnsiCommand> {
        let mut commands = Vec::new();
        let chars = self.input.chars().collect::<Vec<char>>();
        let mut i = 0;

        let mut current_text = String::new();

        while i < chars.len() {
            match chars[i] {
                '\x1b' => {
                    // read until 'm', which is the end of the escape sequence
                    let start = i.clone();
                    while i < chars.len() && chars[i] != 'm' {
                        i += 1;
                    }
                    let escape_sequence = &chars[start..i];
                    i += 1;
                    if escape_sequence.len() < 3 {
                        continue;
                    }
                    if !current_text.is_empty() {
                        AnsiParser::run_render_text(&mut commands, &mut current_text);
                    }
                    let escape_sequence = &escape_sequence[2..];
                    let control_characters = escape_sequence
                        .iter()
                        .collect::<String>()
                        .split(";")
                        .filter_map(|x| x.parse::<u8>().ok())
                        .collect::<Vec<u8>>();

                    for control_character in control_characters {
                        commands.push(AnsiParser::get_ansi_command(control_character))
                    }
                }
                _ => {
                    while i < chars.len() && chars[i] != '\x1b' {
                        current_text.push(chars[i]);
                        i += 1;
                    }
                }
            }
        }

        if !current_text.is_empty() {
            AnsiParser::run_render_text(&mut commands, &mut current_text);
        }

        commands
    }

    fn get_ansi_command(control_character: u8) -> AnsiCommand {
        match control_character {
            30 => AnsiCommand::ModifyStyle(AnsiControl::Black),
            31 => AnsiCommand::ModifyStyle(AnsiControl::Red),
            32 => AnsiCommand::ModifyStyle(AnsiControl::Green),
            33 => AnsiCommand::ModifyStyle(AnsiControl::Yellow),
            34 => AnsiCommand::ModifyStyle(AnsiControl::Blue),
            35 => AnsiCommand::ModifyStyle(AnsiControl::Magenta),
            36 => AnsiCommand::ModifyStyle(AnsiControl::Cyan),
            37 => AnsiCommand::ModifyStyle(AnsiControl::White),
            0 => AnsiCommand::Reset,
            _ => AnsiCommand::Reset,
        }
    }

    fn run_render_text(commands: &mut Vec<AnsiCommand>, current_text: &mut String) {
        let mut text = String::new();
        for c in current_text.chars() {
            if c == ' ' {
                if !text.is_empty() {
                    commands.push(AnsiCommand::RenderText(text.clone()));
                    text.clear();
                }
                commands.push(AnsiCommand::RenderText(" ".to_string()));
            } else {
                text.push(c);
            }
        }
        if !text.is_empty() {
            commands.push(AnsiCommand::RenderText(text.clone()));
        }
        current_text.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_parse_ansi_string() {
        let input = "\x1b[31mhello\x1b[0m \x1b[34mworld\x1b[0m".to_string();
        let parser = AnsiParser::new(input);
        let commands = parser.parse();
        println!("{:?}", commands);
        assert_eq!(commands.len(), 7);
        assert_eq!(commands[0], AnsiCommand::ModifyStyle(AnsiControl::Red));
        assert_eq!(commands[1], AnsiCommand::RenderText("hello".to_string()));
        assert_eq!(commands[2], AnsiCommand::Reset);
        assert_eq!(commands[3], AnsiCommand::RenderText(" ".to_string()));
        assert_eq!(commands[4], AnsiCommand::ModifyStyle(AnsiControl::Blue));
        assert_eq!(commands[5], AnsiCommand::RenderText("world".to_string()));
        assert_eq!(commands[6], AnsiCommand::Reset);
    }
}
