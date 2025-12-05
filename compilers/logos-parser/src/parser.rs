use nom::{
    branch::alt,
    bytes::complete::{take_while1},
    character::complete::{char, multispace0},
    combinator::{map},
    IResult,
};
use crate::token::Span;

/// Predicate to define what constitutes a "Greek Word" character.
/// Includes Standard Greek and Extended Greek (Polytonic).
fn is_greek_alphabetic(c: char) -> bool {
    // Basic Greek block: U+0370 - U+03FF
    // Greek Extended: U+1F00 - U+1FFF
    // Plus standard alphabetic check for resilience
    match c {
        '\u{0370}'..='\u{03FF}' => true,
        '\u{1F00}'..='\u{1FFF}' => true,
        _ => c.is_alphabetic(),
    }
}

#[derive(Debug, Clone)]
pub enum RawToken<'a> {
    Word(&'a str),
    Punct(char),
}

pub fn parse_with_spans(original_input: &str) -> Vec<(Span, RawToken)> {
    let mut input = original_input;
    let mut result = Vec::new();

    loop {
        // 1. Skip whitespace
        let (next_input, _) = match multispace0::<&str, nom::error::Error<&str>>(input) {
            Ok(res) => res,
            Err(_) => break,
        };
        input = next_input;

        if input.is_empty() {
            break;
        }

        // 2. Try to match a token
        let parse_res: IResult<&str, RawToken> = alt((
            map(take_while1(is_greek_alphabetic), RawToken::Word),
            map(char('.'), |_| RawToken::Punct('.')),
            map(char(','), |_| RawToken::Punct(',')),
            map(char(';'), |_| RawToken::Punct(';')),
            map(char('?'), |_| RawToken::Punct('?')),
            map(char('!'), |_| RawToken::Punct('!')),
        ))(input);

        match parse_res {
            Ok((next_input, token)) => {
                // Calculate span
                // We know 'token' came from 'input', which came from 'original_input'
                let len = input.len() - next_input.len();
                let start = input.as_ptr() as usize - original_input.as_ptr() as usize;
                
                result.push((Span::new(start, start + len), token));
                input = next_input;
            }
            Err(_) => {
                // Skip one char to recover (resilient parsing)
                if let Some(c) = input.chars().next() {
                    let len = c.len_utf8();
                    input = &input[len..];
                } else {
                    break;
                }
            }
        }
    }

    result
}
