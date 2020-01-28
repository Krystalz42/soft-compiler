use std::rc::Rc;
use std::fs::File;
use std::io::Read;
use crate::lexer::KeywordMap;
use crate::data::{UnOp, BiOp};
use std::fmt::{Formatter, Error};


#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token {
	pub line: usize,
	pub position: usize,
	pub token: TokenType,
}

impl std::fmt::Debug for Token {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f, "{:?}", self.token)
	}
}

impl Token {
	pub fn new(token: TokenType, position: usize, line: usize) -> Token {
		Token { line, position, token }
	}
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
	Int(i32),
	Char(u8),
}


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
	Keyword(Keyword),
	Identifier(String),
	Literal(Value),
	OpenParentheses,			// (
	CloseParentheses,			// )
	OpenBrace,					// {
	CloseBrace,					// }
	Semicolon,					// ;
	Whitespace,					// ' '
	Minus,						// -
	Bitwise,					// ~
	LogicalNegation,			// !
	Addition,					// +
	Multiplication,				// *
	Division					// /
}

impl TokenType {
	#[allow(dead_code)]
	pub fn to_unary_operator(&self) -> UnOp {
		match self {
			TokenType::Minus => UnOp::Negation,
			TokenType::Bitwise => UnOp::Bitwise,
			TokenType::LogicalNegation => UnOp::LogicalNegation,
			_ => panic!("critical error")
		}
	}

	#[allow(dead_code)]
	pub fn to_binary_operator(&self) -> BiOp {
		match self {
			TokenType::Minus => BiOp::Minus,
			TokenType::Addition => BiOp::Addition,
			TokenType::Multiplication => BiOp::Multiplication,
			TokenType::Division => BiOp::Division,
			_ => panic!("critical error")
		}
	}

	#[allow(dead_code)]
	pub fn is_unary_operator(&self) -> bool {
		*self == TokenType::Minus || *self == TokenType::Bitwise || *self == TokenType::LogicalNegation
	}

	#[allow(dead_code)]
	pub fn is_binary_operator(&self) -> bool {
		*self == TokenType::Minus || *self == TokenType::Addition || *self == TokenType::Multiplication || *self == TokenType::Division
	}
}

#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
pub enum Keyword {
	Return,
	Int,
}

#[derive(Debug)]
pub enum CharacterType {
	Whitespace,
	Alphabetic,
	Numeric,
	NewLine,
	NonAlphabetic(char),
}

#[warn(dead_code)]
#[derive(Debug)]
pub struct Tokenizer {
	ptr: Vec<char>,
	pub file: String,
	position: usize,
	line: usize,
	pub tokens: Vec<Token>,
}


impl Tokenizer {
	pub fn new(filename: Rc<String>) -> Self {
		let file = Self::read_file(filename);
		println!("{}", file);
		Tokenizer {
			ptr: file.chars().collect(),
			file,
			position: 0,
			line: 0,
			tokens: vec![],
		}
	}

	fn read_file(filename: Rc<String>) -> String {
		let mut s = String::new();
		let mut file = File::open(filename.as_str()).expect("File not found");
		file.read_to_string(&mut s).expect("Error reading file");

		s
	}

	fn add_token(&mut self, token: TokenType) {
		self.tokens.push(Token::new(token, self.position, self.line));
	}
}

impl Tokenizer {
	fn get_char_type(&self, advance_from_pos: usize) -> Option<CharacterType> {
		self.ptr.get(self.position + advance_from_pos).map(|c| {
			if c.is_alphabetic() || c == &'_' {
				CharacterType::Alphabetic
			} else if c.is_ascii_digit() {
				CharacterType::Numeric
			} else if c == &' ' || c == &'\t' {
				CharacterType::Whitespace
			} else if c == &'\n' || c == &'\r' {
				CharacterType::NewLine
			} else {
				CharacterType::NonAlphabetic(*c)
			}
		})
	}

	pub fn tokenize(&mut self, keyword_map: &KeywordMap) {
		while let Some(ch) = self.get_char_type(0) {
			match ch {
				CharacterType::Whitespace => self.position += 1,
				CharacterType::Alphabetic => self.get_identifier(keyword_map),
				CharacterType::Numeric => self.get_literal(),
				CharacterType::NewLine => {
					self.position += 1;
					self.line += 1;
				}
				CharacterType::NonAlphabetic(c) => {
					match c {
						'(' => { self.add_token(TokenType::OpenParentheses) }
						')' => { self.add_token(TokenType::CloseParentheses) }
						'{' => { self.add_token(TokenType::OpenBrace) }
						'}' => { self.add_token(TokenType::CloseBrace) }
						';' => { self.add_token(TokenType::Semicolon) }
						'~' => { self.add_token(TokenType::Bitwise) }
						'!' => { self.add_token(TokenType::LogicalNegation) }
						'-' => { self.add_token(TokenType::Minus) }
						'*' => { self.add_token(TokenType::Multiplication) }
						'/' => { self.add_token(TokenType::Division) }
						'+' => { self.add_token(TokenType::Addition) }
						_ => {}
					}
					self.position += 1;
				}
			}
		}
	}

	fn get_identifier(&mut self, keyword_map: &KeywordMap) {
		let mut len = 1;
		while let Some(c) = self.ptr.get(self.position + len) {
			if c.is_alphabetic() || c.is_ascii_digit() || c == &'_' {
				len += 1;
				continue;
			}
			break;
		}
		let value: String = self.ptr[self.position..self.position + len].iter().collect();
		if let Some(keyword) = keyword_map.get(value.as_str()) {
			self.add_token(TokenType::Keyword(*keyword))
		} else {
			self.add_token(TokenType::Identifier(value.into()))
		}
		self.position += len;
	}

	fn get_literal(&mut self) {
		let mut len = 1;
		while let Some(c) = self.ptr.get(self.position + len) {
			if c.is_ascii_digit() {
				len += 1;
				continue;
			}
			break;
		}
		let value: String = self.ptr[self.position..self.position + len].iter().collect();
		self.add_token(TokenType::Literal(Value::Int(value.parse().expect("Error parsing literal value"))));
		self.position += len;
	}
}
