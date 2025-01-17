use super::tokens::{Operator, Separator, Token, Whitespace};
use std::collections::VecDeque;

use std::fmt::{self, Display};
use std::str::Chars;

// ///////////////// //
// Character Parsing //
// ///////////////// //
#[derive(Debug, PartialEq, Clone, Copy)]
struct CharacterLocation {
    row: usize,
    col: usize,
}

impl Default for CharacterLocation {
    fn default() -> Self {
        return Self { row: 0, col: 0 };
    }
}

impl Display for CharacterLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CharacterLocation({self.row}:{self.col})")
    }
}

#[derive(Debug)]
struct CharacterIter<'a> {
    // Original Input
    input: &'a str,
    // Iterator over characters
    chars: Chars<'a>,
    // Current location
    location: CharacterLocation,
}

impl<'a> CharacterIter<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            location: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct CharacterItem {
    character: char,
    location: CharacterLocation,
}

impl<'a> Iterator for CharacterIter<'a> {
    type Item = CharacterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars
            .next()
            .inspect(|character: &char| {
                if *character == '\n' {
                    self.location.col += 1;
                    self.location.row = 0;
                } else {
                    self.location.row += 1;
                }
            })
            .map(|character: char| CharacterItem {
                character,
                location: self.location,
            })
    }
}

// ///////////// //
// Token Parsing //
// ///////////// //
#[derive(Debug, PartialEq)]
struct TokenItem {
    token: Token,
    start: CharacterLocation,
    end: CharacterLocation,
}

#[derive(Debug)]
struct BaseState;
#[derive(Debug)]
struct StringState;
#[derive(Debug)]
struct CommentState;
#[derive(Debug)]
struct OperatorState;
#[derive(Debug)]
struct NumberState { parsing_decimals: bool }
#[derive(Debug)]
struct InvalidState;

#[derive(Debug)]
enum TokenizerStateMachine {
    Base(Tokenizer<BaseState>),
    String(Tokenizer<StringState>),
    Comment(Tokenizer<CommentState>),
    Operator(Tokenizer<OperatorState>),
    Number(Tokenizer<NumberState>),
    Invalid(Tokenizer<InvalidState>),
}

impl TokenizerStateMachine {
    fn new() -> Self {
        Self::Base(Tokenizer::new())
    }
}

impl TokenizerStateMachine {
    fn process_character(self, character_item: CharacterItem) -> Self {
        match self {
            TokenizerStateMachine::Base(state) => state.process_character(character_item),
            TokenizerStateMachine::String(state) => state.process_character(character_item),
            TokenizerStateMachine::Comment(state) => state.process_character(character_item),
            TokenizerStateMachine::Operator(state) => state.process_character(character_item),
            TokenizerStateMachine::Number(state) => state.process_character(character_item),
            TokenizerStateMachine::Invalid(..) => panic!("INVALID"),
        }
    }
}
#[derive(Debug)]
struct Tokenizer<S> {
    state: S,
    char_buffer: String,
    token_start: CharacterLocation,
    tokens: VecDeque<TokenItem>,
}

impl Tokenizer<BaseState> {
    fn new() -> Self {
        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: Default::default(),
            tokens: vec![].into(),
        }
    }
}

impl Tokenizer<BaseState> {
    fn tokenize(string: String, start: CharacterLocation, end: CharacterLocation) -> Option<TokenItem> {
        if string == "" {
            return None
        }

        Some(TokenItem {
            token: Token::from(string.as_str()),
            start,
            end,
        })
    }

