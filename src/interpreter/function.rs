
use std::collections::HashMap;
use super::{Interpreter};
use super::super::{Token, Stmt, StmtType, data::LuaData};

type RustFunc = fn(Vec<LuaData>, &Interpreter) -> Option<LuaData>;

#[derive(Clone)]
pub struct LuaFunc{
    pub arg_defs: Vec<Token>,
    pub stmts: Vec<Stmt>,
}

impl LuaFunc{

    pub fn new(arg_defs: Vec<Token>, stmts: Vec<Stmt>) -> LuaFunc{
        LuaFunc{arg_defs: arg_defs, stmts: stmts}
    }

    pub fn execute(&mut self, arg_data: Vec<LuaData>, interpreter: &mut Interpreter) -> Option<LuaData>{
        if self.arg_defs.len() != arg_data.len(){
            panic!("Incorrect number of arguments found! Expected {} but found {}", self.arg_defs.len(), arg_data.len());
        }

        self.add_args(interpreter, arg_data);

        let mut return_value = None;

        for stmt in &self.stmts{
            match stmt.stmt_type{
                StmtType::Return(ref expr) => {
                    return_value = Some(interpreter.evaluate_expr(expr));
                    break;
                }
                _ => interpreter.run_stmt(&mut stmt.clone()),
            }
        }

        //The intepreter will automatically pop the stack and remove the arguments
        return_value
    }

    fn add_args(&self, interpreter: &mut Interpreter, data: Vec<LuaData>){
        for x in 0..self.arg_defs.len(){
            let name = match self.arg_defs.get(x).unwrap(){
                Token::Identifier(x) => x,
                x => panic!("Expected identifier but found {:?}", x),    
            }.to_string();

            interpreter.assign_variable(name, data.get(x).unwrap().clone(), true)
        }
    }
}

#[derive(Clone)]
pub enum Function{
    Lua(LuaFunc),
    Rust(RustFunc)
}