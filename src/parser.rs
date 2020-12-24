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
    fn preprocess<'a>(&self, line: &'a str) -> Vec<&'a str>;
}

pub struct DefaultTokenizer {
    quotations: Vec<char>,
}

#[derive(Debug)]
struct QuoteLoc {
    pos: usize,
    quotation: char,
}

#[derive(Debug)]
struct QuotePair {
    start: usize,
    end: usize,
    quotation: char,
}

#[derive(Debug, PartialEq)]
enum Blob<'a> {
    Normal(&'a str),
    Quoted(&'a str),
}

/// Some shorthand functions for constructing Blobs.
impl<'a> Blob<'a> {
    /// Constructs a Normal blob.
    fn n(s: &str) -> Blob {
        Blob::Normal(s)
    }

    /// Constructs a Quoted blob.
    fn q(s: &str) -> Blob {
        Blob::Quoted(s)
    }
}

impl DefaultTokenizer {
    pub fn new(quotations: Vec<char>) -> DefaultTokenizer {
        DefaultTokenizer { quotations }
    }

    /// Finds quotes in the line string, and returns them.
    fn find_quotes(&self, line: &str) -> Vec<QuoteLoc> {
        let mut quote_locs: Vec<QuoteLoc> = Vec::new();

        for (i, ch) in line.char_indices() {
            if self.quotations.contains(&ch) {
                quote_locs.push(QuoteLoc {
                    pos: i,
                    quotation: ch,
                })
            }
        }

        quote_locs
    }

    /// Finds pairings of balanced quotes in the string, given a series of quote locations.
    fn find_quote_pairs(&self, quote_locs: Vec<QuoteLoc>) -> Vec<QuotePair> {
        let mut quote_pairs: Vec<QuotePair> = Vec::new();
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

        quote_pairs
    }

    /// Creates slices into the original line slice based on the given quote pairs.
    fn construct_slices_from_pairs<'a>(
        &self,
        line: &'a str,
        pairs: Vec<QuotePair>,
    ) -> Vec<Blob<'a>> {
        let mut blobs: Vec<Blob> = Vec::new();

        let mut cur = 0;
        // Now we have the pairs. Get the slices.
        for pair in pairs.iter() {
            if cur != pair.start {
                blobs.push(Blob::Normal(&line[cur..pair.start]));
            }
            blobs.push(Blob::Quoted(&line[pair.start..pair.end + 1]));
            cur = pair.end + 1;
        }

        if let Some(quote_pair) = pairs.last() {
            if quote_pair.end + 1 != line.len() {
                blobs.push(Blob::Normal(&line[quote_pair.end + 1..]));
            }
        }

        return blobs;
    }

    /// Globs together parts of the string that are surrounded by quotation marks.
    ///
    /// In practice for shi, this refers to ASCII " and ', but it is written generally for any set
    /// of quotation characters. I expect I may need to increase that list, maybe. Or extend this to
    /// be customizable by users.
    fn split_into_quote_blobs<'a>(&self, line: &'a str) -> Vec<Blob<'a>> {
        // This is not a particularly fast algorithm. But it doesn't need to be. Instead, we opt
        // for clarity.

        // First, identify where all the quotes are.
        let quote_locs = self.find_quotes(line);

        // Now, go through those quote locations and pair them accordingly.
        let quote_pairs = self.find_quote_pairs(quote_locs);

        // If no quotes matched, then just pretend we don't care (because we don't).
        if quote_pairs.is_empty() {
            return vec![Blob::Normal(line)];
        }

        // Finally, use the pair ranges to construct the individual slices.
        self.construct_slices_from_pairs(line, quote_pairs)
    }

    /// Splits the given blobs by spaces, so long as the blob is not a quoted blob, and returns the
    /// flattened vector of splits.
    fn split_by_space<'a>(&self, line_blobs: Vec<Blob<'a>>) -> Vec<&'a str> {
        let mut splitted_parts: Vec<&str> = Vec::new();
        for blob in line_blobs {
            match blob {
                Blob::Normal(s) => {
                    // Since this is not protected by surrounding quotes, we _do_ want to split
                    // this. We simply do it by space, and iterate the split result, adding them
                    // onto splitted parts. extend() helps us do this elegantly.
                    splitted_parts.extend(s.split(' '));
                }
                Blob::Quoted(s) => {
                    // We don't want to split inside the quote, so just add this immediately.
                    splitted_parts.push(s);
                }
            }
        }

        splitted_parts
    }
}

