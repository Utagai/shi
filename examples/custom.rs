use shi::command::BaseCommand;
use shi::error::ShiError;
use shi::leaf;
use shi::shell::Shell;
use shi::Result as ShiResult;

use anyhow::Result;
struct PickyCommand {}

impl PickyCommand {
    #[allow(dead_code)]
    pub fn new() -> PickyCommand {
        PickyCommand {}
    }
}

impl BaseCommand for PickyCommand {
    type State = u64;

    fn name(&self) -> &str {
        "custom"
    }

    fn validate_args(&self, args: &[String]) -> ShiResult<()> {
        if args.len() != 3 {
            return Err(ShiError::general(format!(
                "expected 3 arguments, but got {}",
                args.len()
            )));
        }

        Ok(())
    }

    fn execute(&self, state: &mut u64, args: &[String]) -> ShiResult<String> {
        println!("I am a custom command! My state is: {:?}", state);
        *state += 1;

        for arg in args.iter() {
            println!("Argument: {}", arg)
        }

        Ok(String::from("custom command executed"))
    }
}

fn main() -> Result<()> {
    let counter: u64 = 0;

    let mut shell = Shell::new_with_state("| ", counter);

    shell.register(leaf!(PickyCommand::new()))?;

    shell.run()?;

    Ok(())
}
