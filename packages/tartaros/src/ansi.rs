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

pub struct AnsiParser;

impl AnsiParser {
    pub fn parse(input: &str) -> Vec<AnsiCommand> {
        let mut commands = Vec::with_capacity(input.len() / 5);
        let mut current_text = String::with_capacity(64);
        let mut escape_start: Option<usize> = None;

        for (i, char) in input.chars().enumerate() {
            match char {
                '\x1B' => {
                    if escape_start.is_none() {
                        if !current_text.is_empty() {
                            AnsiParser::run_render_text(
                                &mut commands,
                                &current_text,
                            );
                            current_text.clear();
                        }
                        escape_start = Some(i);
                    }
                }
                'm' => {
                    if let Some(start) = escape_start {
                        // skip the '\x1B' and '[' characters
                        let sequence = &input[start + 2..i];

                        for num_str in sequence.split(';') {
                            if let Ok(num) = num_str.parse::<u8>() {
                                commands
                                    .push(AnsiParser::get_ansi_command(num));
                            }
                        }

                        escape_start = None;
                    } else {
                        current_text.push(char);
                    }
                }
                _ => {
                    if escape_start.is_none() {
                        current_text.push(char);
                    }
                }
            }
        }

        if !current_text.is_empty() {
            AnsiParser::run_render_text(&mut commands, &current_text);
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

    fn run_render_text(commands: &mut Vec<AnsiCommand>, current_text: &str) {
        let mut word_start: Option<usize> = None;

        for (i, c) in current_text.chars().enumerate() {
            if c == ' ' {
                commands.push(AnsiCommand::RenderText(" ".to_string()));

                if let Some(start) = word_start {
                    commands.push(AnsiCommand::RenderText(
                        current_text[start..i].to_string(),
                    ));
                    word_start = None;
                }
            } else if word_start.is_none() {
                word_start = Some(i);
            }
        }

        if let Some(start) = word_start {
            commands.push(AnsiCommand::RenderText(
                current_text[start..].to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_parse_ansi_string() {
        let input = "\x1b[31mhello\x1b[0m \x1b[34mworld\x1b[0m".to_string();
        let commands = AnsiParser::parse(&input);
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
