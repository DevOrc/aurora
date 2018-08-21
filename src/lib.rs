
mod scanner;
mod parser;
mod interpreter;

#[derive(Debug)]
pub enum LuaResult{
    Successful, 
    Failure
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token{
    Identifier(String), 
    StringLiteral(String),
    LeftParenthesis,
    RightParenthesis,
    Newline,
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

#[derive(Debug)]
pub struct Stmt{
    stmt_type: StmtType,
    tokens: Vec<Token>
}

#[derive(Debug, PartialEq)]
pub enum StmtType{
    FunctionCall,
    EOF
}


pub fn run(src: String) -> LuaResult{
    let tokens = scanner::scan(src);
    print_token_info(&tokens);
    println!("");

    let mut stmts = parser::parse(tokens);
    print_stmt_info(&stmts);

    println!("\n---------- Running -------");
    interpreter::run(&mut stmts);
    println!("---------- Finished -------");

    LuaResult::Successful
}

fn print_stmt_info(stmts: &Vec<Stmt>){
    println!("Stmt Count: {}", stmts.len());

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