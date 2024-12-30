use super::tokens::{Keyword, Operator, Token, Separator, Whitespace};
use std::{iter::Peekable, str::Chars};

#[derive(Debug)]
struct CharLocation {
    row: usize,
    col: usize,
}

impl Default for CharLocation {
    fn default() -> Self {
        return Self { row: 0, col: 0 };
    }
}

#[derive(Debug)]
struct CharStream<'a> {
    // Original Input
    input: &'a str,
    // Iterator over characters
    chars: Peekable<Chars<'a>>,
    // Current location
    location: CharLocation,
}

impl<'a> CharStream<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            location: Default::default(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next().inspect(|character: &char| {
            if *character == '\n' {
                self.location.col += 1;
                self.location.row = 0;
            } else {
                self.location.row += 1;
            }
        })
    }
}

enum TokenizerStates {
    Base,
    String,
    Comment,
    Operator,
    Number { parsing_decimals: bool },
    Invalid { reason: String },
}

struct Tokenizer {
    state: TokenizerStates,
    char_buffer: String,
    tokens: Vec<Token>,
}


impl Tokenizer {
    fn new() -> Self {
        Tokenizer {
            state: TokenizerStates::Base,
            char_buffer: String::from(""),
            tokens: vec![],
        }
    }

    fn next(mut self, character: char) -> Tokenizer {
        match (self.state, character, self.char_buffer.as_str()) {

            /*
                Operator parsing
            */
            // Handle multi-character Operators:
            // If the buffer + current character is a multi-character operator
            //    we simply add the token and move on.
            // Otherwise we need to add the single character operator and reprocess
            //    the current character with an empty buffer.
            (TokenizerStates::Operator, ..) => {
                let multi_char_operator =  Operator::from(format!("{}{}", self.char_buffer, character).as_str());
                if multi_char_operator != Operator::Invalid {
                    self.tokens.push(Token::Separator(Separator::Operator(multi_char_operator)));
                    Tokenizer {
                        state: TokenizerStates::Base,
                        char_buffer: String::from(""),
                        tokens: self.tokens,
                    }
                } else {
                    let single_char_operator =  Operator::from(self.char_buffer.as_str());
                    self.tokens.push(Token::Separator(Separator::Operator(single_char_operator)));
                    let tokenizer = Tokenizer {
                        state: TokenizerStates::Base,
                        char_buffer: String::from(""),
                        tokens: self.tokens,
                    };
                    tokenizer.next(character)
                }

            }

            /*
                String parsing
            */
            // Trigger: double quotes
            (TokenizerStates::Base, '"', ..) => {
                Tokenizer {
                    state: TokenizerStates::String,
                    char_buffer: String::from(""),
                    tokens: self.tokens,
                }
            }
            // Invalid if adding new line to string
            (TokenizerStates::String, '\n', ..) => {
                Tokenizer {
                    state: TokenizerStates::Invalid{ reason: String::from("Unterminated string") },
                    char_buffer: format!("{}{}", self.char_buffer, character),
                    tokens: self.tokens
                }
            }
            // String terminator
            (TokenizerStates::String, '"', ..) => {
                let token = Token::String(String::from(self.char_buffer));
                self.tokens.push(token);
                Tokenizer {
                    state: TokenizerStates::Base,
                    char_buffer: String::from(""),
                    tokens: self.tokens,
                }
            }
            // Add character to string
            (TokenizerStates::String, ..) => {
                Tokenizer {
                    state: TokenizerStates::String,
                    char_buffer: format!("{}{}", self.char_buffer, character),
                    tokens: self.tokens,
                }
            }

            /*
                Comment parsing
            */
            // Trigger: "--"
            (TokenizerStates::Base, '-', "-") => {
                Tokenizer {
                    state: TokenizerStates::Comment,
                    char_buffer: String::from(""),
                    tokens: self.tokens,
                }
            }
            // Comment terminator
            (TokenizerStates::Comment, '\n', ..) => {
                Tokenizer {
                    state: TokenizerStates::Base,
                    char_buffer: String::from(""),
                    tokens: self.tokens,
                }
            }
            // Skip comment characters
            (TokenizerStates::Comment, ..) => {
                Tokenizer {
                    state: TokenizerStates::Comment,
                    char_buffer: String::from(""),
                    tokens: self.tokens,
                }
            }

            /*
                Number parsing
            */
            // Trigger: decimal or period
            (TokenizerStates::Base, '0'..='9' | '.', .. ) => {
                Tokenizer {
                    state: TokenizerStates::Number {
                        parsing_decimals: character == '.'
                    },
                    char_buffer: character.to_string(),
                    tokens: self.tokens
                }
            }
            // Invalid if numbers have more than 1 '.'
            (TokenizerStates::Number{ parsing_decimals: true }, '.', ..) => {
                Tokenizer {
                    state: TokenizerStates::Invalid{ reason: String::from("Invalid numeric, found second '.'") },
                    char_buffer: self.char_buffer,
                    tokens: self.tokens
                }
            }
            // Start processing decimals after first '.'
            (TokenizerStates::Number{ parsing_decimals: false }, '.', ..) => {
                Tokenizer {
                    state: TokenizerStates::Number{ parsing_decimals: true},
                    char_buffer: format!("{}{}", self.char_buffer, character),
                    tokens: self.tokens
                }
            }
            // Add a character to the number
            (TokenizerStates::Number{ parsing_decimals }, '0'..='9', ..) => {
                Tokenizer {
                    state: TokenizerStates::Number{ parsing_decimals },
                    char_buffer: format!("{}{}", self.char_buffer, character),
                    tokens: self.tokens
                }
            }
            // Any other value terminates number
            (TokenizerStates::Number{ .. }, ..) => {
                let token = Token::Number(self.char_buffer.into());
                self.tokens.push(token);
                Tokenizer {
                    state: TokenizerStates::Base,
                    char_buffer: character.to_string(),
                    tokens: self.tokens
                }
            }

            /*
                Base
            */
            _ => {
                let separator = Separator::from(character.to_string().as_str());

                match separator {
                    // Default add the character to the buffer
                    Separator::Invalid {} => {
                        Tokenizer {
                            state: TokenizerStates::Base,
                            char_buffer: format!("{}{}", self.char_buffer, character),
                            tokens: self.tokens
                        }
                    }
                    // Handle Separators
                    _ => {
                        // Add the buffered token if its not empty
                        let str_buffer = self.char_buffer.as_str();
                        if str_buffer != "" {
                            let buffered_token = Token::from(self.char_buffer.as_str());
                            self.tokens.push(buffered_token);
                        }
                        match separator {
                            // Operators are special because they could be multi-character.
                            // Thus store the character in the buffer and switch states
                            Separator::Operator { .. } => {
                                Tokenizer {
                                    state: TokenizerStates::Operator,
                                    char_buffer: character.to_string(),
                                    tokens: self.tokens
                                }
                            }
                            // All other separators are single-character, so start at a blank base case
                            _ => {
                                self.tokens.push(Token::Separator(separator));
                                Tokenizer {
                                    state: TokenizerStates::Base,
                                    char_buffer: String::from(""),
                                    tokens: self.tokens
                                }

                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::{Keyword, Token, Whitespace, Tokenizer};

    // #[test]
    // fn tokenize_select() {
    //     let sql = "SELECT * FROM table;";

    //     assert_eq!(
    //         tokenize(sql, Token),
    //         Ok(vec![
    //             Token::Keyword(Keyword::Select),
    //             Token::Whitespace(Whitespace::Space),
    //             Token::Identifier("*".into()),
    //             Token::Whitespace(Whitespace::Space),
    //             Token::Keyword(Keyword::From),
    //             Token::Whitespace(Whitespace::Space),
    //             Token::Identifier("table".into()),
    //             Token::Whitespace(Whitespace::Space),
    //             Token::Semicolon,
    //         ])
    //     )
    // }
}
