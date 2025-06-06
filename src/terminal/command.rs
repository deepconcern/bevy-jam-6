const AVAILABLE_COMMANDS: [Command; 2] = [Command::Help, Command::List];

/// Commands to be interpreted by the terminal
///
/// When adding your own command, first add it here.
/// Then, add the name of the command (from the terminal's point of view) to the `parse` function below.
/// You'll also need to add your command to the `fmt::Display` implementation and the `AVAILABLE_COMMANDS` const so the "help" command can print it properly.
/// Finally, add the logic for your command in the `run` command.
#[derive(Debug)]
pub enum Command {
    List,
    Help,
    Invalid, // When we can't recognize the command
    Noop,    // For when the user presses enter without any input
}

impl Command {
    /// Parses the command from text input
    pub fn parse(input: &str) -> Command {
        match input.trim() {
            "" => Command::Noop,
            "?" => Command::Help,
            "ls" => Command::List,
            _ => Command::Invalid,
        }
    }

    // Command logic area
    pub fn run(&self, args: &[String]) -> Vec<String> {
        let mut output = Vec::new();

        match self {
            Command::Help => {
                if args.is_empty() {
                    output.push("Lol, can't remember your own commands?".to_string());
                    output.push(AVAILABLE_COMMANDS.map(|c| c.to_string()).join(" "));
                } else {
                    output.push(format!(
                        "{}: {}",
                        args[0],
                        match Command::parse(&args[0]) {
                            Command::Help => "Uh... You serious?",
                            Command::List => "List stuff. Like \"virus\" for viruses.",
                            _ => "Man... I don't even know! What nonsense are you asking me?",
                        }
                    ));
                }
            }
            Command::Invalid => output.push(format!(
                "Invalid command, dummy (type ? if you already forgot your own scripts): {}",
                args[0]
            )),
            Command::List => output.push("TODO".to_string()),
            Command::Noop => output.push(String::new()),
        }

        output
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Help => write!(f, "?"),
            Command::List => write!(f, "ls"),
            invalid_command => panic!(
                "Command '{:?}' is not meant to be stringified!",
                invalid_command
            ),
        }
    }
}
