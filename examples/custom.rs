use shi::command::{BaseCommand, Completion};
use shi::error::ShiError;
use shi::leaf;
use shi::shell::Shell;
use shi::Result as ShiResult;

use anyhow::Result;
struct PickyCommand {}

static PI_DIGITS: [u8; 500] = [
    1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3, 2, 3, 8, 4, 6, 2, 6, 4, 3, 3, 8, 3, 2, 7, 9, 5, 0,
    2, 8, 8, 4, 1, 9, 7, 1, 6, 9, 3, 9, 9, 3, 7, 5, 1, 0, 5, 8, 2, 0, 9, 7, 4, 9, 4, 4, 5, 9, 2, 3,
    0, 7, 8, 1, 6, 4, 0, 6, 2, 8, 6, 2, 0, 8, 9, 9, 8, 6, 2, 8, 0, 3, 4, 8, 2, 5, 3, 4, 2, 1, 1, 7,
    0, 6, 7, 9, 8, 2, 1, 4, 8, 0, 8, 6, 5, 1, 3, 2, 8, 2, 3, 0, 6, 6, 4, 7, 0, 9, 3, 8, 4, 4, 6, 0,
    9, 5, 5, 0, 5, 8, 2, 2, 3, 1, 7, 2, 5, 3, 5, 9, 4, 0, 8, 1, 2, 8, 4, 8, 1, 1, 1, 7, 4, 5, 0, 2,
    8, 4, 1, 0, 2, 7, 0, 1, 9, 3, 8, 5, 2, 1, 1, 0, 5, 5, 5, 9, 6, 4, 4, 6, 2, 2, 9, 4, 8, 9, 5, 4,
    9, 3, 0, 3, 8, 1, 9, 6, 4, 4, 2, 8, 8, 1, 0, 9, 7, 5, 6, 6, 5, 9, 3, 3, 4, 4, 6, 1, 2, 8, 4, 7,
    5, 6, 4, 8, 2, 3, 3, 7, 8, 6, 7, 8, 3, 1, 6, 5, 2, 7, 1, 2, 0, 1, 9, 0, 9, 1, 4, 5, 6, 4, 8, 5,
    6, 6, 9, 2, 3, 4, 6, 0, 3, 4, 8, 6, 1, 0, 4, 5, 4, 3, 2, 6, 6, 4, 8, 2, 1, 3, 3, 9, 3, 6, 0, 7,
    2, 6, 0, 2, 4, 9, 1, 4, 1, 2, 7, 3, 7, 2, 4, 5, 8, 7, 0, 0, 6, 6, 0, 6, 3, 1, 5, 5, 8, 8, 1, 7,
    4, 8, 8, 1, 5, 2, 0, 9, 2, 0, 9, 6, 2, 8, 2, 9, 2, 5, 4, 0, 9, 1, 7, 1, 5, 3, 6, 4, 3, 6, 7, 8,
    9, 2, 5, 9, 0, 3, 6, 0, 0, 1, 1, 3, 3, 0, 5, 3, 0, 5, 4, 8, 8, 2, 0, 4, 6, 6, 5, 2, 1, 3, 8, 4,
    1, 4, 6, 9, 5, 1, 9, 4, 1, 5, 1, 1, 6, 0, 9, 4, 3, 3, 0, 5, 7, 2, 7, 0, 3, 6, 5, 7, 5, 9, 5, 9,
    1, 9, 5, 3, 0, 9, 2, 1, 8, 6, 1, 1, 7, 3, 8, 1, 9, 3, 2, 6, 1, 1, 7, 9, 3, 1, 0, 5, 1, 1, 8, 5,
    4, 8, 0, 7, 4, 4, 6, 2, 3, 7, 9, 9, 6, 2, 7, 4, 9, 5, 6, 7, 3, 5, 1, 8, 8, 5, 7, 5, 2, 7, 2, 4,
    8, 9, 1, 2, 2, 7, 9, 3, 8, 1, 8, 3, 0, 1, 1, 9, 4, 9, 1, 2,
];

static PI_STR: &str = "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458700660631558817488152092096282925409171536436789259036001133053054882046652138414695194151160943305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912";

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

    fn autocomplete(&self, args: Vec<&str>, trailing_space: bool) -> Completion {
        // Counts up from 1 to 3, after which it begins returning the digits of pi one by one.
        // This code may very well have bugs and edge cases, but since it is an example, I don't
        // really care.
        match args.last() {
            None => Completion::Possibilities(vec![String::from("1")]),
            Some(last) => {
                if let Ok(last_num) = last.parse::<f32>() {
                    if last_num >= 3.0 {
                        if trailing_space {
                            Completion::Nothing
                        } else if PI_STR.starts_with(last) {
                            if *last == "3" {
                                Completion::PartialArgCompletion(vec![".".to_string()])
                            } else {
                                Completion::PartialArgCompletion(vec![
                                    PI_DIGITS[last.len() - 2].to_string()
                                ])
                            }
                        } else {
                            Completion::Nothing
                        }
                    } else {
                        Completion::Possibilities(vec![(last_num + 1.0).to_string()])
                    }
                } else {
                    Completion::Nothing
                }
            }
        }
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

    let mut shell = Shell::new_with_state("| ", counter)?;

    shell.register(leaf!(PickyCommand::new()))?;

    shell.run()?;

    Ok(())
}
