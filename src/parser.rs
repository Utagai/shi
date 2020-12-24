// Preprocesses the string into a vector of &str tokens for the Parser. Effectively a tokenizer,
// but it doesn't actually really emit a variety of tokens, but serves a similar purpose to a
// tokenizer.
// Handles things like splitting by space, acknowledging quotation marks, etc.
struct Parser {}

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
        Parser {}
    }

    pub fn parse(&self, line: &str) -> Outcome {
        Outcome {
            cmd_path: Vec::new(),
            cmd_type: CommandType::Custom,
        }
    }
}

trait Tokenizer {
    fn preprocess(&self, line: &str) -> Vec<&str>;
}

pub struct DefaultTokenizer {
    quotations: Vec<char>,
}

impl DefaultTokenizer {
    pub fn new(quotations: Vec<char>) -> DefaultTokenizer {
        DefaultTokenizer { quotations }
    }

    // Globs together parts of the string that are surrounded by quotation marks.
    // In practice for shi, this refers to ASCII " and '.
    fn split_into_quote_blobs<'a>(&self, line: &'a str) -> Vec<&'a str> {
        // This is not a particularly fast algorithm. But it doesn't need to be. Instead, we opt
        // for clarity.

        // First, identify where all the quotes are.
        #[derive(Debug)]
        struct QuoteLoc {
            pos: usize,
            quotation: char,
        }
        let mut quote_locs: Vec<QuoteLoc> = Vec::new();

        for (i, ch) in line.char_indices() {
            if self.quotations.contains(&ch) {
                quote_locs.push(QuoteLoc {
                    pos: i,
                    quotation: ch,
                })
            }
        }

        #[derive(Debug)]
        struct QuotePair {
            start: usize,
            end: usize,
            quotation: char,
        }
        let mut quote_pairs: Vec<QuotePair> = Vec::new();

        // Now, go through those quote locations and pair them accordingly.
        let mut start_idx = 0;
        let mut next_idx = None;
        while start_idx < quote_locs.len() {
            let start = quote_locs.get(start_idx).unwrap();
            for i in start_idx + 1..quote_locs.len() {
                let current = quote_locs.get(i).unwrap();

                if current.quotation == start.quotation {
                    quote_pairs.push(QuotePair {
                        start: start.pos,
                        end: current.pos,
                        quotation: current.quotation,
                    });
                    next_idx = Some(i + 1);
                    break;
                } else {
                    if next_idx.is_none() {
                        next_idx = Some(i)
                    }
                }
            }

            if let Some(idx) = next_idx {
                start_idx = idx;
            } else {
                break;
            }
            next_idx = None;
        }

        if quote_pairs.is_empty() {
            // If no quotes matched, then just pretend we don't care.
            return vec![line];
        }

        // Finally, use the pair ranges to construct the individual slices.
        let mut blobs: Vec<&str> = Vec::new();

        let mut cur = 0;
        // Now we have the pairs. Get the slices.
        for pair in quote_pairs.iter() {
            if cur != pair.start {
                blobs.push(&line[cur..pair.start]);
            }
            blobs.push(&line[pair.start..pair.end + 1]);
            cur = pair.end + 1;
        }

        if let Some(quote_pair) = quote_pairs.last() {
            if quote_pair.end + 1 != line.len() {
                blobs.push(&line[quote_pair.end + 1..]);
            }
        }

        blobs
    }

    fn split_by_space(&self, line_bits: Vec<&str>) -> Vec<&str> {
        Vec::new()
    }
}

impl Tokenizer for DefaultTokenizer {
    fn preprocess(&self, line: &str) -> Vec<&str> {
        let line_bits_with_quotes_globbed = self.split_into_quote_blobs(line);

        self.split_by_space(line_bits_with_quotes_globbed)
    }
}

#[cfg(test)]
mod preprocess_tests {
    use super::*;

    mod glob_quotes {
        use super::*;

        #[test]
        fn basic_single() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("foo 'hi there!' btw hello"),
                vec!["foo ", "'hi there!'", " btw hello"]
            );
        }

        #[test]
        fn basic_double() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("foo \"hi there!\" btw hello"),
                vec!["foo ", "\"hi there!\"", " btw hello"]
            );
        }

        #[test]
        fn no_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("foo hi there! btw hello"),
                vec!["foo hi there! btw hello"]
            );
        }

        #[test]
        fn quote_at_left() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("'foo hi' there! btw hello"),
                vec!["'foo hi'", " there! btw hello"]
            );
        }

        #[test]
        fn quote_at_right() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo hi'"),
                vec!["there! btw hello ", "'foo hi'"]
            );
        }

        #[test]
        fn single_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo hi"),
                vec!["there! btw hello 'foo hi"]
            );
        }

        #[test]
        fn multiple_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defghijklmnopq\"rstuvwxyz|vvvv|v"),
                vec!["abc'defghijklmnopq\"rstuvwxyz", "|vvvv|", "v"]
            );
        }

        #[test]
        fn one_success_amongst_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|', '-', '.']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defghi.jklmnopq\"rstu\"vwx-yz|vvvv|v"),
                vec!["abc'defghi.jklmnopq", "\"rstu\"", "vwx-yz", "|vvvv|", "v"]
            );
        }

        #[test]
        fn dangling_inside_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo\" hi'"),
                vec!["there! btw hello ", "'foo\" hi'"]
            );
        }

        #[test]
        fn dangling_after_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw 'foo hi' \" hello"),
                vec!["there! btw ", "'foo hi'", " \" hello"]
            );
        }

        #[test]
        fn dangling_before_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there!\" btw 'foo hi' hello"),
                vec!["there!\" btw ", "'foo hi'", " hello"]
            );
        }

        #[test]
        fn multiple_non_overlapping_pairs() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz"),
                vec!["abc", "'defg'", "hijk", "'lmno'", "pqr", "'stuvwx'", "yz"]
            );
        }

        #[test]
        fn many_kinds_of_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec!["abc", "'defg'", "hijk", "'lmno'", "pqr", "'stuvwx'", "yz", "|vvvv|", "v"]
            );
        }

        #[test]
        fn only_one_quote() {
            let preproc = DefaultTokenizer::new(vec!['|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec!["abc'defg'hijk'lmno'pqr'stuvwx'yz", "|vvvv|", "v"]
            );
        }

        #[test]
        fn multiple_kinds_of_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijklmnopqr\"stuvwx\"yz"),
                vec!["abc", "'defg'", "hijklmnopqr", "\"stuvwx\"", "yz"]
            );
        }

        #[test]
        fn empty_string() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(preproc.split_into_quote_blobs(""), vec![""]);
        }

        #[test]
        fn only_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("''''''''"),
                vec!["''", "''", "''", "''"]
            );
        }

        #[test]
        fn mixture_of_only_quotes() {
            let preproc = DefaultTokenizer::new(vec!['|', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("''||'|''|'||''|'|"),
                vec!["''", "||", "'|'", "'|'", "||", "''", "|'|"]
            );
        }
    }

    // #[test]
    // fn test_split_by_space() {
    //     let preproc = DefaultTokenizer::new(vec!['"', '\'']);
    //     let empty_vec: Vec<&str> = Vec::new();
    //     assert_eq!(
    //         preproc.split_by_space(vec!["foo", "'hi there'", " btw hello"]),
    //         vec!["foo", "'hi there'", "btw", "hello"]
    //     );
    // }
}
