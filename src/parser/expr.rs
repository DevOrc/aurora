
use std::collections::VecDeque;

use super::{Token, BinOp, Stmt, StmtType, Expr, error};
use super::super::{ExprType, error::LuaError};

struct ExprParser{
    expr_type: ExprType,
    tokens: VecDeque<Token>,
}

impl ExprParser{

    fn new(tokens: Vec<Token>) -> ExprParser{
        let mut tokens_deque: VecDeque<Token> = VecDeque::new();
        let mut expr_type = ExprType::SingleValue;

        for token in tokens{
            tokens_deque.push_back(token);
        }

        for token in &tokens_deque{

            // If the expression has a operator it have more than value 
            let token_type = match token{
                Token::Operator(BinOp::Concat) => Some(ExprType::Str),
                Token::Operator(BinOp::LessThan) | Token::Operator(BinOp::LessEqualThan) |
                Token::Operator(BinOp::GreaterThan) | Token::Operator(BinOp::GreaterEqualThan) => Some(ExprType::Bool),
                Token::Operator(_) => Some(ExprType::Number),
                _ => None,
            };

            if let Some(token_type) = token_type{
                expr_type = token_type;
                break;
            }
        }

        ExprParser {tokens: tokens_deque, expr_type: expr_type}
    }
    
    fn parse(mut self) -> Result<Vec<Stmt>, LuaError>{
        let mut stmts: Vec<Stmt> = Vec::new();

        loop{
            let stmt = self.scan_stmt()?;

            if stmt.stmt_type == StmtType::EOF{
                stmts.push(stmt);
                break;
            }

            stmts.push(stmt);
        }

        Ok(stmts)
    }

    fn scan_stmt(&mut self) -> Result<Stmt, LuaError>{
        let token = self.next_token();

        if token == None{
            return Ok(Stmt {stmt_type: StmtType::EOF});
        }

        let token = token.unwrap();
        
        match self.expr_type{
            ExprType::Number | ExprType::Bool => self.scan_num_expr(token),
            ExprType::Str => self.scan_string_expr(token),
            ExprType::SingleValue => self.scan_value(token),
        }
    }

    fn scan_value(&mut self, token: Token) -> Result<Stmt, LuaError>{
        let mut tokens = vec![token];

        loop{
            let next_token = self.next_token();

            if let Some(token) = next_token{

                if token == Token::EOF || token == Token::Newline{
                    tokens.push(token);
                    break;
                }

                tokens.push(token);
            }else{
                break;
            }
        }

        Ok(Stmt{stmt_type: StmtType::Value(tokens)})
    }

    fn scan_num_expr(&mut self, left: Token) -> Result<Stmt, LuaError>{
        let operator = match self.next_token(){
            Some(Token::Operator(operator)) => operator,
            x => return error(format!("Expected binary operator but found {:?}", x)),
        };

        let right = match self.next_token(){
            Some(x) => x,
            _ => return error(format!("Expected token but found EOF")),
        };

        Ok(Stmt{stmt_type: StmtType::BinOp(operator, left, right)})
    }

    fn scan_string_expr(&mut self, left: Token) -> Result<Stmt, LuaError>{
        let operator = match self.next_token(){
            Some(Token::Operator(BinOp::Concat)) => BinOp::Concat,
            x => return error(format!("Expected binary operator but found {:?}", x)),
        };

        let right = match self.next_token(){
            Some(x) => x,
            _ => return error(format!("Expected token but found EOF")),
        };

        Ok(Stmt{stmt_type: StmtType::BinOp(operator, left, right)})
    }

    fn next_token(&mut self) -> Option<Token>{
        self.tokens.pop_front()
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, LuaError>{
    let parser = ExprParser::new(tokens);
    let expr_type = parser.expr_type.clone();
    
    Ok(Expr{expr_type: expr_type, stmts: parser.parse()?})
}