use std::iter::Peekable;

use crate::token::Token;


#[derive(Debug, Clone)]
pub enum NodeType {
	StmtFunction(String),
	StmtAssign(String),
	ExprIdent(String),
	ExprLiteral(u32),
	ExprBinAdd,
	ExprBinSub,
	ExprBinMul,
	ExprBinDiv
}

#[derive(Debug, Clone)]
pub struct Node {
	pub variant: NodeType,
	pub left: Option<usize>,
	pub right: Option<usize>
}


#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
	UnexpectedToken
}

pub struct Parser<I: Iterator<Item = Token>> {
    input: Peekable<I>,
    nodes: Vec<Node>
}
	
impl <I: Iterator<Item = Token>> Parser<I> {
    pub fn parse(iterator: I) -> Result<Vec<Node>, ParserError> {

		let input = iterator.peekable();

		let mut parser: Parser<I> = Parser {
			input,
			nodes: Vec::new()
		};

		loop {
			match parser.parse_node() {
				Ok(_) => (),
				Err(ParserError::EndOfInput) => break,
				Err(err) => return Err(err),
			}
		}

		Ok(parser.nodes)
	}

	fn parse_node(&mut self) -> Result<(), ParserError> {
		let Some(token) = self.input.peek() else {
			return Err(ParserError::EndOfInput)
		};
		match token {
			Token::Let => self.parse_assignment(),
			Token::Exit => self.parse_function(),
			// Token::Ident(value) => Node::Ident(*value),
			// Token::IntLiteral(_) => todo!(),
			// Token::Eq => Node,
			Token::Sep => {
				self.input.next();
				Ok(())
			},
			_ => Ok(())
		}
	}

	fn parse_function(&mut self) -> Result<(), ParserError> {
		/* return <expr> */

		match self.input.next() {
			Some(Token::Exit) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		// let using_paren = match self.input.peek() {
		// 	Some(Token::LParen) => true,
		// 	_ => false
		// };

		self.parse_expression()?;

		// if using_paren {
		// 		match self.input.next() {
		// 		Some(Token::RParen) => (),
		// 		_ => return Err(ParserError::UnexpectedToken)
		// 	};
		// }

		self.nodes.push(Node {
			variant: NodeType::StmtFunction(String::from("exit")),
			left: Some(self.nodes.len() - 1),
			right: None
		});

		Ok(())
	}

	fn parse_assignment(&mut self) -> Result<(), ParserError> {
		/* let <var> = <expr> */ 

		match self.input.next() {
			Some(Token::Let) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		let ident_name = self.input.next()
			.ok_or(ParserError::EndOfInput)
			.and_then(|token| if let Token::Ident(value) = token { Ok(value) } else { Err(ParserError::EndOfInput) })?;

		match self.input.next() {
			Some(Token::Eq) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.parse_expression()?;


		// Assignment node
		self.nodes.push(Node {
			variant: NodeType::StmtAssign(ident_name),
			left: Some(self.nodes.len() - 1),
			right: None
		});

		Ok(())
	}

	fn parse_expression(&mut self) -> Result<(), ParserError> {
		// expr = ident | literal | expr

		let mut operators = Vec::new();

		#[inline(always)]
		fn precedence(token: Option<&Token>) -> usize {
			match token {
				Some(Token::FSlash) => 2,
				Some(Token::Star) => 2,
				Some(Token::Minus) => 1,
				Some(Token::Plus) => 1,
				_ => 0,
			}
		}	

		while let Some(node) = self.input.peek() {

			match node {
				Token::Ident(name) => self.nodes.push(Node {
					variant: NodeType::ExprIdent(name.to_owned()), left: None, right: None }),

				Token::IntLiteral(value) => self.nodes.push(Node {
					variant: NodeType::ExprLiteral(*value), left: None, right: None }),

				token
				if token == &Token::Plus 
				|| token == &Token::Minus
				|| token == &Token::Star
				|| token == &Token::FSlash => {
					while precedence(operators.last()) > precedence(Some(token)) {
						let variant = match token {
							Token::Plus => NodeType::ExprBinAdd,
							Token::Minus => NodeType::ExprBinSub,
							Token::Star => NodeType::ExprBinMul,
							Token::FSlash => NodeType::ExprBinDiv,
							_ => unreachable!()
						};
						let right = Some(self.nodes.len() - 1);
						let left = Some({
							let mut index = self.nodes.len() - 1;
							loop {
								let Some(node) = self.nodes.get(index) else {
									break
								};
								match node.left {
									Some(new_index) => index = new_index,
									None => break,
								}
							}
							index - 1
						});
						self.nodes.push(Node {
							variant,
							left,
							right
						});
					}
					operators.push(token.clone())
				}
				// Token::LParen => todo!(),
				// Token::RParen => todo!(),
				_ => break
			}

			self.input.next();
		}

		while let Some(token) = operators.pop() {
			let variant = match token {
				Token::Plus => NodeType::ExprBinAdd,
				Token::Minus => NodeType::ExprBinSub,
				Token::Star => NodeType::ExprBinMul,
				Token::FSlash => NodeType::ExprBinDiv,
				_ => unreachable!()
			};
			let right = Some(self.nodes.len() - 1);
			let left = Some({
				let mut index = self.nodes.len() - 1;
				loop {
					let Some(node) = self.nodes.get(index) else {
						break
					};
					match node.left {
						Some(new_index) => index = new_index,
						None => break,
					}
				}
				index - 1
			});
			self.nodes.push(Node {
				variant,
				left,
				right
			});
		}

		Ok(())
	}
}
