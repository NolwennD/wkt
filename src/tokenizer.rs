// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter::Peekable;
use std::marker::PhantomData;
use std::str;
use WktFloat;

#[derive(Debug, PartialEq)]
pub enum Token<T>
where
    T: WktFloat,
{
    Comma,
    Number(T),
    ParenClose,
    ParenOpen,
    Word(String),
}

fn is_whitespace(c: char) -> bool {
    match c {
        '\n' | '\r' | '\t' | ' ' => true,
        _ => false,
    }
}

fn is_numberlike(c: char) -> bool {
    match c {
        c if c.is_numeric() => true,
        '.' | '-' | '+' => true,
        _ => false,
    }
}

pub type PeekableTokens<'a, T> = Peekable<Tokens<'a, T>>;

#[derive(Debug)]
pub struct Tokens<'a, T> {
    chars: Peekable<str::Chars<'a>>,
    phantom: PhantomData<T>,
}

impl<'a, T> Tokens<'a, T>
where
    T: WktFloat,
{
    pub fn from_str(input: &'a str) -> Self {
        Tokens {
            chars: input.chars().peekable(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Tokens<'a, T>
where
    T: WktFloat + str::FromStr + Default,
{
    type Item = Token<T>;

    fn next(&mut self) -> Option<Token<T>> {
        // TODO: should this return Result?
        let mut next_char = self.chars.next()?;

        // Skip whitespace
        while is_whitespace(next_char) {
            next_char = self.chars.next()?
        }

        match next_char {
            '\0' => None,
            '(' => Some(Token::ParenOpen),
            ')' => Some(Token::ParenClose),
            ',' => Some(Token::Comma),
            c if is_numberlike(c) => {
                let number = c.to_string() + &self.read_until_whitespace().unwrap_or_default();
                match number.trim_start_matches('+').parse::<T>() {
                    Ok(parsed_num) => Some(Token::Number(parsed_num)),
                    Err(_) => None,
                }
            }
            c => {
                let word = c.to_string() + &self.read_until_whitespace().unwrap_or_default();
                Some(Token::Word(word))
            }
        }
    }
}

impl<'a, T> Tokens<'a, T>
where
    T: WktFloat + str::FromStr + Default,
{
    fn read_until_whitespace(&mut self) -> Option<String> {
        let mut result = String::new();

        while let Some(&next_char) = self.chars.peek() {
            let marker = match next_char {
                '\0' | '(' | ')' | ',' => true,
                _ => false,
            };

            // Consume non-markers
            if !marker {
                let _ = self.chars.next();
            }

            let whitespace = is_whitespace(next_char);

            // Append non-whitespace, non-marker characters
            if !marker && !whitespace {
                result.push(next_char);
            }

            // Stop reading when reached marker or whitespace
            if marker || whitespace {
                break;
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[test]
fn test_tokenizer_empty() {
    let test_str = "";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Word("hello".to_string())]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(
        tokens,
        vec![
            Token::Word("hello".to_string()),
            Token::Word("world".to_string()),
        ]
    );
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_1number_plus() {
    let test_str = "+4.2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_invalid_number() {
    let test_str = "4.2p";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".4 -2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(0.4), Token::Number(-2.0)]);
}

#[test]
fn test_no_stack_overflow() {
    fn check(c: &str, count: usize, expected: usize) {
        let test_str = c.repeat(count);
        let tokens: Vec<Token<f64>> = Tokens::from_str(&test_str).collect();
        assert_eq!(expected, tokens.len());
    }

    let count = 100_000;
    check("+", count, 0);
    check(" ", count, 0);
    check("A", count, 1);
    check("1", count, 1);
    check("(", count, count);
    check(")", count, count);
    check(",", count, count);
}

#[test]
fn test_tokenizer_point() {
    let test_str = "POINT (10 -20)";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(
        tokens,
        vec![
            Token::Word("POINT".to_string()),
            Token::ParenOpen,
            Token::Number(10.0),
            Token::Number(-20.0),
            Token::ParenClose,
        ]
    );
}
