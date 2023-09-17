use std::iter::Peekable;

use crate::token::Token;


#[derive(Debug, Clone)]
pub enum NodeType {
	// Closure,
	StmtFunction(String),
	StmtAssign(String),
	StmtReassign(String),
	ExprIdent(String),
	ExprLiteral(u32),
	ExprBinAdd,
	ExprBinSub,
	ExprBinMul,
	ExprBinDiv
}

#[derive(Debug, Clone)]
pub struct ChildNode {
	pub variant: NodeType,
	pub parent: Option<usize>,
}


#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
	UnexpectedToken
}

pub struct Parser<I: Iterator<Item = Token>> {
    input: Peekable<I>,
    nodes: Vec<ChildNode>,
	closures: Vec<usize>
}
	
impl <I: Iterator<Item = Token>> Parser<I> {
    pub fn parse(iterator: I) -> Result<Vec<ChildNode>, ParserError> {

		let input = iterator.peekable();

		let mut parser: Parser<I> = Parser {
			input,
			nodes: Vec::new(),
			closures: Vec::new()
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
		match self.input.peek() {
			Some(Token::LBrace) => self.parse_closure(),
			Some(Token::Let) => self.parse_assignment(),
			Some(Token::Exit) => self.parse_function(),
			Some(Token::Ident(_)) => self.parse_reassignment(),
			Some(Token::Sep) => {
				self.input.next();
				Ok(())
			},
			Some(_) => Err(ParserError::UnexpectedToken),
			None => Err(ParserError::EndOfInput)
		}
	}

	fn parse_closure(&mut self) -> Result<(), ParserError> {
		match self.input.next() {
			Some(Token::LBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		// Push closure node
		// Push to self.closure

		//parse

		match self.input.next() {
			Some(Token::RBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};
		Ok(())
	}

	fn parse_function(&mut self) -> Result<(), ParserError> {
		/* return <expr> */

		match self.input.next() {
			Some(Token::Exit) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(ChildNode {
			variant: NodeType::StmtFunction(String::from("exit")),
			parent: self.closures.last().copied()
		});
	
		self.parse_expression()?;

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

		self.nodes.push(ChildNode {
			variant: NodeType::StmtAssign(ident_name),
			parent: self.closures.last().copied()
		});

		self.parse_expression()?;

		Ok(())
	}

	fn parse_reassignment(&mut self) -> Result<(), ParserError> {
		/* let <Ident> = <expr> */ 

		let ident_name = match self.input.next() {
			Some(Token::Ident(ident)) => ident,
			Some(_) => return Err(ParserError::UnexpectedToken),
			None => return Err(ParserError::EndOfInput)
		};

		match self.input.next() {
			Some(Token::Eq) => (),
			Some(_) => return Err(ParserError::UnexpectedToken),
			None => return Err(ParserError::EndOfInput)
		};

		self.nodes.push(ChildNode {
			variant: NodeType::StmtReassign(ident_name),
			parent: self.closures.last().copied()
		});

		self.parse_expression()?;

		Ok(())
	}

	fn parse_expression(&mut self) -> Result<(), ParserError> {
		let mut operators: Vec<NodeType> = Vec::new();
		let expr_parent = Some(self.nodes.len() - 1);

		#[inline(always)]
		fn precedence(node_type: &NodeType) -> usize {
			match node_type {
				NodeType::ExprBinDiv => 2,
				NodeType::ExprBinMul => 2,
				NodeType::ExprBinAdd => 1,
				NodeType::ExprBinSub => 1,
				_ => 0,
			}
		}	

		while let Some(token) = self.input.peek() {
			let variant = match token {
				Token::Ident(name) => NodeType::ExprIdent(name.to_string()),
				Token::IntLiteral(value) => NodeType::ExprLiteral(*value),
				Token::Plus => NodeType::ExprBinAdd,
				Token::Minus => NodeType::ExprBinSub,
				Token::Star => NodeType::ExprBinMul,
				Token::FSlash => NodeType::ExprBinDiv,
				// Token::LParen => todo!(),
				// Token::RParen => todo!(),
				_ => break,
			};
			
			let parent = expr_parent;
			let node_precedence = precedence(&variant);

			if node_precedence == 0 {
				self.nodes.push(ChildNode {
					variant,
					parent
				});	
				self.input.next();
				continue;
			}

			while let Some(stack_variant) = operators.pop() {
				if node_precedence > precedence(&stack_variant) {
					operators.push(stack_variant);
					break;
				}
				self.nodes.push(ChildNode {
					variant: stack_variant,
					parent
				});
			}
			operators.push(variant);

			self.input.next();
		}

		while let Some(variant) = operators.pop() {
			self.nodes.push(ChildNode {
				variant,
				parent: expr_parent
			});
		}

		Ok(())
	}
}
