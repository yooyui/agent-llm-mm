use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    Serve,
    Doctor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    fn unsupported_command(command: &str) -> Self {
        Self {
            message: format!("unsupported command: {command}"),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CliError {}

pub fn command_from_args<I>(args: I) -> Result<AppCommand, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program_name = args.next();

    match args.next().as_deref() {
        None | Some("serve") => Ok(AppCommand::Serve),
        Some("doctor") => Ok(AppCommand::Doctor),
        Some(other) => Err(CliError::unsupported_command(other)),
    }
}
