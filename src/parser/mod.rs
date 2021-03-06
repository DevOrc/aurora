
pub mod scanner;
pub mod expr;

use std::collections::VecDeque;
use super::{Token, BinOp, Stmt, StmtType, Expr, Keyword};
use super::error::LuaError;

pub struct Parser{
    tokens: VecDeque<Token>,
    pub line: usize
}

impl Parser{

    pub fn new(tokens: Vec<Token>) -> Parser{
        let mut tokens_deque: VecDeque<Token> = VecDeque::new();

        for token in tokens{
            tokens_deque.push_back(token);
        }

        Parser {tokens: tokens_deque, line: 1}
    }
    
    pub fn parse(&mut self) -> Result<Vec<Stmt>, LuaError>{
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
        let location = format!("Line {}", self.line);

        if token == None{
            return Ok(Stmt {location, stmt_type: StmtType::EOF});
        }

        let token = token.unwrap();

        match token{
            Token::Identifier(_) => self.handle_indentifier(token),
            Token::Keyword(Keyword::Local) => self.handle_local(),
            Token::Keyword(Keyword::If) => self.handle_if_stmt(),
            Token::Keyword(Keyword::Function) => self.handle_func_dec(),
            Token::Keyword(Keyword::Return) => self.handle_return_stmt(),
            Token::Keyword(Keyword::While) => self.handle_while_stmt(),
            Token::Keyword(Keyword::For) => self.handle_for_stmt(),
            Token::LeftParenthesis | Token::RightParenthesis | Token::StringLiteral(_) | 
            Token::Operator(_) | Token::NumberLiteral(_) | Token::Comma | Token::Keyword(_) |
            Token::LeftBrace | Token::RightBrace | Token::Equal =>{ 
                error(format!("Stmt's cannot start with {:?}", token), self.line)
            },
            Token::Semicolon | Token::Newline => self.scan_stmt(),
            Token::EOF => return Ok(Stmt {location, stmt_type : StmtType::EOF}),
        }
    }

