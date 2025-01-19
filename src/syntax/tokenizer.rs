use super::tokens::{Operator, Separator, Token, Whitespace};
use std::collections::VecDeque;

use std::fmt::{self, Display};
use std::iter::Peekable;
use std::str::Chars;
// ///////////////// //
// Character Parsing //
// ///////////////// //
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct CharacterLocation {
    pub(crate) row: usize,
    pub(crate) col: usize,
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
    chars: Peekable<Chars<'a>>,
    // Current location
    location: CharacterLocation,
}

impl<'a> CharacterIter<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            location: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct CharacterItem {
    character: char,
    next_character: Option<char>,
    location: CharacterLocation,
}

impl<'a> Iterator for CharacterIter<'a> {
    type Item = CharacterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(character) = self.chars.next() {
            let current_location = self.location;
            // Update location for next character
            if character == '\n' {
                self.location.row += 1;
                self.location.col = 0;
            } else {
                self.location.col += 1;
            }
            Some(CharacterItem {
                character,
                next_character: self.chars.peek().copied(),
                location: current_location,
            })
        } else if self.input.is_empty() {
            None
        } else {
            let eof = CharacterItem {
                character: '\0',
                next_character: None,
                location: self.location,
            };
            self.input = "";
            Some(eof)
        }
    }
}

// ///////////// //
// Token Parsing //
// ///////////// //
#[derive(Debug, PartialEq)]
pub(crate) struct TokenItem {
    pub(crate) token: Token,
    pub(crate) start: CharacterLocation,
    pub(crate) end: CharacterLocation,
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
struct NumberState {
    parsing_decimals: bool,
}
#[derive(Debug)]
struct InvalidState;

#[derive(Debug)]
pub(crate) enum TokenizerError {
    UnterminatedString(CharacterLocation),
    InvalidNumber(CharacterLocation),
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

impl<S> Tokenizer<S> {
    fn push_token<F>(
        &mut self,
        string: String,
        start: CharacterLocation,
        end: CharacterLocation,
        tokenize_fn: F,
    ) where
        F: FnOnce(String, CharacterLocation, CharacterLocation) -> Option<TokenItem>,
    {
        tokenize_fn(string, start, end).map(|token_item| {
            self.tokens.push_back(token_item);
        });
    }
}

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
    fn process_character(&mut self, character_item: CharacterItem) -> Result<(), TokenizerError> {
        *self = match std::mem::replace(
            self,
            TokenizerStateMachine::Invalid(Tokenizer {
                state: InvalidState,
                char_buffer: String::from(""),
                token_start: Default::default(),
                tokens: vec![].into(),
            }),
        ) {
            TokenizerStateMachine::Base(state) => state.process_character(character_item)?,
            TokenizerStateMachine::String(state) => state.process_character(character_item)?,
            TokenizerStateMachine::Comment(state) => state.process_character(character_item)?,
            TokenizerStateMachine::Operator(state) => state.process_character(character_item)?,
            TokenizerStateMachine::Number(state) => state.process_character(character_item)?,
            TokenizerStateMachine::Invalid(_) => {
                return Err(TokenizerError::InvalidNumber(character_item.location))
            }
        };
        Ok(())
    }

    fn collect_tokens(&mut self) -> VecDeque<TokenItem> {
        match self {
            TokenizerStateMachine::Base(state) => std::mem::take(&mut state.tokens),
            TokenizerStateMachine::String(state) => std::mem::take(&mut state.tokens),
            TokenizerStateMachine::Comment(state) => std::mem::take(&mut state.tokens),
            TokenizerStateMachine::Operator(state) => std::mem::take(&mut state.tokens),
            TokenizerStateMachine::Number(state) => std::mem::take(&mut state.tokens),
            TokenizerStateMachine::Invalid(state) => std::mem::take(&mut state.tokens),
        }
    }
}

impl Tokenizer<BaseState> {
    fn tokenize(
        string: String,
        start: CharacterLocation,
        end: CharacterLocation,
    ) -> Option<TokenItem> {
        if string.is_empty() || string == "\0" {
            return None;
        }

        Some(TokenItem {
            token: Token::from(string.as_str()),
            start,
            end,
        })
    }

    fn to_string_state(mut self, character_item: CharacterItem) -> Tokenizer<StringState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<BaseState>::tokenize,
        );

