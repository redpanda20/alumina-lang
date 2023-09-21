use std::iter::Peekable;

use crate::token::Token;


#[derive(Debug, Clone)]
pub enum NodeType {
	BlockStart,
	BlockEnd,
	StmtFunction(String),
	StmtNewVar(String),
	StmtAssign(String),
	// StmtReassign(String),
	StmtIf(usize),
	StmtWhile,
	ExprIdent(String),
	ExprLiteral(u32),
	ExprParen,
	ExprBinAdd,
	ExprBinSub,
	ExprBinMul,
	ExprBinDiv,
	ExprEqual,
	ExprNotEqual,
	ExprGreater,
	ExprLessEqual,
	ExprLess,
	ExprGreaterEqual
}

#[derive(Debug, Clone)]
pub struct Node {
	pub variant: NodeType,
	pub parent: Option<usize>,
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(parent) = self.parent {
			write!(f, "{:?}, Parent: {parent}", self.variant)
		} else {
			write!(f, "{:?}", self.variant)
		}
    }
}

#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
	EndOfBlock,
	UnexpectedToken
}

pub struct Parser<I: Iterator<Item = Token>> {
    input: Peekable<I>,
    nodes: Vec<Node>,
	blocks: Vec<usize>
}
	
impl <I: Iterator<Item = Token>> Parser<I> {
    pub fn parse(iterator: I) -> Result<Vec<Node>, ParserError> {

		let input = iterator.peekable();

		let mut parser: Parser<I> = Parser {
			input,
			nodes: Vec::new(),
			blocks: Vec::new()
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
			Some(Token::LBrace) => self.parse_block(),
			Some(Token::Let) => self.parse_assignment(),
			Some(Token::If) => self.parse_conditional(),
			Some(Token::While) => self.parse_loop(),
			Some(Token::Exit) => self.parse_function(),
			Some(Token::Ident(_)) => self.parse_reassignment(),
			Some(Token::Sep) => { self.input.next(); Ok(()) },
			Some(Token::RBrace) => Err(ParserError::EndOfBlock),
			Some(_) => Err(ParserError::UnexpectedToken),
			None => Err(ParserError::EndOfInput)
		}
	}

	fn parse_block(&mut self) -> Result<(), ParserError> {
		match self.input.next() {
			Some(Token::LBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::BlockStart,
			parent: self.blocks.last().copied()
		});

		self.blocks.push(self.nodes.len() - 1);

		loop {
			match self.parse_node() {
				Ok(_) => (),
				Err(ParserError::EndOfBlock) => break,
				Err(err) => return Err(err),
			}
		}

		match self.input.next() {
			Some(Token::RBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::BlockEnd,
			parent: self.blocks.last().copied()
		});

		self.blocks.pop();

		Ok(())
	}