    fn handle_return_stmt(&mut self) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);
        let value_tokens = self.advance_to_mult(vec![Token::Newline, Token::Semicolon]);

        match expr::parse(value_tokens, self.line){
            Ok(expr) => Ok(Stmt{location, stmt_type: StmtType::Return(expr)}),
            Err(e) => Err(e)
        }
    }

    fn handle_for_stmt(&mut self) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);

        let var_name = if let Some(x) = self.next_token(){
            x
        }else{
            return error("Expected identifier but found none!".to_string(), self.line);
        };

        self.next_token(); // Remove equal sign

        let start_expr = expr::parse(self.advance_to(Token::Comma), self.line)?;
        let end_expr = expr::parse(self.advance_to(Token::Comma), self.line)?;
        let increment_expr = expr::parse(self.advance_to(Token::Keyword(Keyword::Do)), self.line)?;
        let block_tokens = self.advance_to_block_end();
        let block = parse(block_tokens)?;

        Ok(Stmt{location, stmt_type : StmtType::For(var_name, start_expr, end_expr, increment_expr, block)})
    }

    fn handle_while_stmt(&mut self) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);
        let expr_tokens = self.advance_to(Token::Keyword(Keyword::Do));
        let expr = expr::parse(expr_tokens, self.line)?;
        let block_tokens = self.advance_to_block_end();
        let block = parse(block_tokens)?;

        Ok(Stmt{location, stmt_type : StmtType::While(expr, block)})
    }

    fn handle_if_stmt(&mut self) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);
        let expr_tokens = self.advance_to(Token::Keyword(Keyword::Then));
        let expr = expr::parse(expr_tokens, self.line)?;
        let (block_tokens, block_end) = self.advance_to_if_end();
        let block = parse(block_tokens)?;

        if block_end == Some(Keyword::Else) {
            let else_block_tokens = self.advance_to(Token::Keyword(Keyword::End));

            return Ok(Stmt {location, stmt_type : StmtType::If(expr, block, Some(parse(else_block_tokens)?))})
        }

        Ok(Stmt{location, stmt_type : StmtType::If(expr, block, None)})
    }

    fn advance_to_if_end(&mut self) -> (Vec<Token>, Option<Keyword>){
        let mut tokens = Vec::new();
        let stop_keywords: Vec<Keyword> = vec![Keyword::End, Keyword::Else];

        loop{
            let token = self.next_token();

            if let Some(token) = token{
                match token{
                    Token::Keyword(ref k) if stop_keywords.contains(k) => return (tokens, Some(k.clone())), 
                    _ => (), 
                }

                tokens.push(token);
            }else{
                break;
            }
        }

        (tokens, None)
    }


    fn handle_func_dec(&mut self) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);
        let name = match self.next_token(){
            Some(x) => x,
            None => return error(format!("Expected to find function name but found None"), self.line),
        };

        //Remove left parenthesis
        match self.next_token(){
            Some(Token::LeftParenthesis) => (),
            x => return error(format!("Expected left parenthesis but found {:?}", x), self.line),
        }

        let mut args = self.advance_to(Token::RightParenthesis);
        args.retain(|t| t != &Token::Comma);

        let block_tokens = self.advance_to_block_end();
        let block = parse(block_tokens)?;

        //Remove 'End'
        self.next_token();

        Ok(Stmt{location, stmt_type : StmtType::FunctionDef(name, args, block)})
    }

    fn advance_to_block_end(&mut self) -> Vec<Token>{
        let mut tokens = Vec::new();
        let mut level = 0;

        loop{
            let token = self.next_token();

            if let Some(token) = token{
                match token{
                    Token::Keyword(Keyword::If) | Token::Keyword(Keyword::While) => level+=1,
                    Token::Keyword(Keyword::End) => {
                        if level == 0{
                            break;
                        }

                        level -= 1;
                    } 
                    _ => (), 
                }

                tokens.push(token);
            }else{
                break;
            }
        }

        tokens
    }

    fn handle_indentifier(&mut self, token: Token) -> Result<Stmt, LuaError>{
        let following_token = self.next_token();
        let location = format!("Line {}", self.line);

        if let Some(following_token) = following_token{
            match following_token{
                Token::LeftParenthesis =>{
                    let args = self.advance_to_args_end();
                    let stmt_type = StmtType::FunctionCall(token, self.parse_args(args)?);

                    Ok(Stmt {location, stmt_type})
                },
                Token::Equal =>{
                  Ok(self.scan_assignment(token, false)?)
                },
                _ => error(format!("Unknown token following identifier: {:?}", token), self.line),
            }
        }else{
            error(format!("Files cannot end with identifiers!"), self.line)
        }
    }

    fn advance_to_args_end(&mut self) -> Vec<Token>{
        let mut tokens = Vec::new();
        let mut level = 0;

        loop{
            let token = self.next_token();

            if let Some(token) = token{
                if token == Token::LeftParenthesis{
                    level += 1;
                }else if token == Token::RightParenthesis{
                    if level == 0{
                        break;
                    }
                    level -= 1;
                }

                tokens.push(token);
            }else{
                break;
            }
        }

        tokens
    }

    fn handle_local(&mut self) -> Result<Stmt, LuaError>{
        let name = match self.next_token(){
            Some(x) => x,
            None => return error(format!("Expected token following keyword local, but found None!"), self.line)
        };

        let equal_token = match self.next_token(){
            Some(x) => x,
            None => return error(format!("Expected token '=' but found None!"), self.line)
        };

        if equal_token != Token::Equal{
            return error(format!("Expected token '=' but found, {:?}", equal_token), self.line);
        }

        self.scan_assignment(name, true)
    }

    fn scan_assignment(&mut self, name: Token, is_local: bool) -> Result<Stmt, LuaError>{
        let location = format!("Line {}", self.line);
        let tokens = self.advance_to_mult(vec![Token::Newline, Token::Semicolon]);
        let expr = expr::parse(tokens, self.line)?;
        let stmt_type = StmtType::Assignment(name, expr, is_local);

        Ok(Stmt {location, stmt_type})
    }

    fn parse_args(&self, args: Vec<Token>) -> Result<Vec<Expr>, LuaError>{
        let mut exprs= Vec::new();
        let mut tokens = Vec::new();

        for token in args{
            if token == Token::Comma{
                let expr = expr::parse(tokens.clone(), self.line)?;
                exprs.push(expr);
                tokens.clear();
            }else{
                tokens.push(token);
            }
        }

        //Parse the last argument
        if tokens.len() > 0{
            let expr = expr::parse(tokens, self.line)?;
            exprs.push(expr);
        }

        Ok(exprs)
    }

    fn advance_to(&mut self, stop: Token) -> Vec<Token>{
        self.advance_to_mult(vec![stop])
    }

    fn advance_to_mult(&mut self, stop: Vec<Token>)-> Vec<Token>{
        let mut tokens = Vec::new();

        loop{
            let token = self.next_token();

            if let Some(token) = token{
                if stop.contains(&token) || token == Token::EOF{
                    break;
                }

                tokens.push(token);
            }else{
                break;
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token>{
        let token = self.tokens.pop_front();

        if token ==Some(Token::Newline){
            self.line += 1;
        }

        token
    }
}

fn error(message: String, line: usize) -> Result<Stmt, LuaError>{
    Err(LuaError::create_parse(&message, Some(format!("Line {}", line))))
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, LuaError>{
    let mut parser = Parser::new(tokens);

    parser.parse()
}