        Tokenizer {
            state: StringState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }
    fn to_comment_state(mut self, character_item: CharacterItem) -> Tokenizer<CommentState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<BaseState>::tokenize,
        );

        Tokenizer {
            state: CommentState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }

    fn to_number_state(self, character_item: CharacterItem) -> Tokenizer<NumberState> {
        // character_item will always be "" in this instance

        Tokenizer {
            state: NumberState {
                parsing_decimals: character_item.character == '.',
            },
            char_buffer: String::from(character_item.character),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }

    fn to_base_state(self, character_item: CharacterItem) -> Tokenizer<BaseState> {
        Tokenizer {
            state: BaseState,
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens,
        }
    }

    fn to_operator_state(mut self, character_item: CharacterItem) -> Tokenizer<OperatorState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<BaseState>::tokenize,
        );

        Tokenizer {
            state: OperatorState,
            char_buffer: character_item.character.to_string(),
            token_start: self.token_start,
            tokens: self.tokens,
        }
    }

    fn process_character(
        mut self,
        character_item: CharacterItem,
    ) -> Result<TokenizerStateMachine, TokenizerError> {
        match (
            character_item.character,
            character_item.next_character,
            self.char_buffer.as_str(),
        ) {
            ('\0', _, ..) => {
                self.push_token(
                    self.char_buffer.clone(),
                    self.token_start,
                    character_item.location,
                    Tokenizer::<BaseState>::tokenize,
                );
                Ok(TokenizerStateMachine::Base(self))
            }
            ('"', ..) => Ok(TokenizerStateMachine::String(
                self.to_string_state(character_item),
            )),
            ('-', Some('-'), _) => Ok(TokenizerStateMachine::Comment(
                self.to_comment_state(character_item),
            )),
            ('0'..='9' | '.', _, "") => Ok(TokenizerStateMachine::Number(
                self.to_number_state(character_item),
            )),
            _ => {
                let separator = Separator::from(character_item.character.to_string().as_str());
                match separator {
                    Separator::Invalid {} => Ok(TokenizerStateMachine::Base(
                        self.to_base_state(character_item),
                    )),
                    _ => {
                        self.push_token(
                            self.char_buffer.clone(),
                            self.token_start,
                            character_item.location,
                            Tokenizer::<BaseState>::tokenize,
                        );

                        match separator {
                            Separator::Operator { .. } => Ok(TokenizerStateMachine::Operator(
                                self.to_operator_state(character_item),
                            )),
                            _ => {
                                self.char_buffer = String::new();
                                self.push_token(
                                    character_item.character.to_string(),
                                    self.token_start,
                                    character_item.location,
                                    Tokenizer::<BaseState>::tokenize,
                                );
                                Ok(TokenizerStateMachine::Base(self))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Tokenizer<StringState> {
    fn tokenize(
        string: String,
        start: CharacterLocation,
        end: CharacterLocation,
    ) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::String(string.into()),
            start,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<StringState>::tokenize,
        );

        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }

    fn to_string_state(self, character_item: CharacterItem) -> Tokenizer<StringState> {
        Tokenizer {
            state: StringState,
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens,
        }
    }
    fn process_character(
        self,
        character_item: CharacterItem,
    ) -> Result<TokenizerStateMachine, TokenizerError> {
        match (character_item.character, self.char_buffer.as_str()) {
            ('\0', _) => Err(TokenizerError::UnterminatedString(self.token_start)),
            ('\n', _) => Err(TokenizerError::UnterminatedString(self.token_start)),
            ('"', _) => Ok(TokenizerStateMachine::Base(
                self.to_base_state(character_item),
            )),
            _ => Ok(TokenizerStateMachine::String(
                self.to_string_state(character_item),
            )),
        }
    }
}

impl Tokenizer<CommentState> {
    fn tokenize(
        _string: String,
        _start: CharacterLocation,
        end: CharacterLocation,
    ) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::Separator(Separator::Whitespace(Whitespace::Newline)),
            start: end,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState> {
        self.push_token(
            String::new(),
            self.token_start,
            character_item.location,
            Tokenizer::<CommentState>::tokenize,
        );

        Tokenizer {
            state: BaseState,
            char_buffer: String::new(),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }

    fn to_comment_state(self, _character_item: CharacterItem) -> Tokenizer<CommentState> {
        Tokenizer {
            state: CommentState,
            char_buffer: self.char_buffer,
            token_start: self.token_start,
            tokens: self.tokens,
        }
    }

    fn process_character(
        self,
        character_item: CharacterItem,
    ) -> Result<TokenizerStateMachine, TokenizerError> {
        match (character_item.character, self.char_buffer.as_str()) {
            // Comment terminator
            ('\n', ..) => Ok(TokenizerStateMachine::Base(
                self.to_base_state(character_item),
            )),
            // Skip comment characters
            _ => Ok(TokenizerStateMachine::Comment(
                self.to_comment_state(character_item),
            )),
        }
    }
}

impl Tokenizer<OperatorState> {
    fn tokenize(
        string: String,
        start: CharacterLocation,
        end: CharacterLocation,
    ) -> Option<TokenItem> {
        Some(TokenItem {
            token: Token::Separator(Separator::Operator(Operator::from(string.as_str()))),
            start,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<OperatorState>::tokenize,
        );

        Tokenizer {
            state: BaseState,
            char_buffer: String::from(""),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }
    fn process_character(
        mut self,
        character_item: CharacterItem,
    ) -> Result<TokenizerStateMachine, TokenizerError> {
        let multi_char_operator = Operator::from(
            format!("{}{}", self.char_buffer.clone(), character_item.character).as_str(),
        );
        match multi_char_operator {
            Operator::Invalid => {
                let base_state = self.to_base_state(character_item);
                base_state.process_character(character_item)
            }
            _ => {
                self.char_buffer = format!("{}{}", self.char_buffer, character_item.character);
                Ok(TokenizerStateMachine::Base(
                    self.to_base_state(character_item),
                ))
            }
        }
    }
}

impl Tokenizer<NumberState> {
    fn tokenize(
        string: String,
        start: CharacterLocation,
        end: CharacterLocation,
    ) -> Option<TokenItem> {
        if string == "" || string == "\0" {
            return None;
        }

        Some(TokenItem {
            token: Token::Number(string.into()),
            start,
            end,
        })
    }

    fn to_base_state(mut self, character_item: CharacterItem) -> Tokenizer<BaseState> {
        self.push_token(
            self.char_buffer.clone(),
            self.token_start,
            character_item.location,
            Tokenizer::<NumberState>::tokenize,
        );

        self.push_token(
            character_item.character.to_string(),
            character_item.location,
            character_item.location,
            Tokenizer::<BaseState>::tokenize,
        );

        Tokenizer {
            state: BaseState,
            char_buffer: String::new(),
            token_start: character_item.location,
            tokens: self.tokens,
        }
    }

    fn to_number_state(
        self,
        character_item: CharacterItem,
        parsing_decimals: bool,
    ) -> Tokenizer<NumberState> {
        Tokenizer {
            state: NumberState { parsing_decimals },
            char_buffer: format!("{}{}", self.char_buffer, character_item.character),
            token_start: self.token_start,
            tokens: self.tokens,
        }
    }
    fn process_character(
        self,
        character_item: CharacterItem,
    ) -> Result<TokenizerStateMachine, TokenizerError> {
        match (
            character_item.character,
            self.char_buffer.as_str(),
            self.state.parsing_decimals,
        ) {
            ('.', _, true) => Err(TokenizerError::InvalidNumber(self.token_start)),
            ('.', _, false) => Ok(TokenizerStateMachine::Number(
                self.to_number_state(character_item, true),
            )),
            ('0'..='9', _, _) => {
                let current_state = self.state.parsing_decimals;
                Ok(TokenizerStateMachine::Number(
                    self.to_number_state(character_item, current_state),
                ))
            }
            _ => Ok(TokenizerStateMachine::Base(
                self.to_base_state(character_item),
            )),
        }
    }
}

// New struct to hold the tokenizer state
pub(crate) struct TokenIterator<'a> {
    char_iter: CharacterIter<'a>,
    state_machine: TokenizerStateMachine,
    buffered_token: Option<TokenItem>,
}

impl<'a> TokenIterator<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            char_iter: CharacterIter::new(input),
            state_machine: TokenizerStateMachine::new(),
            buffered_token: None,
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<TokenItem, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token_item) = self.buffered_token.take() {
            return Some(Ok(token_item));
        }

        while let Some(character) = self.char_iter.next() {
            match self.state_machine.process_character(character) {
                Ok(()) => {
                    let mut tokens = self.state_machine.collect_tokens();
                    if let Some(token) = tokens.pop_front() {
                        if let Some(second_token) = tokens.pop_front() {
                            self.buffered_token = Some(second_token);
                        }
                        return Some(Ok(token));
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

// Make the tokenize function return type explicit
pub(crate) fn tokenize(sql: &str) -> TokenIterator {
    TokenIterator::new(sql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::tokens::{Keyword, Operator, Separator, Token, Whitespace};

    fn collect_tokens(sql: &str) -> Result<Vec<Token>, TokenizerError> {
        tokenize(sql)
            .map(|result| result.map(|token_item| token_item.token))
            .collect()
    }

    #[test]
    fn test_basic_select() {
        let tokens = collect_tokens("SELECT * FROM a_table").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Select),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Separator(Separator::Operator(Operator::Multiply)),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Keyword(Keyword::From),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Identifier("a_table".to_string()),
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let tokens = collect_tokens(r#"SELECT "hello world""#).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Select),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::String("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = collect_tokens("SELECT 42, 3.14").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Select),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Number("42".to_string()),
                Token::Separator(Separator::Comma),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Number("3.14".to_string()),
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = collect_tokens("1 + 2 >= 3").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number("1".to_string()),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Separator(Separator::Operator(Operator::Add)),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Number("2".to_string()),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Separator(Separator::Operator(Operator::GtEq)),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Number("3".to_string()),
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = collect_tokens("SELECT -- this is a comment\n42").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Select),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Separator(Separator::Whitespace(Whitespace::Newline)),
                Token::Number("42".to_string()),
            ]
        );
    }

    #[test]
    fn test_unterminated_string() {
        let result = collect_tokens(r#"SELECT "unterminated"#);
        assert!(matches!(result, Err(TokenizerError::UnterminatedString(_))));
    }

    #[test]
    fn test_invalid_number() {
        let result = collect_tokens("SELECT 3.14.15");
        assert!(matches!(result, Err(TokenizerError::InvalidNumber(_))));
    }

    #[test]
    fn test_final_token() {
        let tokens = collect_tokens("SELECT abc").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Select),
                Token::Separator(Separator::Whitespace(Whitespace::Space)),
                Token::Identifier("abc".to_string()),
            ]
        );
    }
}
