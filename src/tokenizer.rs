pub struct Tokenization<'a> {
    pub tokens: Vec<&'a str>,
    pub trailing_space: bool,
}

/// Tokenizers pre-process the string into a vector of &str tokens for a parser. These tokens are
/// essentially a way to split apart a line into command and arguments. Effectively a tokenizer,
/// but it doesn't necessarily emit a variety of tokens, but serves a purpose similar to a
/// tokenizer, or I suppose, at least a scanner?
pub trait Tokenizer {
    // Tokenize returns a vector of tokens (&str), and a bool to indicate if there was a trailing
    // space.
    fn tokenize<'a>(&self, line: &'a str) -> Tokenization<'a>;
}

/// DefaultTokenizer tokenizes an input string into tokens based on some default, basic rules.
///
/// Handles things like splitting by space, acknowledging quotation marks, etc.
pub struct DefaultTokenizer {
    quotations: Vec<char>,
}

#[derive(Debug, PartialEq)]
/// Describes the position of a quotation mark.
///
/// Quotation marks are generally either `"` or `'`, but can be any character.
struct QuoteLoc {
    pos: usize,
    quotation: char,
}

#[derive(Debug)]
/// Describes a pair of quotes.
///
/// Quotation marks are generally either `"` or `'`, but can be any character.
struct QuotePair {
    start: usize,
    end: usize,
    _quotation: char,
}

