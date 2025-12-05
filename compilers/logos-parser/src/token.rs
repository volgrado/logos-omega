use logos_protocol::LemmaId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A word found in the dictionary (resolved to a Lemma)
    Word(LemmaId),
    /// A word that looks like Greek but isn't in our dict
    UnknownWord,
    /// Punctuation mark
    Punctuation(char),
    /// Numbers, etc. (MVP placeholder)
    Other,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub span: Span,
    pub text: &'a str,
    pub kind: TokenKind,
}
