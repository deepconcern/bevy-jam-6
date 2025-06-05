const AVAILABLE_COMMANDS: [Command; 2] = [Command::Help, Command::List];

#[derive(Debug)]
pub enum Command {
    List,
    Help,
    Invalid,
    Noop,
}

impl Command {
    pub fn parse(input: &str) -> Command {
        match input.trim() {
            "" => Command::Noop,
            "?" => Command::Help,
            "ls" => Command::List,
            _ => Command::Invalid,
        }
    }

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
