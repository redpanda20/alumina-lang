
/* Minimal program
global _start
_start
	mov rdi, 0
	mov rax, 60
	syscall
*/

use std::{collections::HashMap, iter::Peekable};

use crate::parser::{ChildNode, NodeType};

#[derive(Debug)]
pub enum GeneratorError {
	EndOfInput,
	VariableAlreadyDeclared,
	VariableNotYetDeclared,
	UnexpectedNode
}

pub struct Generator<I: Iterator<Item = ChildNode>> {
	input: Peekable<I>,
	variables: HashMap<String, usize>,
	stack_size: usize,
	output: String
}

impl <I: Iterator<Item = ChildNode>> Generator<I> {
	pub fn generate_program(iterator: I) -> Result<String, GeneratorError> {

		let input = iterator.peekable();

		let mut generator = Generator {
			input,
			variables: HashMap::new(),
			stack_size: 0,
			output: String::new(),
		};

		loop {
			match generator.generate_node() {
				Ok(_) => (),
				Err(GeneratorError::EndOfInput) => break,
				Err(err) => return Err(err),
			}
		}

		generator.output = String::from("global _start\n_start:\n") + &generator.output + "mov rdi, 0\nmov rax, 60\nsyscall";
		
		// generator.opt();

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
				let offset = format!("QWORD [rsp + {}]", (self.stack_size - num) * 8);
				self.push(&offset);
			},
			NodeType::ExprLiteral(num) => {
				self.output += &format!("mov rax, {}\n", num);
				self.push("rax");
			},
			NodeType::ExprBinAdd => self.generate_bin_expr(node)?,
			NodeType::ExprBinSub => self.generate_bin_expr(node)?,
			NodeType::ExprBinMul => self.generate_bin_expr(node)?,
			NodeType::ExprBinDiv => self.generate_bin_expr(node)?,
		}
		Ok(())
	}

	fn generate_bin_expr(&mut self, node: ChildNode) -> Result<(), GeneratorError> {

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

	fn generate_function(&mut self, node: ChildNode) -> Result<(), GeneratorError> {
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