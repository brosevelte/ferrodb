#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Keyword(Keyword),
    Identifier(String),
    Separator(Separator),
    String(String),
    Number(String),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Separator {
    Comma,
    Invalid,
    Operator(Operator),
    Semicolon,
    Whitespace(Whitespace),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Whitespace {
    Invalid,
    Newline,
    Space,
    Tab,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Keyword {
    And,
    As,
    Begin,
    Between,
    BigInt,
    Bool,
    By,
    Commit,
    Create,
    Database,
    Delete,
    Distinct,
    Drop,
    False,
    From,
    In,
    Index,
    Insert,
    Int,
    Invalid,
    Key,
    Like,
    Limit,
    Not,
    Null,
    Or,
    Order,
    Primary,
    Rollback,
    Select,
    Set,
    Table,
    Transaction,
    True,
    Unique,
    Unsigned,
    Update,
    Values,
    Varchar,
    Where,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Operator {
    Add,
    Divide,
    Eq,
    Gt,
    GtEq,
    Invalid,
    Lt,
    LtEq,
    Modulo,
    Multiply,
    NotEq,
    ParenClose,
    ParenOpen,
    Subtract,
}

impl From<&str> for Token {
    fn from(val: &str) -> Token {
        if Separator::from(val) != Separator::Invalid {
            Token::Separator(Separator::from(val))
        } else if Operator::from(val) != Operator::Invalid {
            Token::Separator(Separator::Operator(Operator::from(val)))
        } else if Keyword::from(val) != Keyword::Invalid {
            Token::Keyword(Keyword::from(val))
        } else {
            Token::Identifier(val.to_string())
        }
    }
}

impl From<&str> for Separator {
    fn from(val: &str) -> Separator {
        match val.to_uppercase().as_str() {
            ";" => Separator::Semicolon,
            "," => Separator::Comma,
            _ => {
                if Whitespace::from(val) != Whitespace::Invalid {
                   Separator::Whitespace(Whitespace::from(val))
                } else if Operator::from(val) != Operator::Invalid {
                    Separator::Operator(Operator::from(val))
                } else {
                    Separator::Invalid
                }
            }
        }
    }
}

impl From<&str> for Whitespace {
    fn from(val: &str) -> Whitespace {
        match val.to_uppercase().as_str() {
            " " => Whitespace::Space,
            "\t" => Whitespace::Tab,
            "\n" => Whitespace::Newline,
            _ => Whitespace::Invalid
        }
    }
}


impl From<&str> for Keyword {
    fn from(val: &str) -> Keyword {
        match val.to_uppercase().as_str() {
            "AND" => Keyword::And,
            "AS" => Keyword::As,
            "BEGIN" => Keyword::Begin,
            "BETWEEN" => Keyword::Between,
            "BIGINT" => Keyword::BigInt,
            "BOOL" => Keyword::Bool,
            "BY" => Keyword::By,
            "COMMIT" => Keyword::Commit,
            "CREATE" => Keyword::Create,
            "DATABASE" => Keyword::Database,
            "DELETE" => Keyword::Delete,
            "DISTINCT" => Keyword::Distinct,
            "DROP" => Keyword::Drop,
            "FALSE" => Keyword::False,
            "FROM" => Keyword::From,
            "IN" => Keyword::In,
            "INDEX" => Keyword::Index,
            "INSERT" => Keyword::Insert,
            "INT" => Keyword::Int,
            "KEY" => Keyword::Key,
            "LIKE" => Keyword::Like,
            "LIMIT" => Keyword::Limit,
            "NOT" => Keyword::Not,
            "NULL" => Keyword::Null,
            "OR" => Keyword::Or,
            "ORDER" => Keyword::Order,
            "PRIMARY" => Keyword::Primary,
            "ROLLBACK" => Keyword::Rollback,
            "SELECT" => Keyword::Select,
            "SET" => Keyword::Set,
            "TABLE" => Keyword::Table,
            "TRANSACTION" => Keyword::Transaction,
            "TRUE" => Keyword::True,
            "UNIQUE" => Keyword::Unique,
            "UNSIGNED" => Keyword::Unsigned,
            "UPDATE" => Keyword::Update,
            "VALUES" => Keyword::Values,
            "VARCHAR" => Keyword::Varchar,
            "WHERE" => Keyword::Where,
            _ => Keyword::Invalid
        }
    }
}

impl From<&str> for Operator {
    fn from(val: &str) -> Operator {
        match val.to_uppercase().as_str() {
            "+" => Operator::Add,
            "/" => Operator::Divide,
            "=" => Operator::Eq,
            ">" => Operator::Gt,
            ">=" => Operator::GtEq,
            "<" => Operator::Lt,
            "<=" => Operator::LtEq,
            "%" => Operator::Modulo,
            "*" => Operator::Multiply,
            "!=" => Operator::NotEq,
            ")" => Operator::ParenClose,
            "(" => Operator::ParenOpen,
            "-" => Operator::Subtract,
            _ => Operator::Invalid
        }
    }
}