impl Tokenizer for DefaultTokenizer {
    fn preprocess<'a>(&self, line: &'a str) -> Vec<&'a str> {
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
                vec![
                    Blob::n("foo "),
                    Blob::q("'hi there!'"),
                    Blob::n(" btw hello")
                ]
            );
        }

        #[test]
        fn basic_double() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("foo \"hi there!\" btw hello"),
                vec![
                    Blob::n("foo "),
                    Blob::q("\"hi there!\""),
                    Blob::n(" btw hello")
                ]
            );
        }

        #[test]
        fn no_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("foo hi there! btw hello"),
                vec![Blob::n("foo hi there! btw hello")]
            );
        }

        #[test]
        fn quote_at_left() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("'foo hi' there! btw hello"),
                vec![Blob::q("'foo hi'"), Blob::n(" there! btw hello")]
            );
        }

        #[test]
        fn quote_at_right() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo hi'"),
                vec![Blob::n("there! btw hello "), Blob::q("'foo hi'")]
            );
        }

        #[test]
        fn single_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo hi"),
                vec![Blob::n("there! btw hello 'foo hi")]
            );
        }

        #[test]
        fn multiple_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defghijklmnopq\"rstuvwxyz|vvvv|v"),
                vec![
                    Blob::n("abc'defghijklmnopq\"rstuvwxyz"),
                    Blob::q("|vvvv|"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn one_success_amongst_dangling() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|', '-', '.']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defghi.jklmnopq\"rstu\"vwx-yz|vvvv|v"),
                vec![
                    Blob::n("abc'defghi.jklmnopq"),
                    Blob::q("\"rstu\""),
                    Blob::n("vwx-yz"),
                    Blob::q("|vvvv|"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn dangling_inside_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw hello 'foo\" hi'"),
                vec![Blob::n("there! btw hello "), Blob::q("'foo\" hi'")]
            );
        }

        #[test]
        fn dangling_after_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there! btw 'foo hi' \" hello"),
                vec![
                    Blob::n("there! btw "),
                    Blob::q("'foo hi'"),
                    Blob::n(" \" hello")
                ]
            );
        }

        #[test]
        fn dangling_before_matched_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("there!\" btw 'foo hi' hello"),
                vec![
                    Blob::n("there!\" btw "),
                    Blob::q("'foo hi'"),
                    Blob::n(" hello")
                ]
            );
        }

        #[test]
        fn multiple_non_overlapping_pairs() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz"),
                vec![
                    Blob::n("abc"),
                    Blob::q("'defg'"),
                    Blob::n("hijk"),
                    Blob::q("'lmno'"),
                    Blob::n("pqr"),
                    Blob::q("'stuvwx'"),
                    Blob::n("yz")
                ]
            );
        }

        #[test]
        fn many_kinds_of_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec![
                    Blob::n("abc"),
                    Blob::q("'defg'"),
                    Blob::n("hijk"),
                    Blob::q("'lmno'"),
                    Blob::n("pqr"),
                    Blob::q("'stuvwx'"),
                    Blob::n("yz"),
                    Blob::q("|vvvv|"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn only_one_quote() {
            let preproc = DefaultTokenizer::new(vec!['|']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec![
                    Blob::n("abc'defg'hijk'lmno'pqr'stuvwx'yz"),
                    Blob::q("|vvvv|"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn multiple_kinds_of_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("abc'defg'hijklmnopqr\"stuvwx\"yz"),
                vec![
                    Blob::n("abc"),
                    Blob::q("'defg'"),
                    Blob::n("hijklmnopqr"),
                    Blob::q("\"stuvwx\""),
                    Blob::n("yz")
                ]
            );
        }

        #[test]
        fn empty_string() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(preproc.split_into_quote_blobs(""), vec![Blob::n("")]);
        }

        #[test]
        fn only_quotes() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("''''''''"),
                vec![Blob::q("''"), Blob::q("''"), Blob::q("''"), Blob::q("''")]
            );
        }

        #[test]
        fn mixture_of_only_quotes() {
            let preproc = DefaultTokenizer::new(vec!['|', '\'']);
            assert_eq!(
                preproc.split_into_quote_blobs("''||'|''|'||''|'|"),
                vec![
                    Blob::q("''"),
                    Blob::q("||"),
                    Blob::q("'|'"),
                    Blob::q("'|'"),
                    Blob::q("||"),
                    Blob::q("''"),
                    Blob::q("|'|")
                ]
            );
        }
    }

    mod split_by_space {
        use super::*;

        #[test]
        fn empty_string() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            let empty_vec: Vec<&str> = Vec::new();
            assert_eq!(preproc.split_by_space(vec![]), empty_vec);
        }

        #[test]
        fn empty_blob() {
            // I don't think this is actually ever possible if we take blobs from
            // split_into_quote_blobs(), but whatever.
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(preproc.split_by_space(vec![Blob::n("")]), vec![""]);
        }

        #[test]
        fn only_normals() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![Blob::n("hi there"), Blob::n("euler is cool")]),
                vec!["hi", "there", "euler", "is", "cool"]
            );
        }

        #[test]
        fn only_quoteds() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![Blob::q("'hi there'"), Blob::q("'euler is cool'")]),
                vec!["'hi there'", "'euler is cool'"]
            );
        }

        #[test]
        fn quoted_then_normal() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![Blob::q("'hi there!'"), Blob::n("euler is cool")]),
                vec!["'hi there!'", "euler", "is", "cool"]
            );
        }

        #[test]
        fn normal_then_quoted() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![Blob::n("euler is cool"), Blob::q("'hi there!'")]),
                vec!["euler", "is", "cool", "'hi there!'"]
            );
        }

        #[test]
        fn quoted_surrounded_by_normals() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![
                    Blob::n("euler is cool"),
                    Blob::q("'hi there!'"),
                    Blob::n("euler is cool")
                ]),
                vec!["euler", "is", "cool", "'hi there!'", "euler", "is", "cool"]
            );
        }

        #[test]
        fn normal_surrounded_by_quoteds() {
            let preproc = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                preproc.split_by_space(vec![
                    Blob::q("'hi there!'"),
                    Blob::n("euler is cool"),
                    Blob::q("'hi there!'")
                ]),
                vec!["'hi there!'", "euler", "is", "cool", "'hi there!'"]
            );
        }
    }
}