#[derive(Debug, PartialEq)]
/// Describes a 'blob' of the input string.
///
/// A 'blob' can be thought of as a chunk or portion of the string. It can be defined as quoted
/// chunks of the string, or non-quoted chunks, with no other cases. Blobs are contiguous and thus
/// do not overlap.
enum Blob<'a> {
    Normal(&'a str),
    Quoted(&'a str),
}

/// Some shorthand functions for constructing Blobs.
#[cfg(test)]
impl<'a> Blob<'a> {
    /// Constructs a Normal blob.
    fn n(s: &'a str) -> Blob<'a> {
        Blob::Normal(s)
    }

    /// Constructs a Quoted blob.
    fn q(s: &'a str) -> Blob<'a> {
        Blob::Quoted(s)
    }
}

impl DefaultTokenizer {
    /// Constructs a `DefaultTokenizer`.
    pub fn new(quotations: Vec<char>) -> DefaultTokenizer {
        DefaultTokenizer { quotations }
    }

    /// Finds quotes in the line string, and returns them.
    ///
    /// This method does not have any intelligence around pairing of quotation marks, it simply
    /// finds and returns the ones it sees.
    ///
    /// # Arguments
    /// `line` - The input line.
    ///
    /// # Returns
    /// `Vec<QuoteLoc>` - A listing of all the quotation marks. Pairs represent two elements in
    /// this listing.
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
    ///
    /// This method is the intelligent sibling of `find_quotes()`. It takes the `QuoteLoc`'s
    /// returned by `find_quotes()` and pairs together the `QuoteLoc`'s into `QuotePair`'s.
    ///
    /// # Arguments
    /// `quote_locs` - The quote locations as returned by `find_quotes()`.
    ///
    /// # Returns
    /// `Vec<QuotePair>` - The paired couples of quotes based on the given quote locations.
    fn find_quote_pairs(&self, quote_locs: Vec<QuoteLoc>) -> Vec<QuotePair> {
        let mut quote_pairs: Vec<QuotePair> = Vec::new();
        let mut start_idx = 0;
        let mut next_idx = None;

        // The algorithm here is that we will go through each of the quote locations, and for each
        // of them, we will iterate the rest of the quote locations until we find a matching
        // quotation character, upon which we will discard any quotations in between (since they
        // are actually contained within the outer quotes), and add this pair.
        //
        // Then beginning from after the second QuoteLoc of the pair, we repeat until we've
        // exhausted all the QuoteLocs.
        while start_idx < quote_locs.len() {
            // This .unwrap() is safe, because of the while condition.
            let start = quote_locs.get(start_idx).unwrap();
            for i in start_idx + 1..quote_locs.len() {
                // This .unwrap() is safe, because of the for loops range being upper bounded by
                // quote_locs.len() exclusively. For the lower bound, we know that start_idx+1 is
                // within bounds, because of the outer while condition. If adding 1 brings it to
                // quote_locs.len(), that would exceed the for range and this code would not be
                // executed.
                let current = quote_locs.get(i).unwrap();

                if current.quotation == start.quotation {
                    quote_pairs.push(QuotePair {
                        start: start.pos,
                        end: current.pos,
                        _quotation: current.quotation,
                    });
                    next_idx = Some(i + 1);
                    break;
                }

                if next_idx.is_none() {
                    next_idx = Some(i)
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

    /// Creates blobs from the original line based on the given quote pairs.
    ///
    /// This function essentially breaks apart the line into quoted and non-quoted pieces.
    ///
    /// # Arguments
    /// `line` - The input line.
    /// `pairs` - The listing of quote pairs.
    ///
    /// # Returns
    /// `Vec<Blob>` - The listing of quoted & non-quoted blobs of the input line.
    fn construct_slices_from_pairs<'a>(
        &self,
        line: &'a str,
        pairs: Vec<QuotePair>,
    ) -> Vec<Blob<'a>> {
        let mut blobs: Vec<Blob> = Vec::new();

        let mut cur = 0;
        // Now we have the pairs. Get the slices.
        for pair in pairs.iter() {
            // If the current position does not match the pair.start, that means that the region of
            // the input from cur to pair.start is itself a blob, and it's unquoted. Let's make
            // sure we don't forget that.
            if cur != pair.start {
                blobs.push(Blob::Normal(&line[cur..pair.start]));
            }

            // Of course, the quote pair describes a blob by its region in the line.
            blobs.push(Blob::Quoted(&line[pair.start + 1..pair.end]));
            cur = pair.end + 1;
        }

        // If a quote pair does not end at the end of a line (aka, the second quotation character
        // in the pair is not the last character of the line), then that means there is an extra
        // unquoted blob at the end of the line that we forgot about. Let's remember that here.
        if let Some(quote_pair) = pairs.last() {
            if quote_pair.end + 1 != line.len() {
                blobs.push(Blob::Normal(&line[quote_pair.end + 1..]));
            }
        }

        blobs
    }

    /// Globs together parts of the string that are surrounded by quotation marks, and returns a
    /// series of blobs of the input line based on it.
    ///
    /// In practice for shi, this refers to ASCII " and ', but it is written generally for any set
    /// of quotation characters.
    ///
    /// # Arguments
    /// `line` - The input line.
    ///
    /// # Returns
    /// `Vec<Blob>` - The listing of quoted & non-quoted blobs of the input line.
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

    /// Splits the given blobs by spaces, and returns the flattened vector of splits.
    ///
    /// The key thing to note here is that a _quoted_ blob is not split, and maintained.
    /// Whereas non-quoted blobs are split by space.
    ///
    /// # Arguments
    /// `line_blobs` - The blobs of an input line.
    ///
    /// # Returns
    /// `Vec<&str>` - A series of slices into an input line that represent its component tokens.
    fn split_by_space<'a>(&self, line_blobs: Vec<Blob<'a>>) -> Vec<&'a str> {
        let mut splitted_parts: Vec<&str> = Vec::new();
        for blob in line_blobs {
            match blob {
                Blob::Normal(s) => {
                    // Since this is not protected by surrounding quotes, we _do_ want to split
                    // this. We simply do it by space, and iterate the split result, adding them
                    // onto splitted parts. extend() helps us do this elegantly.
                    // Small note though: we don't want to add empty strings, since they are
                    // meaningless and are likely just the result of trailing/leading whitespace.
                    splitted_parts.extend(s.split(' ').filter(|s| !s.is_empty()));
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

#[cfg(test)]
#[test]
fn test_find_quotes() {
    let tokenizer = DefaultTokenizer::new(vec!['\'']);
    let quote_locs = tokenizer.find_quotes("hello 'how are' you?");
    assert_eq!(
        quote_locs,
        vec![
            QuoteLoc {
                pos: 6,
                quotation: '\''
            },
            QuoteLoc {
                pos: 14,
                quotation: '\''
            }
        ]
    );
}

impl Tokenizer for DefaultTokenizer {
    /// Tokenizes the given input line into its constituent components.
    ///
    /// In particular, this preserves quoted strings and does not split inside of them, but
    /// outside, splits them, by space.
    ///
    /// # Arguments
    /// `line` - The input line.
    ///
    /// # Returns
    /// `Vec<&str>` - A series of slices into an input line that represent its component tokens.
    fn tokenize<'a>(&self, line: &'a str) -> Tokenization<'a> {
        let line_bits_with_quotes_globbed = self.split_into_quote_blobs(line);

        Tokenization {
            tokens: self.split_by_space(line_bits_with_quotes_globbed),
            trailing_space: line.ends_with(' '),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Since we test the two functions that comprise this individually, and the implementation of
    // this function is just a composition, most of the coverage is already handled.
    // So, we give one big and complex case.
    #[test]
    fn tokenize() {
        use pretty_assertions::assert_eq;
        let tokenizer = DefaultTokenizer::new(vec!['"', '\'', '|', '-']);
        assert_eq!(
            tokenizer.tokenize(
                "bar 'foo is here' and quux is not\n necessarily 'here'\" b\"ut you co|uld say 'there'-",
            ).tokens,
            vec![
                "bar",
                "foo is here",
                "and",
                "quux",
                "is",
                "not\n",
                "necessarily",
                "here",
                " b",
                "ut",
                "you",
                "co|uld",
                "say",
                "there",
                "-",
            ]
        )
    }

    mod glob_quotes {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn basic_single() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("foo 'hi there!' btw hello"),
                vec![Blob::n("foo "), Blob::q("hi there!"), Blob::n(" btw hello")]
            );
        }

        #[test]
        fn basic_double() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("foo \"hi there!\" btw hello"),
                vec![Blob::n("foo "), Blob::q("hi there!"), Blob::n(" btw hello")]
            );
        }

        #[test]
        fn no_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("foo hi there! btw hello"),
                vec![Blob::n("foo hi there! btw hello")]
            );
        }

        #[test]
        fn quote_at_left() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("'foo hi' there! btw hello"),
                vec![Blob::q("foo hi"), Blob::n(" there! btw hello")]
            );
        }

        #[test]
        fn quote_at_right() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw hello 'foo hi'"),
                vec![Blob::n("there! btw hello "), Blob::q("foo hi")]
            );
        }

        #[test]
        fn single_dangling() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw hello 'foo hi"),
                vec![Blob::n("there! btw hello 'foo hi")]
            );
        }

        #[test]
        fn multiple_dangling() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defghijklmnopq\"rstuvwxyz|vvvv|v"),
                vec![
                    Blob::n("abc'defghijklmnopq\"rstuvwxyz"),
                    Blob::q("vvvv"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn one_success_amongst_dangling() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'', '|', '-', '.']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defghi.jklmnopq\"rstu\"vwx-yz|vvvv|v"),
                vec![
                    Blob::n("abc'defghi.jklmnopq"),
                    Blob::q("rstu"),
                    Blob::n("vwx-yz"),
                    Blob::q("vvvv"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn dangling_inside_matched_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw hello 'foo\" hi'"),
                vec![Blob::n("there! btw hello "), Blob::q("foo\" hi")]
            );
        }

        #[test]
        fn dangling_after_matched_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw 'foo hi' \" hello"),
                vec![
                    Blob::n("there! btw "),
                    Blob::q("foo hi"),
                    Blob::n(" \" hello")
                ]
            );
        }

        #[test]
        fn dangling_before_matched_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there!\" btw 'foo hi' hello"),
                vec![
                    Blob::n("there!\" btw "),
                    Blob::q("foo hi"),
                    Blob::n(" hello")
                ]
            );
        }

        #[test]
        fn dangling_at_start() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("'there! btw foo hi hello"),
                vec![Blob::n("'there! btw foo hi hello")]
            );
        }

        #[test]
        fn dangling_at_end() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw foo hi hello'"),
                vec![Blob::n("there! btw foo hi hello'")]
            );
        }

        #[test]
        fn dangling_at_start_with_pair() {
            let tokenizer = DefaultTokenizer::new(vec!['|', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("'there! btw |foo |hi hello"),
                vec![
                    Blob::n("'there! btw "),
                    Blob::q("foo "),
                    Blob::n("hi hello")
                ]
            );
        }

        #[test]
        fn dangling_at_end_with_pair() {
            let tokenizer = DefaultTokenizer::new(vec!['|', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("there! btw |foo |hi hello'"),
                vec![
                    Blob::n("there! btw "),
                    Blob::q("foo "),
                    Blob::n("hi hello'")
                ]
            );
        }

        #[test]
        fn multiple_non_overlapping_pairs() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz"),
                vec![
                    Blob::n("abc"),
                    Blob::q("defg"),
                    Blob::n("hijk"),
                    Blob::q("lmno"),
                    Blob::n("pqr"),
                    Blob::q("stuvwx"),
                    Blob::n("yz")
                ]
            );
        }

        #[test]
        fn many_kinds_of_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'', '|']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec![
                    Blob::n("abc"),
                    Blob::q("defg"),
                    Blob::n("hijk"),
                    Blob::q("lmno"),
                    Blob::n("pqr"),
                    Blob::q("stuvwx"),
                    Blob::n("yz"),
                    Blob::q("vvvv"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn only_one_quote() {
            let tokenizer = DefaultTokenizer::new(vec!['|']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defg'hijk'lmno'pqr'stuvwx'yz|vvvv|v"),
                vec![
                    Blob::n("abc'defg'hijk'lmno'pqr'stuvwx'yz"),
                    Blob::q("vvvv"),
                    Blob::n("v")
                ]
            );
        }

        #[test]
        fn multiple_kinds_of_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("abc'defg'hijklmnopqr\"stuvwx\"yz"),
                vec![
                    Blob::n("abc"),
                    Blob::q("defg"),
                    Blob::n("hijklmnopqr"),
                    Blob::q("stuvwx"),
                    Blob::n("yz")
                ]
            );
        }

        #[test]
        fn empty_string() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(tokenizer.split_into_quote_blobs(""), vec![Blob::n("")]);
        }

        #[test]
        fn only_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("''''''''"),
                // There are 4 pairs of single quotes above, so 4 blobs:
                vec![Blob::q(""), Blob::q(""), Blob::q(""), Blob::q("")]
            );
        }

        #[test]
        fn mixture_of_only_quotes() {
            let tokenizer = DefaultTokenizer::new(vec!['|', '\'']);
            assert_eq!(
                tokenizer.split_into_quote_blobs("''||'|''|'||''|'|"),
                vec![
                    Blob::q(""),
                    Blob::q(""),
                    Blob::q("|"),
                    Blob::q("|"),
                    Blob::q(""),
                    Blob::q(""),
                    Blob::q("'")
                ]
            );
        }
    }

    mod split_by_space {
        use super::*;

        #[test]
        fn empty_string() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            let empty_vec: Vec<&str> = Vec::new();
            assert_eq!(tokenizer.split_by_space(vec![]), empty_vec);
        }

        #[test]
        fn empty_blob() {
            // I don't think this is actually ever possible if we take blobs from
            // split_into_quote_blobs(), but whatever.
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            // We expect us to not include the empty string, since the tokenizer considers it
            // useless.
            let empty_vec: Vec<&str> = Vec::new();
            assert_eq!(tokenizer.split_by_space(vec![Blob::n("")]), empty_vec);
        }

        #[test]
        fn multiple_empty_blobs() {
            // Ditto comments in the empty_blob() test.
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            let empty_vec: Vec<&str> = Vec::new();
            assert_eq!(
                tokenizer.split_by_space(vec![Blob::n(""), Blob::n(""), Blob::n("")]),
                empty_vec
            );
        }

        #[test]
        fn only_normals() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![Blob::n("hi there"), Blob::n("euler is cool")]),
                vec!["hi", "there", "euler", "is", "cool"]
            );
        }

        #[test]
        fn only_quoteds() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![Blob::q("hi there"), Blob::q("euler is cool")]),
                vec!["hi there", "euler is cool"]
            );
        }

        #[test]
        fn quoted_then_normal() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![Blob::q("hi there!"), Blob::n("euler is cool")]),
                vec!["hi there!", "euler", "is", "cool"]
            );
        }

        #[test]
        fn normal_then_quoted() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![Blob::n("euler is cool"), Blob::q("hi there!")]),
                vec!["euler", "is", "cool", "hi there!"]
            );
        }

        #[test]
        fn quoted_surrounded_by_normals() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::n("euler is cool"),
                    Blob::q("hi there!"),
                    Blob::n("euler is cool")
                ]),
                vec!["euler", "is", "cool", "hi there!", "euler", "is", "cool"]
            );
        }

        #[test]
        fn normal_surrounded_by_quoteds() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::q("hi there!"),
                    Blob::n("euler is cool"),
                    Blob::q("hi there!")
                ]),
                vec!["hi there!", "euler", "is", "cool", "hi there!"]
            );
        }

        #[test]
        fn trailing_spaces() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::q("hi there!"),
                    Blob::n("euler is cool "),
                    Blob::q("hi there!")
                ]),
                vec!["hi there!", "euler", "is", "cool", "hi there!"]
            );
        }

        #[test]
        fn with_newline() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::q("hi there!"),
                    // We expect the newline to not be used as a splitting term.
                    Blob::n("euler is\ncool "),
                    Blob::q("hi there!")
                ]),
                vec!["hi there!", "euler", "is\ncool", "hi there!"]
            );
        }

        #[test]
        fn with_tab() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::q("hi there!"),
                    // We expect the tab to not be used as a splitting term.
                    Blob::n("euler is\tcool "),
                    Blob::q("hi there!")
                ]),
                vec!["hi there!", "euler", "is\tcool", "hi there!"]
            );
        }

        #[test]
        fn multiple_spaces() {
            let tokenizer = DefaultTokenizer::new(vec!['"', '\'']);
            assert_eq!(
                tokenizer.split_by_space(vec![
                    Blob::q("hi there!"),
                    // We expect the tab to not be used as a splitting term.
                    Blob::n("euler    is   cool "),
                    Blob::q("hi   there!")
                ]),
                vec!["hi there!", "euler", "is", "cool", "hi   there!"]
            );
        }
    }
}
