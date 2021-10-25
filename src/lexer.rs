use std::str::FromStr;

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum LexToken<'a> {
    Ident(&'a str),
    Symbol(&'a str),
    Number(i64),

    KeyMod,
    KeyOutput,
    KeyLet,
    KeyIf,
    KeyMatch,

    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpMod,
    OpPower,
    
    OpBitOr,
    OpBitAnd,
    OpBitXor,
    OpShiftLeft,
    OpShiftRight,

    OpCmpEq,
    OpCmpNeq,
    OpCmpGt,
    OpCmpLt,
    OpCmpGeq,
    OpCmpLeq,

    OpAssign,
    OpComma,
    OpSemicolon,
    OpMatchArrow,
    OpThinArrow,

    OpParenOpen,
    OpParenClose,
    OpBraceOpen,
    OpBraceClose,
}

impl<'a> LexToken<'a> {
    fn ident_or_keyword(ident: &'a str) -> Self {
        match ident {
            "mod" => Self::KeyMod,
            "output" => Self::KeyOutput,
            "let" => Self::KeyLet,
            "if" => Self::KeyIf,
            "match" => Self::KeyMatch,
            _ => Self::Ident(ident)
        }
    }
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>
}

impl<'a> Lexer<'a> {
    pub fn new(string: &str) -> Lexer {
        Lexer{
            chars: string.chars()
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LexToken<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let parse_str = self.chars.as_str();
            
            return if let Some(c) = self.chars.next() {
                if c.is_ascii_alphabetic() || c == '_' {
                    let token_end = parse_str.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .unwrap_or(parse_str.len());
                    let token_str = &parse_str[0..token_end];
                    let remainder_str = &parse_str[token_end..];
    
                    self.chars = remainder_str.chars();
    
                    Some(LexToken::ident_or_keyword(token_str))
                } else if c.is_ascii_digit() {
                    // TODO _ seperators?
                    // Don't bother handling negatives. All constants are unsigned.

                    let next_char = parse_str.chars().nth(1);

                    if c == '0' && next_char.unwrap_or('0').is_ascii_alphabetic() {
                        let radix = if next_char == Some('x') {
                            16
                        } else if next_char == Some('b') {
                            2
                        } else {
                            panic!("Bad radix indicator '{:?}'.",next_char);
                        };

                        let parse_str = &parse_str[2..];
                        
                        let token_end = parse_str.find(|c: char| !c.is_ascii_alphanumeric())
                            .unwrap_or(parse_str.len());
                        let token_str = &parse_str[0..token_end];
                        let remainder_str = &parse_str[token_end..];
    
                        self.chars = remainder_str.chars();

                        let num = i64::from_str_radix(token_str, radix).expect("failed to parse int");
    
                        Some(LexToken::Number(num))
                    } else {
                        let token_end = parse_str.find(|c: char| !c.is_ascii_digit())
                            .unwrap_or(parse_str.len());
                        let token_str = &parse_str[0..token_end];
                        let remainder_str = &parse_str[token_end..];
    
                        self.chars = remainder_str.chars();
    
                        let num = i64::from_str(token_str).expect("failed to parse int");
    
                        Some(LexToken::Number(num))
                    }
                } else if c.is_ascii_whitespace() {
                    // TODO line handling?
                    // Probably use another iterator wrapper if we want that?
                    // skip
                    continue;
                } else if c == '$' {
                    let parse_str = self.chars.as_str();

                    let token_end = parse_str.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .unwrap_or(parse_str.len());
                    let token_str = &parse_str[0..token_end];
                    let remainder_str = &parse_str[token_end..];

                    self.chars = remainder_str.chars();

                    Some(LexToken::Symbol(token_str))
                } else {
                    match c {
                        '+' => Some(LexToken::OpAdd),
                        '-' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('>') {
                                self.chars.next();
                                Some(LexToken::OpThinArrow)
                            } else {
                                Some(LexToken::OpSub)
                            }
                        },
                        '%' => Some(LexToken::OpMod),
                        '/' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('/') {
                                // Single-line comment.
                                while self.chars.next() != Some('\n') { }
                                continue;
                            } else if next_char == Some('*') {
                                self.chars.next();
                                while !self.chars.as_str().starts_with("*/") {
                                    self.chars.next();
                                }
                                self.chars.next();
                                self.chars.next();
                                continue;
                            } else {
                                Some(LexToken::OpDiv)
                            }
                        },
                        '*' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('*') {
                                self.chars.next();
                                Some(LexToken::OpPower)
                            } else {
                                Some(LexToken::OpMul)
                            }
                        },

                        '|' => Some(LexToken::OpBitOr),
                        '&' => Some(LexToken::OpBitAnd),
                        '^' => Some(LexToken::OpBitXor),

                        '=' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('=') {
                                self.chars.next();
                                Some(LexToken::OpCmpEq)
                            } else if next_char == Some('>') {
                                self.chars.next();
                                Some(LexToken::OpMatchArrow)
                            } else {
                                Some(LexToken::OpAssign)
                            }
                        },

                        '!' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('=') {
                                self.chars.next();
                                Some(LexToken::OpCmpNeq)
                            } else {
                                panic!("Unexpected '!' {:?}, logical not is not supported.",next_char);
                            }
                        },

                        '<' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('=') {
                                self.chars.next();
                                Some(LexToken::OpCmpLeq)
                            } else if next_char == Some('<') {
                                self.chars.next();
                                Some(LexToken::OpShiftLeft)
                            } else {
                                Some(LexToken::OpCmpLt)
                            }
                        },

                        '>' => {
                            let next_char = parse_str.chars().nth(1);
                            if next_char == Some('=') {
                                self.chars.next();
                                Some(LexToken::OpCmpGeq)
                            } else if next_char == Some('>') {
                                self.chars.next();
                                Some(LexToken::OpShiftRight)
                            } else {
                                Some(LexToken::OpCmpGt)
                            }
                        },

                        ',' => Some(LexToken::OpComma),
                        ';' => Some(LexToken::OpSemicolon),

                        '(' => Some(LexToken::OpParenOpen),
                        ')' => Some(LexToken::OpParenClose),
                        '{' => Some(LexToken::OpBraceOpen),
                        '}' => Some(LexToken::OpBraceClose),

                        _ => panic!("unhandled char [{}]",c)
                    }
                }
            } else {
                None
            }
        }
    }
}