	/// Parses a function
	/// Currently only handles hardcoded exit
	/// 
	/// Expects:
	/// <ident> <expr>[1+]
	/// 
	/// Returns:
	/// <function> <expr>
	fn parse_function(&mut self) -> Result<(), ParserError> {

		match self.input.next() {
			Some(Token::Exit) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtFunction(String::from("exit")),
			parent: self.blocks.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	///	Parses a conditional statement from input
	/// 
	/// Expects:
	/// - if <expr> <block>
	/// - if <expr> <block> else <block>
	/// 
	/// Returns:
	/// - <if> <expr> <block>[1/2]
	/// 
	fn parse_conditional(&mut self) -> Result<(), ParserError> {
		match self.input.next() {
			Some(Token::If) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtIf(0),
			parent: self.blocks.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		self.parse_block()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		// Else
		match self.input.next_if_eq(&Token::Else) {
			None => return Ok(()),
			_ => ()
		}
		self.nodes.get_mut(index).unwrap().variant = NodeType::StmtIf(1);

		self.parse_block()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	/// Parses an assignement expression
	/// 
	/// Expects:
	/// let <ident> = <expr>
	/// 
	/// Returns:
	/// - StmtNewVar(<ident>)
	/// - Expr
	fn parse_assignment(&mut self) -> Result<(), ParserError> {

		match self.input.next() {
			Some(Token::Let) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		let ident_name = self.input.next()
			.ok_or(ParserError::EndOfInput)
			.and_then(|token| if let Token::Ident(value) = token { Ok(value) } else { Err(ParserError::EndOfInput) })?;

		match self.input.next() {
			Some(Token::Equal) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtNewVar(ident_name.to_string()),
			parent: self.blocks.last().copied()
		});
		let index = self.nodes.len() - 1;
 
		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	/// Parses a loop from the input
	/// 
	/// Expects
	/// - while <expr> <block>
	/// 
	/// Returns (unusual)
	/// - <while> <expr> <block> 
	/// 
	fn parse_loop(&mut self) -> Result<(), ParserError> {
		let Some(Token::While) = self.input.next() else {
			return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtWhile,
			parent: self.blocks.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		self.parse_block()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	/// Parses reassignment expression
	/// 
	/// Expects:
	/// <ident> = <expr>
	/// 
	/// Returns
	/// - <StmtAssign>
	/// - <Expr>
	fn parse_reassignment(&mut self) -> Result<(), ParserError> {
		/* let <Ident> = <expr> */ 

		let ident_name = match self.input.next() {
			Some(Token::Ident(ident)) => ident,
			Some(_) => return Err(ParserError::UnexpectedToken),
			None => return Err(ParserError::EndOfInput)
		};

		match self.input.next() {
			Some(Token::Equal) => (),
			Some(_) => return Err(ParserError::UnexpectedToken),
			None => return Err(ParserError::EndOfInput)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtAssign(ident_name.to_string()),
			parent: self.blocks.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	fn parse_expression(&mut self) -> Result<(), ParserError> {
		let mut operators: Vec<NodeType> = Vec::new();

		#[inline(always)]
		fn precedence(node_type: &NodeType) -> usize {
			match node_type {
				NodeType::ExprLess => 3,
				NodeType::ExprGreaterEqual => 3,
				NodeType::ExprLessEqual => 3,
				NodeType::ExprGreater => 3,
				NodeType::ExprNotEqual => 3,
				NodeType::ExprEqual => 3,
				NodeType::ExprBinDiv => 2,
				NodeType::ExprBinMul => 2,
				NodeType::ExprBinAdd => 1,
				NodeType::ExprBinSub => 1,
				_ => 0,
			}
		}	

		while let Some(token) = self.input.peek() {
			let parent = self.blocks.last().copied();
			let variant = match token {
				Token::Ident(name) => NodeType::ExprIdent(name.to_string()),
				Token::IntLiteral(value) => NodeType::ExprLiteral(*value),
				Token::LParen => NodeType::ExprParen,
				Token::RParen => NodeType::ExprParen,
				Token::Plus => NodeType::ExprBinAdd,
				Token::Minus => NodeType::ExprBinSub,
				Token::Star => NodeType::ExprBinMul,
				Token::FSlash => NodeType::ExprBinDiv,
				Token::NotEqual => NodeType::ExprNotEqual,
				Token::EqualEqual => NodeType::ExprEqual,
				Token::Greater => NodeType::ExprGreater,
				Token::LessEqual => NodeType::ExprLessEqual,
				Token::Less => NodeType::ExprLess,
				Token::GreaterEqual => NodeType::ExprGreaterEqual,
				_ => break,
			};
			
			match variant {
				NodeType::ExprParen
				=> {
					if let Token::LParen = token {
						operators.push(NodeType::ExprParen);
					} else {
						while let Some(stack_variant) = operators.pop() {
							if let NodeType::ExprParen = stack_variant {
								break;
							}
							self.nodes.push(Node {
								variant: stack_variant,
								parent
							});
						}
						self.nodes.push(Node {
							variant,
							parent
						});
					}	
				},
				NodeType::ExprIdent(_) | NodeType::ExprLiteral(_)
				=> {
					self.nodes.push(Node {
						variant,
						parent
					});	
				},
				NodeType::ExprBinAdd | NodeType::ExprBinSub |
				NodeType::ExprBinMul | NodeType::ExprBinDiv |
				NodeType::ExprEqual  | NodeType::ExprGreater|
				NodeType::ExprLessEqual | NodeType::ExprNotEqual|
				NodeType::ExprLess | NodeType::ExprGreaterEqual
				=> {
					while let Some(stack_variant) = operators.pop() {
						if precedence(&variant) > precedence(&stack_variant) {
							operators.push(stack_variant);
							break;
						}
						self.nodes.push(Node {
							variant: stack_variant,
							parent
						});
					}
					operators.push(variant);		
				},
				_ => ()
			}

			self.input.next();
		}

		while let Some(variant) = operators.pop() {
			self.nodes.push(Node {
				variant,
				parent: self.blocks.last().copied()
			});
		}

		Ok(())
	}
}
