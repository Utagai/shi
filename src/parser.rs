use crate::tokenizer::DefaultTokenizer;

struct Parser {
    tokenizer: DefaultTokenizer,
}

enum CommandType {
    Builtin,
    Custom,
}

struct Outcome<'a> {
    cmd_path: Vec<&'a str>,
    cmd_type: CommandType,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokenizer: DefaultTokenizer::new(vec!['\'', '"']),
        }
    }

    pub fn parse(&self, _: &str) -> Outcome {
        Outcome {
            cmd_path: Vec::new(),
            cmd_type: CommandType::Custom,
        }
    }
}
