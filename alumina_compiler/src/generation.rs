
/* Minimum req.

global _start
section .text
_start:

linux -:
	mov rdi (val)
	mov rax 60
	syscall

win   -:
	? ? ?
*/

use std::iter::Peekable;

use crate::parser::{Node, NodeType};

#[derive(Debug)]
pub enum GeneratorError {
	EndOfInput,
	VariableAlreadyDeclared(String),
	VariableNotYetDeclared(String),
	ClosureNotYetOpened,
	UnexpectedNode(NodeType)
}

pub struct Generator<I: Iterator<Item = Node>> {
	input: Peekable<I>,
	variables: Vec<(String, usize)>,
	stack_size: usize,
	label_count: usize,
	scopes: Vec<usize>,
	output: String
}

impl <I: Iterator<Item = Node>> Generator<I> {
	pub fn generate_program(iterator: I) -> Result<String, GeneratorError> {

		let input = iterator.peekable();

		let mut generator = Generator {
			input,
			variables: Vec::new(),
			stack_size: 0,
			label_count: 0,
			scopes: Vec::new(),
			output: String::new(),
		};

		loop {
			match generator.generate_node() {
				Ok(_) => (),
				Err(GeneratorError::EndOfInput) => break,
				Err(err) => return Err(err),
			}
		}
		generator.output = String::from("global _start\nsection .text\n_start:\n") + &generator.output + "mov rdi, 0\nmov rax, 60\nsyscall";
		
		Ok(generator.output)

	}

	fn push(&mut self, reg: &str) {
		self.output += &format!("push {}\n", reg);
		self.stack_size += 1;
	}

	fn pop(&mut self, reg: &str) {
		self.output += &format!("pop {}\n", reg);
		self.stack_size -= 1;
	}

	fn create_label(&mut self, name: &str) -> String {
		let label = format!(".{}{}", name, self.label_count);
		self.label_count += 1;
		label
	}

	fn generate_node(&mut self) -> Result<(), GeneratorError> {
		let node = self.input.peek().ok_or(GeneratorError::EndOfInput)?;
		match &node.variant {
			NodeType::StmtFunction(_) => self.generate_function()?,
			NodeType::StmtIf(_) => self.generate_conditional()?,
			NodeType::StmtWhile => self.generate_loop()?,
			NodeType::StmtNewVar(_) => self.generate_variable()?,
			NodeType::StmtAssign(_) => self.generate_assignment()?,
			node_type => return Err(GeneratorError::UnexpectedNode(node_type.clone()))
		}
		Ok(())
	}

	fn generate_closure(&mut self) -> Result<(), GeneratorError> {
		match self.input.next().ok_or(GeneratorError::EndOfInput)?.variant {
			NodeType::ClosureStart => (),
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};
		
		self.scopes.push(self.variables.len());

		while let Some(node) = self.input.peek() {
			if let NodeType::ClosureEnd = node.variant {
				break;
			}
			self.generate_node()?;
		}
		self.input.next();

		let closure_start = self.scopes.pop()
			.ok_or(GeneratorError::ClosureNotYetOpened)?;
		let pop_count = self.variables.len() - closure_start;

		self.output += &format!("add rsp, {}\n", pop_count * 8);
		self.stack_size -= pop_count;
		self.variables.truncate(closure_start);	

		Ok(())
	}

	fn generate_variable(&mut self) -> Result<(), GeneratorError> {
		let node = self.input.next().ok_or(GeneratorError::EndOfInput)?;
		let name = match node.variant {
			NodeType::StmtNewVar(name) => name,
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};

		if self.variables.iter().any(|(str, _)| str == &name) {
			return Err(GeneratorError::VariableAlreadyDeclared(name.clone()));
		}

		self.generate_expr()?;
		self.output.pop(); self.output += &format!("	; variable ({}) assigned\n", name);

		self.variables.push((name.to_owned(), self.stack_size));

		Ok(())
	}
	fn generate_assignment(&mut self) -> Result<(), GeneratorError> {
		let node = self.input.next().ok_or(GeneratorError::EndOfInput)?;
		let name = match node.variant {
			NodeType::StmtAssign(name) => name,
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};
		let var_index = match self.variables.iter().find(|(str, _)| str == &name) {
			Some(var) => var.1,
			None => return Err(GeneratorError::VariableNotYetDeclared(name))
		};

		self.generate_expr()?;

		if self.stack_size - var_index > 0 {
			self.pop("rax");

			self.output += &format!(
				r"mov QWORD [rsp + {}], rax{}",
				(self.stack_size - var_index) * 8,
				"\n"
			);
		} 
		
		Ok(())
	}

