use std::iter::Peekable;

use crate::token::Token;


#[derive(Debug, Clone)]
pub enum NodeType {
	ClosureStart,
	ClosureEnd,
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
	ExprBinDiv
}

#[derive(Debug, Clone)]
pub struct Node {
	pub variant: NodeType,
	pub parent: Option<usize>,
}


#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
	EndOfClosure,
	UnexpectedToken
}

pub struct Parser<I: Iterator<Item = Token>> {
    input: Peekable<I>,
    nodes: Vec<Node>,
	closures: Vec<usize>
}
	
impl <I: Iterator<Item = Token>> Parser<I> {
    pub fn parse(iterator: I) -> Result<Vec<Node>, ParserError> {

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
			Some(Token::If) => self.parse_conditional(),
			Some(Token::While) => self.parse_loop(),
			Some(Token::Exit) => self.parse_function(),
			Some(Token::Ident(_)) => self.parse_reassignment(),
			Some(Token::Sep) => { self.input.next(); Ok(()) },
			Some(Token::RBrace) => Err(ParserError::EndOfClosure),
			Some(_) => Err(ParserError::UnexpectedToken),
			None => Err(ParserError::EndOfInput)
		}
	}

	fn parse_closure(&mut self) -> Result<(), ParserError> {
		match self.input.next() {
			Some(Token::LBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::ClosureStart,
			parent: self.closures.last().copied()
		});

		self.closures.push(self.nodes.len() - 1);

		loop {
			match self.parse_node() {
				Ok(_) => (),
				Err(ParserError::EndOfClosure) => break,
				Err(err) => return Err(err),
			}
		}

		match self.input.next() {
			Some(Token::RBrace) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::ClosureEnd,
			parent: self.closures.last().copied()
		});

		self.closures.pop();

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
			parent: self.closures.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	///	Parses a conditional statement from input
	/// 
	/// Expects:
	/// - if <expr> <closure>
	/// - if <expr> <closure> else <closure>
	/// 
	/// Returns:
	/// - <if> <expr> <closure>[1/2]
	/// 
	fn parse_conditional(&mut self) -> Result<(), ParserError> {
		match self.input.next() {
			Some(Token::If) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtIf(0),
			parent: self.closures.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		self.parse_closure()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		// Else
		match self.input.next_if_eq(&Token::Else) {
			None => return Ok(()),
			_ => ()
		}
		self.nodes.get_mut(index).unwrap().variant = NodeType::StmtIf(1);

		self.parse_closure()?;
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
			Some(Token::Eq) => (),
			 _ => return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtNewVar(ident_name.clone()),
			parent: self.closures.last().copied()
		});
		let index = self.nodes.len() - 1;
 
		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		Ok(())
	}

	/// Parses a loop from the input
	/// 
	/// Expects
	/// - while <expr> <closure>
	/// 
	/// Returns (unusual)
	/// - <while> <expr> <closure> 
	/// 
	fn parse_loop(&mut self) -> Result<(), ParserError> {
		let Some(Token::While) = self.input.next() else {
			return Err(ParserError::UnexpectedToken)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtWhile,
			parent: self.closures.last().copied()
		});
		let index = self.nodes.len() - 1;

		self.parse_expression()?;
		self.nodes.last_mut().unwrap().parent = Some(index);

		self.parse_closure()?;
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
			Some(Token::Eq) => (),
			Some(_) => return Err(ParserError::UnexpectedToken),
			None => return Err(ParserError::EndOfInput)
		};

		self.nodes.push(Node {
			variant: NodeType::StmtAssign(ident_name),
			parent: self.closures.last().copied()
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
				NodeType::ExprBinDiv => 2,
				NodeType::ExprBinMul => 2,
				NodeType::ExprBinAdd => 1,
				NodeType::ExprBinSub => 1,
				_ => 0,
			}
		}	

		while let Some(token) = self.input.peek() {
			let parent = self.closures.last().copied();
			let variant = match token {
				Token::Ident(name) => NodeType::ExprIdent(name.to_string()),
				Token::IntLiteral(value) => NodeType::ExprLiteral(*value),
				Token::LParen => NodeType::ExprParen,
				Token::RParen => NodeType::ExprParen,
				Token::Plus => NodeType::ExprBinAdd,
				Token::Minus => NodeType::ExprBinSub,
				Token::Star => NodeType::ExprBinMul,
				Token::FSlash => NodeType::ExprBinDiv,
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
				NodeType::ExprBinMul | NodeType::ExprBinDiv
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
				parent: self.closures.last().copied()
			});
		}

		Ok(())
	}
}
