
/* Minimal program
global _start
_start
	mov rdi, 0
	mov rax, 60
	syscall
*/

use std::{collections::HashMap, iter::Peekable};

use crate::parser::{Node, NodeType};

#[derive(Debug)]
pub enum GeneratorError {
	EndOfInput,
	VariableAlreadyDeclared,
	VariableNotYetDeclared,
	UnexpectedNode
}

pub struct Generator<I: Iterator<Item = Node>> {
	input: Peekable<I>,
	variables: HashMap<String, usize>,
	stack_size: usize,
	output: String
}

impl <I: Iterator<Item = Node>> Generator<I> {
	pub fn generate_program(iterator: I) -> Result<String, GeneratorError> {

		let input = iterator.peekable();

		let mut generator = Generator {
			input,
			variables: HashMap::new(),
			stack_size: 0,
			output: String::from("global _start\n_start:\n"),
		};

		loop {
			match generator.generate_node() {
				Ok(_) => (),
				Err(GeneratorError::EndOfInput) => break,
				Err(err) => return Err(err),
			}
		}

		generator.opt();

		generator.output += "mov rdi, 0\nmov rax, 60\nsyscall";
		Ok(generator.output)
	}

	fn opt(&mut self) {
		// Immediate variable usage
		self.output = self.output.replace(
			"push rax\nQWORD [rsp + 0]\n",
			"");

		// Direct move
		self.output = self.output.replace(
			"push rax\npop rbx\n",
			"move rbx, rax\n");		
		self.output = self.output.replace(
			"push rax\npop rdi\n",
			"move rdi, rax\n");	
}

	fn push(&mut self, reg: &str) {
		self.output += &format!("push {}\n", reg);
		self.stack_size += 1;
	}

	fn pop(&mut self, reg: &str) {
		self.output += &format!("pop {}\n", reg);
		self.stack_size -= 1;
	}

	fn generate_node(&mut self) -> Result<(), GeneratorError> {
		let Some(node) = self.input.next() else {
			return Err(GeneratorError::EndOfInput)
		};
		match &node.variant {
			NodeType::StmtFunction(_) => self.generate_function(node)?,
			
			NodeType::StmtAssign(name) => {
				if self.variables.contains_key(name) {
					return Err(GeneratorError::VariableAlreadyDeclared);
				}
				self.variables.insert(name.to_owned(), self.stack_size);
			},
			NodeType::ExprIdent(name) => {
				let Some(num) = self.variables.get(name) else {
					return Err(GeneratorError::VariableNotYetDeclared)
				};
				self.output += &format!("QWORD [rsp + {}]\n", (self.stack_size - num) * 8);
				self.push("rax");
			},
			NodeType::ExprLiteral(num) => {
				self.output += &format!("mov rax, {}\n", num);
				self.push("rax");
			},
			NodeType::ExprBinAdd => {
				self.pop("rbx");
				self.pop("rax");
				self.output += "add rax, rbx\n";
				self.push("rax");
			},
			NodeType::ExprBinSub => {
				self.pop("rbx");
				self.pop("rax");
				self.output += "sub rax, rbx\n";
				self.push("rax");
			},
			NodeType::ExprBinMul => {
				self.pop("rbx");
				self.pop("rax");
				self.output += "mul rbx\n";
				self.push("rax");
			},
			NodeType::ExprBinDiv => {
				self.pop("rbx");
				self.pop("rax");
				self.output += "div rbx\n";
				self.push("rax");
			},
		}
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
}