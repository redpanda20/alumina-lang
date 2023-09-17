
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
	VariableAlreadyDeclared,
	VariableNotYetDeclared,
	ClosureNotYetOpened,
	UnexpectedNode
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
		let label = format!("{}{}", name, self.label_count);
		self.label_count += 1;
		label
	}

	fn generate_node(&mut self) -> Result<(), GeneratorError> {
		let Some(node) = self.input.next() else {
			return Err(GeneratorError::EndOfInput)
		};
		match &node.variant {
			NodeType::ClosureStart => {
				self.scopes.push(self.variables.len())
			},
			NodeType::ClosureEnd => {
				let closure_start = self.scopes.pop()
					.ok_or(GeneratorError::ClosureNotYetOpened)?;
				let pop_count = self.variables.len() - closure_start;

				self.output += &format!("add rsp, {}\n", pop_count * 8);
				self.stack_size -= pop_count;
				self.variables.truncate(closure_start);	
			},

			NodeType::StmtFunction(_) => self.generate_function(node)?,

			NodeType::StmtIf => self.generate_conditional(node)?,
			
			NodeType::StmtAssign(name) => {
				if self.variables.iter().any(|(str, _)| str == name) {
					return Err(GeneratorError::VariableAlreadyDeclared);
				}
				self.variables.push((name.to_owned(), self.stack_size));
				self.output.pop();
				self.output += &format!("	; variable ({}) assigned\n", name);
			},
			NodeType::StmtReassign(name) => {
				self.pop("rax");
				let num = match self.variables.iter().find(|(str, _)| str == name) {
					Some(var) => var.1,
					None => return Err(GeneratorError::VariableNotYetDeclared)
				};
				self.output += &format!(
					r"mov QWORD [rsp + {}], rax{}",
					(self.stack_size - num) * 8,
					"\n"
				);
			},
			NodeType::ExprIdent(name) => {
				let Some((_, num)) = self.variables.iter().find(|(str, _)| str == name) else {
					return Err(GeneratorError::VariableNotYetDeclared)
				};
				self.push(&format!(r"QWORD [rsp + {}]", (self.stack_size - num) * 8));
			},
			NodeType::ExprLiteral(num) => {
				self.output += &format!("mov rax, {}\n", num);
				self.push("rax");
			},
			NodeType::ExprParen => {
				self.push("rax");
			}
			NodeType::ExprBinAdd => self.generate_bin_expr(node)?,
			NodeType::ExprBinSub => self.generate_bin_expr(node)?,
			NodeType::ExprBinMul => self.generate_bin_expr(node)?,
			NodeType::ExprBinDiv => self.generate_bin_expr(node)?,
		}
		Ok(())
	}

	fn generate_bin_expr(&mut self, node: Node) -> Result<(), GeneratorError> {

		self.pop("rbx");
		
		self.pop("rax");

		self.output += match node.variant {
			NodeType::ExprBinAdd => "add rax, rbx\n",
			NodeType::ExprBinSub => "sub rax, rbx\n",
			NodeType::ExprBinMul => "mul rbx\n",
			NodeType::ExprBinDiv => "div rbx\n",
			_ => unreachable!()
		};

		self.push("rax");

		Ok(())
	}

	fn generate_function(&mut self, node: Node) -> Result<(), GeneratorError> {
		let NodeType::StmtFunction(name) = node.variant else {
			return Err(GeneratorError::UnexpectedNode)
		};

		if name == "exit" {
			self.pop("rdi");
			self.output += "mov rax, 60\n";
			self.output += "syscall\n";
		}

		Ok(())
	}

	fn generate_conditional(&mut self, node: Node) -> Result<(), GeneratorError> {
		let label = self.create_label("if");
		let NodeType::StmtIf = node.variant else {
			return Err(GeneratorError::UnexpectedNode)
		};

		self.pop("rax");
		self.output += "test rax, rax\n";
		self.output += &format!("jz {}\n", label);

		while let Some(node) = self.input.peek() {
			if let NodeType::ClosureEnd = node.variant {
				break;
			}
			self.generate_node()?
		}

		// Generate closure end
		self.generate_node()?;

		self.output += &format!("{}:\n", label);

		Ok(())
	}
}