	fn generate_expr(&mut self) -> Result<(), GeneratorError> {
		loop {
			let node = match self.input.peek() {
				Some(node) => node,
				None => break
			};
			match &node.variant {
				NodeType::ExprIdent(name) => {
					let var_index = match self.variables.iter().find(|(str, _)| str == name) {
						Some(var) => var.1,
						None => return Err(GeneratorError::VariableNotYetDeclared(name.clone()))
					};
					self.push(&format!(r"QWORD [rsp + {}]", (self.stack_size - var_index) * 8));
				},
				NodeType::ExprLiteral(num) => {
					self.output += &format!("mov rax, {}\n", num);
					self.push("rax");
				},
				NodeType::ExprParen => {
					self.push("rax");
				}
				NodeType::ExprBinAdd => self.generate_bin_expr()?,
				NodeType::ExprBinSub => self.generate_bin_expr()?,
				NodeType::ExprBinMul => self.generate_bin_expr()?,
				NodeType::ExprBinDiv => self.generate_bin_expr()?,
				_ => break
			};	
			self.input.next();
		}

		Ok(())
	}

	fn generate_bin_expr(&mut self) -> Result<(), GeneratorError> {
		let node_type = self.input.peek().ok_or(GeneratorError::EndOfInput)?.variant.clone();
		
		self.pop("rbx");
		
		self.pop("rax");

		self.output += match node_type {
			NodeType::ExprBinAdd => "add rax, rbx\n",
			NodeType::ExprBinSub => "sub rax, rbx\n",
			NodeType::ExprBinMul => "mul rbx\n",
			NodeType::ExprBinDiv => "div rbx\n",
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};

		self.push("rax");

		Ok(())
	}

	fn generate_function(&mut self) -> Result<(), GeneratorError> {
		let name = match self.input.next().ok_or(GeneratorError::EndOfInput)?.variant {
			NodeType::StmtFunction(name) => name,
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};

		self.generate_expr()?;

		if name == "exit" {
			self.pop("rdi");
			self.output += "mov rax, 60\n";
			self.output += "syscall\n";
		}

		Ok(())
	}

	fn generate_conditional(&mut self) -> Result<(), GeneratorError> {
		let paths = match self.input.next().ok_or(GeneratorError::EndOfInput)?.variant {
			NodeType::StmtIf(paths) => paths,
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};
		let label = self.create_label("if");

		self.generate_expr()?;
		self.pop("rax");
		
		self.output += "test rax, rax\n";
		self.output += &format!("jz {}\n", label);

		self.generate_closure()?;
		
		// No else
		if paths == 0 {
			self.output += &format!("{}:\n", label);
			return Ok(())
		}
		let label_else = self.create_label("else");
		
		self.output += &format!("jmp {}\n", label_else);
		self.output += &format!("{}:\n", label);

		self.generate_closure()?;

		self.output += &format!("{}:\n", label_else);

		Ok(())
	}

	/// Generates a loop from nodes
	/// 
	/// Unusual format
	/// 
	/// - while
	/// - expr
	/// - closure
	/// 
	fn generate_loop(&mut self) -> Result<(), GeneratorError> {
		match self.input.next().ok_or(GeneratorError::EndOfInput)?.variant {
			NodeType::StmtWhile => (),
			node_type => return Err(GeneratorError::UnexpectedNode(node_type))
		};

		let start = self.create_label("loopstart");
		let end = self.create_label("loopend");

		self.output += &format!("{}:\n", start);
		self.generate_expr()?;

		self.pop("rax");
		self.output += "test rax, rax\n";
		self.output += &format!("jz {}\n", end);

		self.generate_closure()?;

		self.output += &format!("jmp {}\n", start);
		self.output += &format!("{}:\n", end);
		
		Ok(())
	}
}