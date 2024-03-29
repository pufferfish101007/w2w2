use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    LParen,
    RParen,
    Identifier(String),
    Annotation(String),
    Instruction(String),
    StringLiteral {
        string: String,
        escape_next_char: bool,
        last_char_escaped: bool,
    },
    NumberLiteral {
        string: String,
        hex: bool,
        dec_point: bool,
        exponent: bool,
        last_char_is_exponent: bool,
        signed: bool,
    },
    Comment {
        string: String,
        multiline: bool,
        nested_level: i32,
    },
    Reserved(String),
    Space(String), // this may not be an actual space - it coukd be a new line, or just exist to signify the end of a literal
}

use Token::*;

#[derive(Clone)]
pub struct TokenList(Vec<Token>, String);

impl TokenList {
    pub fn list(self) -> Vec<Token> {
        self.0
    }

    pub fn src(self) -> String {
        self.1
    }
}

impl From<String> for TokenList {
    fn from(src: String) -> TokenList {
        let mut token_list: Vec<Token> = Vec::new();

        macro_rules! last_reserved {
            ( $string:expr, $ch:expr ) => {{
                $string.push($ch);
                let new = $string.clone();
                token_list.remove(token_list.len() - 1);
                Some(Reserved(new))
            }};
        }

        macro_rules! add_to_string_literal {
            ( $string:expr, $escape_next_char:expr, $last_char_escaped:expr, $ch:expr ) => {{
                if *$escape_next_char {
                    *$escape_next_char = false;
                    *$last_char_escaped = true;
                }
                $string.push($ch);
                None
            }};
        }

        fn check_token(tokenlist: &mut Vec<Token>, index: usize) {
            match tokenlist.last().unwrap_or(&Space("".to_string())) {
                Instruction(string) => {
                    if string.len() == 3 {
                        let lcs = &string.to_lowercase()[..];
                        if let "nan" | "inf" = lcs {
                            let new = string.clone();
                            tokenlist[index] = NumberLiteral {
                                string: new,
                                hex: false,
                                exponent: false,
                                dec_point: false,
                                last_char_is_exponent: false,
                                signed: false,
                            };
                        }
                    }
                }
                Reserved(string) => {
                    if string.len() == 4 {
                        let lcs = &string.to_lowercase()[..];
                        if let "+inf" | "-inf" = lcs {
                            let new = string.clone();
                            tokenlist[index] = NumberLiteral {
                                string: new,
                                hex: false,
                                exponent: false,
                                dec_point: false,
                                last_char_is_exponent: false,
                                signed: false,
                            };
                        }
                    }
                }
                StringLiteral {
                    string,
                    last_char_escaped,
                    ..
                } => {
                    let last_char = unsafe { string.chars().last().unwrap_unchecked() };
                    if last_char != '"' || *last_char_escaped {
                        let new = string.clone();
                        tokenlist[index] = Reserved(new);
                    }
                }
                _ => {}
            };
        }

        for ch in src.chars() {
            let new_token: Option<Token> = {
                let mut space = Space("".to_string());
                #[allow(unused_mut)]
                let mut last_token = token_list.last_mut().unwrap_or(&mut space);
                unsafe {
                    match ch {
                        '\u{09}' | '\u{0A}' | '\u{0D}' => match last_token {
                            Comment {
                                string, multiline, ..
                            } if *multiline => {
                                string.push(ch);
                                None
                            }
                            Space(string) => {
                                string.push(ch);
                                None
                            },
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(Space(String::from(ch))),
                        },
                        ' ' => match last_token {
                            Comment { string, .. } => {
                                string.push(' ');
                                None
                            }
                            Space(string) => {
                                string.push(ch);
                                None
                            },
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(Space(String::from(' '))),
                        },
                        '(' => match last_token {
                            Comment { string, .. } => {
                                string.push('(');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(LParen),
                        },
                        ')' => match last_token {
                            Comment {
                                string,
                                multiline,
                                nested_level,
                            } if *multiline => {
                                if string.chars().last().unwrap_unchecked() == ';'
                                    && string.chars().rev().nth(1).unwrap_unchecked() != '('
                                {
                                    *nested_level -= 1;
                                }
                                string.push(')');
                                if *nested_level == 0 {
                                    Some(Space(ch.to_string()))
                                } else {
                                    None
                                }
                            }
                            Comment { string, .. } => {
                                string.push(')');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(RParen),
                        },
                        '@' => match last_token {
                            Space(_) | LParen | RParen => Some(Annotation("@".to_string())),
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push('@');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(Reserved("@".to_string())),
                        },
                        '$' => match last_token {
                            Space(_) | LParen | RParen => Some(Identifier("$".to_string())),
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push('$');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(Reserved("$".to_string())),
                        },
                        ';' => match last_token {
                            Reserved(s) if *s == ";" => {
                                token_list.pop();
                                Some(Comment {
                                    string: ";;".to_string(),
                                    multiline: false,
                                    nested_level: 0,
                                })
                            }
                            LParen => {
                                token_list.pop();
                                Some(Comment {
                                    string: "(;".to_string(),
                                    multiline: true,
                                    nested_level: 1,
                                })
                            }
                            Comment {
                                string,
                                multiline,
                                nested_level,
                            } => {
                                if *multiline && string.chars().last().unwrap_unchecked() == '(' {
                                    *nested_level += 1;
                                }
                                string.push(';');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            _ => Some(Reserved(";".to_string())),
                        },
                        'x' | 'X' => match last_token {
                            NumberLiteral {
                                string,
                                hex,
                                signed,
                                ..
                            } if !*hex
                                && ((!*signed
                                    && string.len() == 1
                                    && string.chars().next().unwrap_unchecked() == '0')
                                    || (*signed
                                        && string.len() == 2
                                        && string.chars().nth(1).unwrap_unchecked() == '0')) =>
                            {
                                string.push('x');
                                *hex = true;
                                None
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Comment { string, .. }
                            | Reserved(string)
                            | Instruction(string)
                            | Identifier(string)
                            | Annotation(string) => {
                                string.push('x');
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            LParen | RParen | Space(_) => Some(Instruction(String::from("x"))),
                        },
                        '.' => match last_token {
                            NumberLiteral {
                                string,
                                dec_point,
                                exponent: false,
                                ..
                            } if !*dec_point => {
                                string.push('.');
                                *dec_point = true;
                                None
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Comment { string, .. }
                            | Reserved(string)
                            | Instruction(string)
                            | Identifier(string)
                            | Annotation(string) => {
                                string.push('.');
                                None
                            }
                            LParen | RParen | Space(_) => Some(Reserved(String::from("."))),
                        },
                        '"' => match last_token {
                            StringLiteral {
                                string,
                                escape_next_char,
                                ..
                            } => {
                                if !*escape_next_char {
                                    string.push('"');
                                    Some(Space("".to_string()))
                                } else {
                                    *escape_next_char = false;
                                    string.push('"');
                                    None
                                }
                            }
                            _ => Some(StringLiteral {
                                string: '"'.to_string(),
                                escape_next_char: false,
                                last_char_escaped: false,
                            }),
                        },
                        '0'..='9' => match last_token {
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            NumberLiteral {
                                string,
                                last_char_is_exponent,
                                ..
                            } => {
                                string.push(ch);
                                *last_char_is_exponent = false;
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Space(_) | LParen | RParen => Some(NumberLiteral {
                                string: ch.to_string(),
                                hex: false,
                                dec_point: false,
                                exponent: false,
                                last_char_is_exponent: false,
                                signed: false,
                            }),
                        },
                        'e' | 'E' => match last_token {
                            NumberLiteral {
                                string,
                                hex: false,
                                exponent,
                                last_char_is_exponent,
                                ..
                            } if !*exponent => {
                                *exponent = true;
                                *last_char_is_exponent = true;
                                string.push(ch);
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            NumberLiteral {
                                string, hex: true, ..
                            } => {
                                string.push(ch);
                                None
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space(_) | LParen | RParen => Some(Instruction(ch.to_string())),
                            Instruction(string)
                            | Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        'p' | 'P' => match last_token {
                            NumberLiteral {
                                string,
                                hex: true,
                                exponent,
                                last_char_is_exponent,
                                ..
                            } if !*exponent => {
                                *exponent = true;
                                *last_char_is_exponent = true;
                                string.push(ch);
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space(_) | LParen | RParen => Some(Instruction(ch.to_string())),
                            Instruction(string)
                            | Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        'A'..='F' | 'a'..='f' => match last_token {
                            NumberLiteral {
                                string,
                                hex: true,
                                last_char_is_exponent,
                                ..
                            } => {
                                string.push(ch);
                                *last_char_is_exponent = false;
                                None
                            }
                            NumberLiteral {
                                string, hex: false, ..
                            } => last_reserved!(string, ch),
                            Space(_) | LParen | RParen => Some(Instruction(ch.to_string())),
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                        },
                        'G'..='Z' | 'g'..='z' => match last_token {
                            Space(_) | LParen | RParen => Some(Instruction(ch.to_string())),
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                        },
                        '\\' => match last_token {
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                *last_char_escaped = *escape_next_char;
                                *escape_next_char = !*escape_next_char;
                                string.push('\\');
                                None
                            }
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space(_) | RParen | LParen => Some(Reserved(ch.to_string())),
                        },
                        '+' | '-' => match last_token {
                            LParen | RParen | Space(_) => Some(NumberLiteral {
                                string: ch.to_string(),
                                signed: true,
                                hex: false,
                                dec_point: false,
                                exponent: false,
                                last_char_is_exponent: false,
                            }),
                            NumberLiteral {
                                string,
                                exponent,
                                last_char_is_exponent,
                                ..
                            } => {
                                if !*exponent {
                                    last_reserved!(string, ch)
                                } else if *last_char_is_exponent {
                                    string.push(ch);
                                    *last_char_is_exponent = false;
                                    None
                                } else {
                                    last_reserved!(string, ch)
                                }
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        '_' => match last_token {
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            NumberLiteral {
                                string,
                                signed,
                                last_char_is_exponent,
                                ..
                            } => {
                                if *last_char_is_exponent || (*signed && string.len() == 1) {
                                    last_reserved!(string, ch)
                                } else {
                                    string.push('_');
                                    None
                                }
                            }
                            Space(_) | RParen | LParen => Some(Reserved(ch.to_string())),
                        },
                        '!' | '#' | '%' | '&' | '\'' | '*' | '/' | ':' | '<' | '=' | '>' | '?'
                        | '^' | '`' | '|' | '~' => match last_token {
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string)
                            | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space(_) | RParen | LParen => Some(Reserved(ch.to_string())),
                        },
                        _ => match last_token {
                            Reserved(string) => {
                                string.push(ch);
                                None
                            }
                            StringLiteral {
                                string,
                                escape_next_char,
                                last_char_escaped,
                            } => {
                                add_to_string_literal!(
                                    string,
                                    escape_next_char,
                                    last_char_escaped,
                                    ch
                                )
                            }
                            Comment { string, .. }
                            | Annotation(string)
                            | Identifier(string)
                            | Instruction(string) => last_reserved!(string, ch),
                            _ => Some(Reserved(ch.to_string())),
                        },
                    }
                }
            };
            if let Some(token) = new_token {
                token_list.push(token);
                let len = token_list.len();
                if len > 1 {
                    check_token(&mut token_list, len - 2);
                }
            }
        }
        let len = token_list.len();
        if len > 0 {
            check_token(&mut token_list, len - 1);
        }
        token_list.retain(|t| !matches!(t, Space(_)));
        TokenList(token_list, src)
    }
}

#[rustversion::nightly]
#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn lparen() {
        let s = "(".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(l.clone().list().get(0).unwrap(), LParen);
    }

    #[test]
    fn rparen() {
        let s = ")".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(l.clone().list().get(0).unwrap(), RParen);
    }

    #[test]
    fn space() {
        let s = " ".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 0);
    }

    #[test]
    fn nl() {
        let s = "\n".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 0);
    }

    #[test]
    fn int() {
        let s = "1024375869".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn hex_int() {
        let s = "0x1024a65b98cd37ef".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn float() {
        let s = "3.14".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn float_dot_exponent() {
        let s = "3.5e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn float_exponent() {
        let s = "4e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn hex_float() {
        let s = "0x3.e".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn hex_float_dot_exponent() {
        let s = "0xa.43pd7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn hex_float_exponent() {
        let s = "0x5ap9b".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_int() {
        let s = "+1024375869".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-1024375869".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_hex_int() {
        let s = "+0x1024a65b98cd37ef".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-0x1024a65b98cd37ef".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_float() {
        let s = "+3.14".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-3.14".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_float_dot_exponent() {
        let s = "+3.5e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-3.5e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_float_dot_signed_exponent() {
        let s = "+3.5e+7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-3.5e+7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "+3.5e-7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-3.5e-7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_float_exponent() {
        let s = "+4e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-4e7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_float_signed_exponent() {
        let s = "+4e+7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-4e+7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "+4e-7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-4e-7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_hex_float() {
        let s = "+0x3.e".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-0x3.e".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_hex_float_dot_exponent() {
        let s = "+0xa.43pd7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-0xa.43pd7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn signed_hex_float_dot_signed_exponent() {
        let s = "+0xa.43p+d7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-0xa.43p+d7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "+0xa.43p-d7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-0xa.43p-d7".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn hex_float_signed_exponent() {
        let s = "0x5ap+9b".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "0x5ap-b".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn nan() {
        let s = "nan".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn inf() {
        let s = "inf".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "+inf".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "-inf".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn numbers_with_underscores() {
        let s = "3.141_592_653_589_794_232".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
        let s = "0x1fd_a4c_0b8_afe_794_36d".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            NumberLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn string() {
        let s = r#""Hello, wørld!""#.to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            StringLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn string_hex() {
        let s = r#""\a2\f6.""#.to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            StringLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn string_escaped_quote() {
        let s = r#""Hello, \"world\"!""#.to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            StringLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn string_double_escape_next_char() {
        let s = r#""\\ \\""#.to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            StringLiteral { string, .. } if *string == s
        );
    }

    #[test]
    fn unterminated_string() {
        let s = r#""this should be reserved"#.to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Reserved(string) if *string == s
        );
    }

    #[test]
    fn identifier() {
        let s = "$this_isAn-identifier".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Identifier(string) if *string == s
        );
    }

    #[test]
    fn annotation() {
        let s = "@annotation".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Annotation(string) if *string == s
        );
    }

    #[test]
    fn comment() {
        let s = ";; $this is @ comment(!)".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Comment { string, .. } if *string == s
        );
    }

    #[test]
    fn multiline_comment() {
        let s = "(;$this is @ multiline comment(!)\n\tso exciting.\n;)".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Comment { string, .. } if *string == s
        );
    }

    #[test]
    fn nested_comment() {
        let s = "(; nested (; block (; comment ;)\n;)\n\t;)".to_string();
        let l = TokenList::from(s.clone());
        assert_eq!(l.clone().list().len(), 1);
        assert_matches!(
            l.clone().list().get(0).unwrap(),
            Comment { string, .. } if *string == s
        );
    }
}

#[rustversion::nightly]
#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;
    const EXPONENT: &'static str =
        "12345679909877665543113468887665432345780824694314159265358979423.2e5";
    const HEX_EXPONENT: &'static str =
        "0x.abc452084d7385f32adafee354c547397c04158b0752a6c8f708a09c0b764.fp9";

    #[bench]
    fn bench_exponent(b: &mut Bencher) {
        b.iter(|| TokenList::from(EXPONENT.to_string()));
    }

    #[bench]
    fn bench_hex_exponent(b: &mut Bencher) {
        b.iter(|| TokenList::from(HEX_EXPONENT.to_string()));
    }

    #[bench]
    fn bench_unterminated_string(b: &mut Bencher) {
        let s = r#""this should be reserved"#.to_string();
        b.iter(|| TokenList::from(s.clone()));
    }

    #[bench]
    fn bench_signed_hex_float_dot_sigmed_exponent(b: &mut Bencher) {
        let s = "-0x13fa25a37bbd4c850a6274c869d806a367bc9f75ea6f5cd86.1a5b8c7d4f6a65296bca966fc064e95b96a9be8d856abc8tp+1bc85de0".to_string();
        b.iter(|| TokenList::from(s.clone()));
    }
}
