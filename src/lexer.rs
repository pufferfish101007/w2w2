#[derive(Debug, PartialEq)]
pub enum Token {
        LParen,
        RParen,
        Identifier(String),
        Annotation(String),
        Instruction(String),
        StringLiteral { string: String, backslash: bool },
        NumberLiteral { string: String, hex: bool, dec_point: bool, exponent: bool, last_char_is_exponent: bool, signed: bool },
        Comment { string: String, multiline: bool, nested_level: i32 },
        Reserved(String),
        Space, // this may not be an actual space - it coukd be a new line, or just exist to signify the end of a literal
}

use Token::*;/*{
        LParen, RParen, Space, Identifier, Annotation, StringLiteral,
        NumberLiteral, Comment, Reserved, Instruction
};*/

pub type TokenList = Vec<Token>;

pub fn lex(src: &String) -> Option<TokenList> {
        println!("{}", src);
        let mut token_list: TokenList = Vec::new();
        
        macro_rules! last_reserved {
            ( $string:expr, $ch:expr ) => {
                {
                    $string.push($ch);
                    let new = $string.clone();
                    token_list.remove(token_list.len() - 1);
                    Some(Reserved(new))
                }
            }
        }
        
        macro_rules! add_to_string_literal {
            ( $string:expr, $backslash:expr, $ch:expr ) => {
                {
                    if *$backslash == true {
                        *$backslash = false;
                    }
                    $string.push($ch);
                    None
                }
            }
        }
        
        for ch in src.chars() {
            let new_token: Option<Token> = {
                let mut space = Space;
                #[allow(unused_mut)]
                let mut last_token = token_list.last_mut().unwrap_or(&mut space);
                unsafe {
                    match ch {
                    '\u{09}' | '\u{0A}' | '\u{0D}' => match last_token {
                            Comment { string, multiline, .. } if *multiline == true => {
                                string.push(ch);
                                None
                            },
                            Space => None,
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(Space)
                        },
                        ' ' => match last_token {
                            Comment {  string, .. } => {
                                string.push(' ');
                                None
                            },
                            Space => None,
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => {
                                Some(Space)
                            }
                        },
                        '(' => match last_token {
                            Comment { string, .. } => {
                                string.push('(');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(LParen),
                        },
                        ')' => match last_token {
                            Comment { string, multiline, nested_level } if *multiline == true => {
                                if string.chars().last().unwrap_unchecked() == ';' && string.chars().rev().nth(1).unwrap_unchecked() != '(' {
                                    *nested_level -= 1;
                                }
                                string.push(')');
                                if *nested_level == 0 {
                                    Some(Space)
                                } else {
                                    None
                                }
                            },
                            Comment { string, .. } => {
                                string.push(')');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(RParen)
                        },
                        '@' => match last_token {
                            Space | LParen | RParen => Some(Annotation("@".to_string())),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push('@');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(Reserved("@".to_string()))
                        },
                        '$' => match last_token {
                            Space | LParen | RParen => Some(Identifier("$".to_string())),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push('$');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(Reserved("$".to_string()))
                        },
                        ';' => match last_token {
                            Reserved(s) if *s == ";" => {
                                token_list.pop();
                                Some(Comment { string: ";;".to_string(), multiline: false, nested_level: 0 })
                            },
                            LParen => {
                                token_list.pop();
                                Some(Comment { string: "(;".to_string(), multiline: true, nested_level: 1 })
                            },
                            Comment {  string, multiline, nested_level } => {
                                if *multiline == true && string.chars().last().unwrap_unchecked() == '(' {
                                    *nested_level += 1;
                                }
                                string.push(';');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            _ => Some(Reserved(";".to_string()))
                        },
                        'x' | 'X' => match last_token {
                            NumberLiteral { string, hex, signed, .. } if *hex == false && ((!*signed && string.len() == 1 && string.chars().nth(0).unwrap_unchecked() == '0') || (*signed && string.len() == 2 && string.chars().nth(1).unwrap_unchecked() == '0')) => {
                                string.push('x');
                                *hex = true;
                                None
                            },
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Comment { string, .. } | Reserved(string) | Instruction(string) | Identifier(string) | Annotation(string) => {
                                string.push('x');
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            LParen | RParen | Space => Some(Instruction(String::from("x")))
                        },
                        '.' => match last_token {
                            NumberLiteral { string, dec_point, exponent: false, .. } if *dec_point == false => {
                                string.push('.');
                                *dec_point = true;
                                None
                            },
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            Comment { string, .. } | Reserved(string) | Instruction(string) | Identifier(string) | Annotation(string) => {
                                string.push('.');
                                None
                            },
                            LParen | RParen | Space => Some(Reserved(String::from(".")))
                        },
                        '"' => match last_token {
                            StringLiteral { string, backslash } => {
                                if *backslash == false {
                                    string.push('"');
                                    Some(Space)
                                } else {
                                    *backslash = false;
                                    string.push('"');
                                    None
                                }
                            },
                            _ => Some(StringLiteral { string: '"'.to_string(), backslash: false })
                        },
                        '0'..= '9' => match last_token {
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            NumberLiteral { string, last_char_is_exponent, .. } => {
                                string.push(ch);
                                *last_char_is_exponent = false;
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            Space | LParen | RParen => Some(NumberLiteral { string: ch.to_string(), hex: false, dec_point: false, exponent: false, last_char_is_exponent: false, signed: false })
                        },
                        'e' | 'E' => match last_token { 
                            NumberLiteral { string, hex: false, exponent, last_char_is_exponent, .. } if *exponent == false => {
                                *exponent = true;
                                *last_char_is_exponent = true;
                                string.push(ch);
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            NumberLiteral { string, hex: true, .. } => {
                                string.push(ch);
                                None
                            },
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space | LParen | RParen => Some(Instruction(ch.to_string())),
                            Instruction(string) | Comment { string, .. } | Annotation(string) | Identifier(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        'p' | 'P' => match last_token { 
                            NumberLiteral { string, hex: true, exponent, last_char_is_exponent, .. } if *exponent == false => {
                                *exponent = true;
                                *last_char_is_exponent = true;
                                string.push(ch);
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space | LParen | RParen => Some(Instruction(ch.to_string())),
                            Instruction(string) | Comment { string, .. } | Annotation(string) | Identifier(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        'A' ..= 'F' | 'a' ..= 'f' => match last_token {
                            NumberLiteral { string, hex: true, last_char_is_exponent, .. } => {
                                string.push(ch);
                                *last_char_is_exponent = false;
                                None
                            },
                            NumberLiteral { string, hex: false, .. } => last_reserved!(string, ch),
                            Space | LParen | RParen => Some(Instruction(ch.to_string())),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch)
                        },
                        'G' ..= 'Z' | 'g' ..= 'z' => match last_token {
                            Space | LParen | RParen => Some(Instruction(ch.to_string())),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            NumberLiteral { string , .. } => last_reserved!(string, ch)
                        },
                        '\\' => match last_token {
                            StringLiteral { string, backslash } => {
                                *backslash = !*backslash;
                                string.push('\\');
                                None
                            },
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space | RParen | LParen => Some(Reserved(ch.to_string()))
                        },
                        '+' | '-' => match last_token {
                            LParen | RParen | Space => Some(NumberLiteral { string: ch.to_string(), signed: true, hex: false, dec_point: false, exponent: false, last_char_is_exponent: false }),
                            NumberLiteral { string, hex, exponent, last_char_is_exponent, .. } => {
                                if !*exponent {
                                    last_reserved!(string, ch)
                                } else if *hex && *last_char_is_exponent {
                                    string.push(ch);
                                    *last_char_is_exponent = false;
                                    None
                                } else {
                                    last_reserved!(string, ch)
                                }
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            }
                        },
                        '!' | '#' | '%' | '&' | '\'' | '*' | '/' | ':' | '<' | '=' | '>' | '?' | '^' | '_' | '`' | '|' | '~' => match last_token {
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) | Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            NumberLiteral { string, .. } => last_reserved!(string, ch),
                            Space | RParen | LParen => Some(Reserved(ch.to_string()))
                        },
                        _ => match last_token {
                            Reserved(string) => {
                                string.push(ch);
                                None
                            },
                            StringLiteral { string, backslash } => add_to_string_literal!(string, backslash, ch),
                            Comment {  string, .. } | Annotation(string) | Identifier(string) | Instruction(string) => last_reserved!(string, ch),
                            _ => Some(Reserved(ch.to_string()))
                        }
                    }
                }
            };
            if let Some(token) = new_token {
                token_list.push(token);
            }
        };
        unsafe {
            if let StringLiteral { string, .. } = token_list.last_mut().unwrap_or(&mut Space) {
                if string.chars().last().unwrap_unchecked() != '"' {
                    let new = string.clone();
                    token_list.remove(token_list.len() - 1);
                    token_list.push(Reserved(new));
                }
            }
        }
        token_list.retain(|t| if let Space = t { false } else { true });
        Some(token_list)
}

#[cfg(test)]
mod tests {
        use std::assert_matches::assert_matches;
        use super::*;
        
        #[test]
        fn lparen() {
            let s = "(".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                LParen
            );
        }
        
        #[test]
        fn rparen() {
            let s = ")".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                RParen
            );
        }
        
        #[test]
        fn space() {
            let s = " ".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 0);
        }
        
        #[test]
        fn nl() {
            let s = "\n".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 0);
        }
        
        #[test]
        fn int() {
            let s = "1024375869".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn hex_int() {
            let s = "0x1024a65b98cd37ef".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn float() {
            let s = "3.14".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn float_dot_exponent() {
            let s = "3.5e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn float_exponent() {
            let s = "4e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn hex_float() {
            let s = "0x3.e".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn hex_float_dot_exponent() {
            let s = "0xa.43pd7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn hex_float_exponent() {
            let s = "0x5ap9b".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_int() {
            let s = "+1024375869".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-1024375869".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_hex_int() {
            let s = "+0x1024a65b98cd37ef".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-0x1024a65b98cd37ef".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_float() {
            let s = "+3.14".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-3.14".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_float_dot_exponent() {
            let s = "+3.5e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-3.5e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_float_dot_signed_exponent() {
            let s = "+3.5e+7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-3.5e+7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "+3.5e-7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-3.5e-7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_float_exponent() {
            let s = "+4e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-4e7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_float_signed_exponent() {
            let s = "+4e+7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-4e+7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "+4e-7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-4e-7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_hex_float() {
            let s = "+0x3.e".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-0x3.e".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_hex_float_dot_exponent() {
            let s = "+0xa.43pd7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-0xa.43pd7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn signed_hex_float_dot_signed_exponent() {
            let s = "+0xa.43p+d7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-0xa.43p+d7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "+0xa.43p-d7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "-0xa.43p-d7".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn hex_float_signed_exponent() {
            let s = "0x5ap+9b".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
            let s = "0x5ap-b".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                NumberLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn string() {
            let s = r#""Hello, wørld!""#.to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                StringLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn string_hex() {
            let s = r#""\a2\f6.""#.to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                StringLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn string_escaped_quote() {
            let s = r#""Hello, \"world\"!""#.to_string();
            let l = lex(&s).unwrap();
            assert_eq!(
                l.len(),
                1
            );
            assert_matches!(
                l.get(0).unwrap(),
                StringLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn string_double_backslash() {
            let s = r#""\\ \\""#.to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                StringLiteral { string, .. } if *string == s
            );
        }
        
        #[test]
        fn unterminated_string() {
            let s = r#""this should be reserved"#.to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Reserved(string) if *string == s
            );
        }
        
        #[test]
        fn identifier() {
            let s = "$this_isAn-identifier".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Identifier(string) if *string == s
            );
        }
        
        #[test]
        fn annotation() {
            let s = "@annotation".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Annotation(string) if *string == s
            );
        }
        
        #[test]
        fn comment() {
            let s = ";; $this is @ comment(!)".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Comment { string, .. } if *string == s
            );
        }
        
        #[test]
        fn multiline_comment() {
            let s = "(;$this is @ multiline comment(!)\n\tso exciting.\n;)".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Comment { string, .. } if *string == s
            );
        }
        
        #[test]
        fn nested_comment() {
            let s = "(; nested (; block (; comment ;)\n;)\n\t;)".to_string();
            let l = lex(&s).unwrap();
            assert_eq!(l.len(), 1);
            assert_matches!(
                l.get(0).unwrap(),
                Comment { string, .. } if *string == s
            );
        }
}

#[cfg(test)]
mod benches {
        use test::Bencher;
        use super::*;
        const EXPONENT: &'static str = "12345679909877665543113468887665432345780824694314159265358979423.2e5";
        const HEX_EXPONENT: &'static str = "0x.abc452084d7385f32adafee354c547397c04158b0752a6c8f708a09c0b764.fp9";
        
        #[bench]
        fn bench_exponent(b: &mut Bencher) {
            b.iter(|| lex(&EXPONENT.to_string()));
        }
        
        #[bench]
        fn bench_hex_exponent(b: &mut Bencher) {
            b.iter(|| lex(&HEX_EXPONENT.to_string()));
        }
        
        #[bench]
        fn bench_unterminated_string(b: &mut Bencher) {
            let s = r#""this should be reserved"#.to_string();
            b.iter(|| lex(&s));
        }
        
        #[bench]
        fn bench_signed_hex_float_dot_sigmed_exponent(b: &mut Bencher) {
            let s = "-0x13fa25a37bbd4c850a6274c869d806a367bc9f75ea6f5cd86.1a5b8c7d4f6a65296bca966fc064e95b96a9be8d856abc8tp+1bc85de0".to_string();
            b.iter(|| lex(&s));
        }
}