    fn to_string_state(mut self, character_item: CharacterItem) -> Tokenizer<StringState>{
        Tokenizer::<BaseState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer {
            state: StringState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }
    fn to_comment_state(mut self, character_item: CharacterItem) -> Tokenizer<CommentState>{
        Tokenizer::<BaseState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer{
            state: CommentState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }

    fn to_number_state(self, character_item: CharacterItem) -> Tokenizer<NumberState>{
        // character_item will always be "" in this instance

        Tokenizer{
            state: NumberState { parsing_decimals: character_item.character == '.'},
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }

    fn to_base_state(self, character_item: CharacterItem) -> Tokenizer<BaseState>{
        Tokenizer{
            state: BaseState,
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens
        }
    }


    fn to_operator_state(mut self, character_item: CharacterItem) -> Tokenizer<OperatorState>{
        Tokenizer::<BaseState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer{
            state: OperatorState,
            char_buffer: character_item.character.to_string(),
            token_start: self.token_start,
            tokens: self.tokens
        }
    }

    fn process_character(mut self, character_item: CharacterItem) -> TokenizerStateMachine {
        match (
            character_item.character,
            self.char_buffer.as_str()
        ) {
            /*
                String Trigger: double quotes
            */
            ('"', ..) => {
                TokenizerStateMachine::String(self.to_string_state(character_item))
            },
            /*
                Comment Trigger: double dash
            */
            ('-', "-") => {
                TokenizerStateMachine::Comment(self.to_comment_state(character_item))
            },
            /*
                Number Trigger: numeric or period with no character buffer
            */
            ('0'..='9' | '.', "") => {
                TokenizerStateMachine::Number(self.to_number_state(character_item))
            },
            _ => {
                let separator = Separator::from(character_item.character.to_string().as_str());

                match separator {
                    // Default add the character to the buffer
                    Separator::Invalid {} => {
                        TokenizerStateMachine::Base(self.to_base_state(character_item))
                    },
                    // Handle Separators
                    _ => {
                        // Add the buffered token if its not empty
                        Tokenizer::<BaseState>::tokenize(
                            String::from(self.char_buffer.as_str()),
                            self.token_start,
                            character_item.location
                        ).map(|token_item| {
                            self.tokens.push_back(token_item);
                        });

                        match separator {
                            // Operators are special because they could be multi-character.
                            // Thus store the character in the buffer and switch states
                            Separator::Operator { .. } => {
                                TokenizerStateMachine::Operator(self.to_operator_state(character_item))
                            },
                            // All other separators are single-character, so start at a blank base case
                            _ => {
                                self.char_buffer = character_item.character.to_string();
                                TokenizerStateMachine::Base(self.to_base_state(character_item))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Tokenizer<StringState> {
    fn tokenize(string: String, start: CharacterLocation, end: CharacterLocation) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::String(string.into()),
            start,
            end,
        })
    }

    fn to_invalid_state(self, _character_item: CharacterItem) -> Tokenizer<InvalidState>{
        Tokenizer {
            state: InvalidState,
            char_buffer: self.char_buffer,
            token_start: self.token_start,
            tokens: self.tokens
        }
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState>{
        Tokenizer::<StringState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }

    fn to_string_state(self, character_item: CharacterItem) -> Tokenizer<StringState>{
        Tokenizer {
            state: StringState,
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens
        }
    }
    fn process_character(self, character_item: CharacterItem) -> TokenizerStateMachine {
        match (
            character_item.character,
            self.char_buffer.as_str()
        ) {
            ('\n', ..) => {
                TokenizerStateMachine::Invalid(self.to_invalid_state(character_item))
            },
            // String terminator
            ('"', ..) => {
                TokenizerStateMachine::Base(self.to_base_state(character_item))
            }
            // Add character to string
            _ => {
                TokenizerStateMachine::String(self.to_string_state(character_item))
            },
        }
    }
}

impl Tokenizer<CommentState> {
    fn to_comment_state(self, _character_item: CharacterItem) -> Tokenizer<CommentState>{
        Tokenizer {
            state: CommentState,
            char_buffer: self.char_buffer,
            token_start: self.token_start,
            tokens: self.tokens
        }
    }

    fn to_base_state(self, character_item: CharacterItem) -> Tokenizer<BaseState>{
        Tokenizer {
            state: BaseState,
            char_buffer: self.char_buffer,
            token_start: character_item.location,
            tokens: self.tokens
        }
    }
    fn process_character(self, character_item: CharacterItem) -> TokenizerStateMachine {
        match (
            character_item.character,
            self.char_buffer.as_str()
        ) {
            // Comment terminator
            ('\n', ..) => {
                TokenizerStateMachine::Base(self.to_base_state(character_item))
            },
            // Skip comment characters
            _ => {
                TokenizerStateMachine::Comment(self.to_comment_state(character_item))
            },
        }
    }
}

impl Tokenizer<OperatorState> {
    fn tokenize(string: String, start: CharacterLocation, end: CharacterLocation) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::Separator(Separator::Operator(Operator::from(string.as_str()))),
            start,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState>{
        Tokenizer::<StringState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }
    fn process_character(mut self, character_item: CharacterItem) -> TokenizerStateMachine {
        // If the buffer + current character is a multi-character operator
        //    we simply add the token and move on.
        // Otherwise we need to add the single character operator and reprocess
        //    the current character with an empty buffer.
        let multi_char_operator = Operator::from(
            format!("{}{}", self.char_buffer, character_item.character).as_str(),
        );
        match multi_char_operator {
            Operator::Invalid => {
                TokenizerStateMachine::Base(self.to_base_state(character_item)).process_character(character_item)

            }
            _ => {
                self.char_buffer = format!("{}{}", self.char_buffer, character_item.character);
                TokenizerStateMachine::Base(self.to_base_state(character_item))
            }
        }
    }
}

impl Tokenizer<NumberState> {
    fn tokenize(string: String, start: CharacterLocation, end: CharacterLocation) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::Number(string.into()),
            start,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState>{
        Tokenizer::<NumberState>::tokenize(
            self.char_buffer,
            self.token_start,
            character_item.location
        ).map(|token_item| {
            self.tokens.push_back(token_item);
        });

        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens
        }
    }

    fn to_number_state(self, character_item: CharacterItem, parsing_decimals: bool) -> Tokenizer<NumberState>{
        Tokenizer {
            state: NumberState{ parsing_decimals },
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens
        }
    }
    fn to_invalid_state(self, _character_item: CharacterItem) -> Tokenizer<InvalidState>{
        Tokenizer {
            state: InvalidState,
            char_buffer: self.char_buffer,
            token_start: self.token_start,
            tokens: self.tokens
        }
    }
    fn process_character(self, character_item: CharacterItem) -> TokenizerStateMachine {
        match (
            character_item.character,
            self.char_buffer.as_str(),
            self.state.parsing_decimals
        ) {

            ('.', .., true) => {
                TokenizerStateMachine::Invalid(self.to_invalid_state(character_item))
            },
            // Start processing decimals after first '.'
            ('.', .., false) => {
                TokenizerStateMachine::Number(self.to_number_state(character_item, true))
            },
            // Add a character to the number
            ('0'..='9', ..) => {
                let current_state = self.state.parsing_decimals;
                TokenizerStateMachine::Number(self.to_number_state(character_item, current_state))
            },
            // Any other value terminates number
            _ => {
                TokenizerStateMachine::Base(self.to_base_state(character_item))
            }
        }
    }
}
























// fn iterate(sql: &str) -> Option<TokenItem> {
//     let mut tokenizer = TokenizerStateMachine::new();
//     let mut char_stream = CharacterIter::new(sql);

//     loop {
//         if tokenizer.tokens.len() > 0 {
//             return tokenizer.tokens.pop_front();
//         }

//         let character_option = char_stream.next();

//         if character_option == Option::None {
//             return Option::None;
//         } else {
//             tokenizer = tokenizer.process_character(character_option.unwrap());
//         }
//     }
// }



// #[cfg(test)]
// mod test {

//     use super::{iterate, Keyword, Separator, Token, Whitespace};

//     #[test]
//     fn tokenize_select() {
//         let sql = "SELECT a_column FROM a_table;";

//         assert_eq!(
//             iterate(sql),
//             vec![
//                 Token::Keyword(Keyword::Select),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Identifier("a_column".into()),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Keyword(Keyword::From),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Identifier("a_table".into()),
//                 Token::Separator(Separator::Semicolon),
//             ]
//         )
//     }

//     #[test]
//     fn tokenize_select_all() {
//         let sql = "SELECT * FROM a_table;";

//         assert_eq!(
//             tokenize(sql),
//             vec![
//                 Token::Keyword(Keyword::Select),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Identifier("a_column".into()),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Keyword(Keyword::From),
//                 Token::Separator(Separator::Whitespace(Whitespace::Space)),
//                 Token::Identifier("a_table".into()),
//                 Token::Separator(Separator::Semicolon),
//             ]
//         )
//     }
// }
