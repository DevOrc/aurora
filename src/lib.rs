
pub mod parser;
pub mod interpreter;
pub mod data;
pub mod error;

use error::LuaError;

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp{
    Concat,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Equal,
    Plus,
    Minus, 
    Multiply,
    Divide
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword{
    True, False, If, Then, Else, End, Function, Return, Local
}

impl Keyword{

    pub fn vec() -> Vec<String>{
        vec!["true", "false", "if", "else", "then", "end", "function", "return", "local"].iter().map(|x| x.to_string()).collect()
    }

    pub fn is_keyword(string: &str) -> bool{
        if Keyword::vec().contains(&string.to_string()){
            return true;
        }
        
        false
    }

    pub fn from_string(string: &str) -> Keyword{
        match string{
            "true" => Keyword::True,
            "false" => Keyword::False,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "then" => Keyword::Then,
            "end" => Keyword::End,
            "function" => Keyword::Function,
            "return" => Keyword::Return,
            "local" => Keyword::Local,
            _ => panic!("Couldn't convert string to keyword: {}", string),
        }
    }

}

#[derive(Debug, PartialEq, Clone)]
pub enum Token{
    Identifier(String), 
    StringLiteral(String),
    NumberLiteral(f64),
    Operator(BinOp),
    Keyword(Keyword),
    LeftParenthesis,
    RightParenthesis,
    Newline,
    Comma,
    EOF 
}

impl Token{

    pub fn can_be_arg(&self) -> bool{
        match self{
            Token::Identifier(_) | Token::StringLiteral(_) => true,
            _ => false
        }
    }

} 

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt{
    stmt_type: StmtType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtType{
    ///Name, Arguments
    FunctionCall(Token, Vec<Expr>),
    ///Name, Arguments, Stmts
    FunctionDef(Token, Vec<Token>, Vec<Stmt>),
    ///Name, Assignment, Is Local,
    Assignment(Token, Expr, bool),
    ///Operator, Left Token, Right Token
    BinOp(BinOp, Token, Token),
    ///A single token value
    Value(Vec<Token>),
    ///Condition, Stmts, Else
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    Return(Expr),
    EOF
}

impl StmtType{

    fn stmt_count_recursive(&self) -> u32{
        match self{
            StmtType::Return(_) | StmtType::Assignment(_, _, _) | StmtType::BinOp(_, _, _) | 
            StmtType::FunctionCall(_, _) | StmtType::EOF | StmtType::Value(_) => 1,
            StmtType::If(_, block, else_block) => {
                let mut count = 1 + count_stmts_recur(block);

                if let Some(else_block) = else_block{
                    count += count_stmts_recur(else_block);
                }

                count
            },
            StmtType::FunctionDef(_, _, block) => {
                1 + count_stmts_recur(block)
            } 
        }
    }

}

pub fn count_stmts_recur(stmts: &Vec<Stmt>) -> u32{
    let mut count = 0;

    for x in stmts {
        count += x.stmt_type.stmt_count_recursive();
    }

    count
}

#[derive(Debug, PartialEq, Clone)]
enum ExprType{
    Str, Number, Bool, SingleValue
}


#[derive(Debug, PartialEq, Clone)]
pub struct Expr{
    stmts: Vec<Stmt>,
    expr_type: ExprType
}

pub fn run(src: String) -> Result<(), Vec<LuaError>>{
    let tokens = parser::scanner::scan(src)?;
    print_token_info(&tokens);
    println!("");

    let mut stmts = match parser::parse(tokens){
        Ok(x) => x,
        Err(e) => return Err(vec![e])
    };
    print_stmt_info(&stmts);

    println!("\n---------- Running -------");
    interpreter::run(&mut stmts);
    println!("---------- Finished -------");

    Ok(())
}

fn print_stmt_info(stmts: &Vec<Stmt>){
    println!("Stmt Count: {}", count_stmts_recur(stmts));

    for stmt in stmts{
        println!("{:#?}", stmt);
    }
}

fn print_token_info(tokens: &Vec<Token>){
    println!("Token Count: {}", tokens.len());

    for token in tokens{
        println!("{:?}", token);
    }
}