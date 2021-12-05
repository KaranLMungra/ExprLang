#![allow(dead_code)]

use std::process::exit;

/*
 * 2 3 + y =
 * Parsing into tokens and syntax check for each token Literal(2) => "2",  "3" "+" "y" "="
 * Determine the type of statement based on Keyword or Symbol e.g. 2 3 + y = no keyword therefore a
 * expr statement, e.g. Def Sq as x => x x * then from `Def` keyword its is a function definition
 * statement.
 * Then use the particular stack engine to evaluate or make the function.
 * */

/// `Tokens` enum consists of all possible types of tokens in the _ExprLang_.
#[derive(Clone, Debug)]
enum Tokens {
    Literal(Literal),
    Variable(Variable),
    Proc(Proc),
    Keyword,
    Symbol(String),
}

// Defining all different types of tokens

#[derive(Clone, Debug)]
struct Literal {
    value: usize,
}

#[derive(Clone, Debug)]
struct Variable {
    name: String,
    value: Option<usize>,
}

#[derive(Clone, Debug)]
struct Proc {
    name: String,
    param: Vec<Parameter>,
    expr: Vec<Tokens>,
}

#[derive(Clone, Debug)]
struct Parameter {
    name: String,
    value: Option<usize>,
}
const KEYWORDS: [&str; 6] = ["Def", "As", "Exit", "If", "Then", "Else"];

const SYMBOLS: [&str; 16] = [
    "+", "-", "*", "/", "%", "=", "=>", "==", "!=", ">", "<", "<=", "=>", "&&", "||", "!",
];
#[derive(Debug)]
pub enum SyntaxParseError {
    InvalidLiteral,
    InvalidVariableName,
    InvalidProcName,
    ProcNameAlreadyExists,
    InvalidProcDefStatement,
    EmptyLine,
    ProcNotFound,
    InvalidExprStatement,
    InvalidNumberOfArguments,
    VarNameAlreadyExists,
}

#[derive(Clone, Debug)]
struct Engine {
    literal_stack: Vec<Literal>,
    variable_stack: Vec<Variable>,
    proc_stack: Vec<Proc>,
    global_stack: Vec<Tokens>,
    expr_storage: Vec<Tokens>,
}

// Implementation for syntax checking for all tokens.

pub struct ExprLang {
    engine: Engine,
}

impl ExprLang {
    pub fn new() -> Self {
        Self {
            engine: Engine {
                literal_stack: Vec::new(),
                variable_stack: Vec::new(),
                proc_stack: Vec::new(),
                global_stack: Vec::new(),
                expr_storage: Vec::new(),
            },
        }
    }

    pub fn parse_syntax_stack(&mut self, statement: &str) -> Result<(), SyntaxParseError> {
        self.engine.syntax_parse(statement)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.engine.global_stack.clear();
        self.engine.literal_stack.clear();
        self.engine.expr_storage.clear();
    }

    pub fn eval(&mut self) -> Result<usize, SyntaxParseError> {
        self.engine.run()
    }
}

impl Engine {
    pub fn syntax_parse(&mut self, statement: &str) -> Result<(), SyntaxParseError> {
        let mut statement: Vec<&str> = statement
            .trim()
            .split(' ')
            .map(|token| token.trim())
            .collect();
        if statement[0] == "" {
            return Ok(());
        }
        // Determining the type of statement by matching the first token
        if statement[0] == KEYWORDS[0] {
            self.syntax_parse_proc(&mut statement)?;
        } else if statement[0] == "Exit" {
            exit(0);
        } else if statement[0] == "If" {
            self.syntax_parse_if_cond(&mut statement)?;
        } else if Literal::is_literal(&statement[0]) {
            self.syntax_parse_expr(&mut statement)?;
        } else if Variable::is_valid_var_name(&statement[0]) {
            self.syntax_parse_expr(&mut statement)?;
        } else {
            return Err(SyntaxParseError::InvalidExprStatement);
        }
        //        println!("{:#?}", self);
        Ok(())
    }

    fn syntax_parse_if_cond(&mut self, statement: &mut Vec<&str>) -> Result<(), SyntaxParseError> {
        statement.reverse();
        statement.pop();
        let mut cond = Vec::new();
        while let Some(token) = statement.pop() {
            if token == "Then" {
                break;
            }
            cond.push(token);
        }
        self.syntax_parse_expr(&mut cond)?;
        let mut if_expr = Vec::new();
        while let Some(token) = statement.pop() {
            if token == "Else" {
                break;
            }
            if_expr.push(token);
        }

        let mut else_expr = Vec::new();
        while let Some(token) = statement.pop() {
            else_expr.push(token);
        }
        println!(
            "Cond: {:?}, If_expr: {:?}, Else_epxr: {:?}",
            cond, if_expr, else_expr
        );
        let res = self.run()?;
        println!("Res: {}", res);
        if res > 0 {
            self.syntax_parse(&if_expr.join(" "))?;
        } else {
            self.syntax_parse(&else_expr.join(" "))?;
        }
        Ok(())
    }

    fn syntax_parse_proc(&mut self, mut statement: &mut Vec<&str>) -> Result<(), SyntaxParseError> {
        // Def Sq as x => x x *
        statement.reverse();
        // * x x => x as Sq
        statement.pop();
        // checking if proc name is valid or not
        let proc_name = if let Some(name) = statement.pop() {
            name
        } else {
            return Err(SyntaxParseError::InvalidProcDefStatement);
        };
        if !Proc::is_valid_proc_name(proc_name) {
            return Err(SyntaxParseError::InvalidProcName);
        }
        // checking if the proc name is already a define proc or variable
        if self.proc_stack.iter().any(|proc| &proc.name == proc_name)
            || self.variable_stack.iter().any(|var| &var.name == proc_name)
        {
            return Err(SyntaxParseError::ProcNameAlreadyExists);
        }
        // Done checking with proc_name all checks out. Now next token should be the keyword `As`
        if let Some(keyword) = statement.pop() {
            if !(keyword == KEYWORDS[1]) {
                return Err(SyntaxParseError::InvalidProcDefStatement);
            }
        } else {
            return Err(SyntaxParseError::InvalidProcDefStatement);
        }
        // Done checking for keyword `As` Now checking each argument variable until we found EOL in
        // which case it will be a error or found the symbol `=>` in which case it we will parse
        // args list.
        let mut args = Vec::new();
        while let Some(token) = statement.pop() {
            // if token is symbol `=>` then we completed arg list and can break for the body
            if token == SYMBOLS[6] {
                break;
            }
            // checking if its a valid variable name;
            if !Variable::is_valid_var_name(&token) {
                return Err(SyntaxParseError::InvalidVariableName);
            }
            // We are not gonna check for if the name already exists as some Item Name but instead
            // just assume it in a local scope and gonna treat as such.
            let token = token.to_owned();
            args.push(Parameter {
                name: token,
                value: None,
            });
        }

        // checking if the statement is empty hence a error not found valid body or symbol `=>`
        if statement.is_empty() {
            return Err(SyntaxParseError::InvalidProcDefStatement);
        }

        let proc_body = self.parse_proc_body(&mut statement)?;

        let proc = Proc {
            name: proc_name.to_owned(),
            param: args,
            expr: proc_body,
        };
        //println!("{:#?}", proc);
        self.proc_stack.push(proc);
        Ok(())
    }

    fn syntax_parse_expr(&mut self, statement: &mut Vec<&str>) -> Result<(), SyntaxParseError> {
        let mut stack = Vec::with_capacity(256);
        // 2 3 + y =
        'outer: while let Some(token_str) = statement.pop() {
            if SYMBOLS.contains(&token_str) && token_str != "=>" {
                stack.push(Tokens::Symbol(token_str.to_owned()));
                continue;
            }
            //e//println!("Symbol Checked!");
            if Literal::is_literal(&token_str) {
                if let Ok(literal) = token_str.parse() {
                    stack.push(Tokens::Literal(Literal { value: literal }));
                    continue;
                } else {
                    return Err(SyntaxParseError::InvalidLiteral);
                }
            }
            //e//println!("Literal Checked!");
            if Variable::is_valid_var_name(&token_str)
                && !self.proc_stack.iter().any(|proc| &proc.name == token_str)
            {
                let var_name = token_str.to_owned();
                for var in self.variable_stack.iter() {
                    if var.name == var_name {
                        stack.push(Tokens::Variable(var.clone()));
                        continue 'outer;
                    }
                }
                stack.push(Tokens::Variable(Variable {
                    name: var_name,
                    value: None,
                }));
                continue;
            }
            //e//println!("Variable Checked!");
            if Proc::is_valid_proc_name(&token_str)
                && self.proc_stack.iter().any(|proc| &proc.name == token_str)
            {
                for proc in self.proc_stack.iter() {
                    if &proc.name == token_str {
                        stack.push(Tokens::Proc(proc.clone()));
                        continue 'outer;
                    }
                }
            } else {
                return Err(SyntaxParseError::ProcNotFound);
            }
            //e//println!("Proc Checked!");
            return Err(SyntaxParseError::InvalidExprStatement);
        }
        self.global_stack = stack;
        Ok(())
    }

    fn parse_proc_body(&self, statement: &mut Vec<&str>) -> Result<Vec<Tokens>, SyntaxParseError> {
        let mut stack = Vec::with_capacity(256);
        // 2 3 + y =
        'outer: while let Some(token_str) = statement.pop() {
            if SYMBOLS.contains(&token_str) && token_str != "=>" {
                stack.push(Tokens::Symbol(token_str.to_owned()));
                continue;
            }
            //e//println!("Symbol Checked!");
            if Literal::is_literal(&token_str) {
                if let Ok(literal) = token_str.parse() {
                    stack.push(Tokens::Literal(Literal { value: literal }));
                    continue;
                } else {
                    return Err(SyntaxParseError::InvalidLiteral);
                }
            }
            //e//println!("Literal Checked!");
            if Variable::is_valid_var_name(&token_str)
                && !self.proc_stack.iter().any(|proc| &proc.name == token_str)
            {
                let var_name = token_str.to_owned();
                for var in self.variable_stack.iter() {
                    if var.name == var_name {
                        stack.push(Tokens::Variable(var.clone()));
                        continue;
                    }
                }
                stack.push(Tokens::Variable(Variable {
                    name: var_name,
                    value: None,
                }));
                continue;
            }
            //e//println!("Variable Checked!");
            if Proc::is_valid_proc_name(&token_str)
                && self.proc_stack.iter().any(|proc| &proc.name == token_str)
            {
                for proc in self.proc_stack.iter() {
                    if &proc.name == token_str {
                        stack.push(Tokens::Proc(proc.clone()));
                        continue 'outer;
                    }
                }
            } else {
                return Err(SyntaxParseError::ProcNotFound);
            }
            //e//println!("Proc Checked!");
            return Err(SyntaxParseError::InvalidExprStatement);
        }
        Ok(stack)
    }

    pub fn run(&mut self) -> Result<usize, SyntaxParseError> {
        let mut result = 0usize;
        let length = self.global_stack.len();
        while let Some(token) = self.global_stack.pop() {
            match token {
                Tokens::Literal(literal) => {
                    if length == 1 {
                        result = literal.value;
                        continue;
                    }
                    self.expr_storage.push(Tokens::Literal(literal));
                }
                Tokens::Symbol(sym) => {
                    result = self.sym_eval(sym)?;
                    self.expr_storage
                        .push(Tokens::Literal(Literal { value: result }));
                }
                Tokens::Variable(var) => {
                    if length == 1 {
                        if var.value.is_none() {
                            return Err(SyntaxParseError::InvalidVariableName);
                        }
                        result = var.value.unwrap();
                    }
                    self.expr_storage.push(Tokens::Variable(var));
                }
                Tokens::Proc(proc) => {
                    if length < 2 {
                        print!("{}: ", proc.name);
                        for i in proc.param.iter() {
                            print!("{} ", i.name);
                        }
                        println!("=> ...");
                        return Ok(0);
                    }
                    result = self.run_proc(proc)?;
                    self.expr_storage
                        .push(Tokens::Literal(Literal { value: result }));
                }

                _ => continue,
            }
        }
        Ok(result)
    }

    fn run_proc(&mut self, mut proc: Proc) -> Result<usize, SyntaxParseError> {
        if self.expr_storage.len() < proc.param.len() {
            return Err(SyntaxParseError::InvalidNumberOfArguments);
        }
        let mut i = 0;
        while i < proc.param.len() {
            let token = self.expr_storage.pop().unwrap();
            proc.param[i].value = match token {
                Tokens::Literal(l) => Some(l.value),
                Tokens::Variable(v) => {
                    let mut val = None;
                    for var in &self.variable_stack {
                        if var.name == v.name {
                            val = var.value;
                        }
                    }
                    if val.is_none() {
                        return Err(SyntaxParseError::InvalidVariableName);
                    }
                    val
                }
                _ => return Err(SyntaxParseError::InvalidExprStatement),
            };
            i += 1;
        }
        //println!("{:?}", proc);
        let mut engine = Engine {
            expr_storage: Vec::new(),
            global_stack: Vec::new(),
            variable_stack: Vec::new(),
            literal_stack: Vec::new(),
            proc_stack: Vec::new(),
        };
        for param in &proc.param {
            let param = Variable {
                name: param.name.clone(),
                value: param.value,
            };
            engine.variable_stack.push(param);
        }
        engine.proc_stack.append(&mut self.proc_stack.clone());
        proc.expr.reverse();
        engine.global_stack.append(&mut proc.expr);
        engine.run()
    }

    fn sym_eval_ari_log_op(&mut self, sym: String) -> Result<usize, SyntaxParseError> {
        if self.expr_storage.len() < 2 {
            return Err(SyntaxParseError::InvalidNumberOfArguments);
        }
        let a = self.expr_storage.pop();
        let b = self.expr_storage.pop();
        let a = match a.unwrap() {
            Tokens::Literal(l) => l.value,
            Tokens::Variable(v) => {
                let mut val = None;
                for var in &self.variable_stack {
                    if var.name == v.name {
                        val = var.value;
                    }
                }
                if val.is_none() {
                    return Err(SyntaxParseError::InvalidVariableName);
                }
                val.unwrap()
            }
            _ => return Err(SyntaxParseError::InvalidExprStatement),
        };
        let b = match b.unwrap() {
            Tokens::Literal(l) => l.value,
            Tokens::Variable(v) => {
                let mut val = None;
                for var in &self.variable_stack {
                    if var.name == v.name {
                        val = var.value;
                    }
                }
                if val.is_none() {
                    return Err(SyntaxParseError::InvalidVariableName);
                }
                val.unwrap()
            }
            _ => return Err(SyntaxParseError::InvalidExprStatement),
        };
        let result = match sym.as_str() {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => a / b,
            "%" => a % b,
            "==" => {
                if b == a {
                    1
                } else {
                    0
                }
            }
            "!=" => {
                if b != a {
                    1
                } else {
                    0
                }
            }
            ">" => {
                if b > a {
                    1
                } else {
                    0
                }
            }
            ">=" => {
                if b >= a {
                    1
                } else {
                    0
                }
            }
            "<" => {
                if b < a {
                    1
                } else {
                    0
                }
            }
            "<=" => {
                if b <= a {
                    1
                } else {
                    0
                }
            }
            "&&" => {
                if a > 0 && b > 0 {
                    1
                } else {
                    0
                }
            }
            "||" => {
                if a == 0 && b == 0 {
                    0
                } else {
                    1
                }
            }
            _ => return Err(SyntaxParseError::InvalidExprStatement),
        };
        Ok(result)
    }
    fn sym_eval(&mut self, sym: String) -> Result<usize, SyntaxParseError> {
        match sym.as_str() {
            "=" => {
                if self.expr_storage.len() < 2 {
                    return Err(SyntaxParseError::InvalidNumberOfArguments);
                }
                let a = self.expr_storage.pop().unwrap();
                let b = self.expr_storage.pop().unwrap();
                let mut var = match a {
                    Tokens::Variable(v) => {
                        for v1 in &self.variable_stack {
                            if v1.name == v.name {
                                return Err(SyntaxParseError::VarNameAlreadyExists);
                            }
                        }
                        v
                    }
                    _ => {
                        return Err(SyntaxParseError::InvalidExprStatement);
                    }
                };
                let b1 = match b {
                    Tokens::Literal(lit) => lit.value,
                    Tokens::Variable(mut var) => {
                        for v in &self.variable_stack {
                            if v.name == var.name {
                                var.value = v.value;
                            }
                        }
                        if var.value.is_none() {
                            return Err(SyntaxParseError::InvalidVariableName);
                        }
                        var.value.unwrap()
                    }
                    _ => {
                        return Err(SyntaxParseError::InvalidExprStatement);
                    }
                };
                var.value = Some(b1);
                self.variable_stack.push(var);
                Ok(b1)
            }
            "!" => {
                if self.expr_storage.len() < 1 {
                    return Err(SyntaxParseError::InvalidNumberOfArguments);
                }
                let a = self.expr_storage.pop().unwrap();
                let a = match a {
                    Tokens::Literal(lit) => lit.value,

                    Tokens::Variable(mut var) => {
                        for v in &self.variable_stack {
                            if v.name == var.name {
                                var.value = v.value;
                            }
                        }
                        if var.value.is_none() {
                            return Err(SyntaxParseError::InvalidVariableName);
                        }
                        var.value.unwrap()
                    }
                    _ => {
                        return Err(SyntaxParseError::InvalidExprStatement);
                    }
                };
                let res = if a == 0 { 1 } else { 0 };
                Ok(res)
            }
            _ => self.sym_eval_ari_log_op(sym),
        }
    }
}

impl Literal {
    pub fn is_literal(token: &str) -> bool {
        !token.as_bytes().iter().any(|c| !c.is_ascii_digit())
    }
}

impl Proc {
    pub fn is_valid_proc_name(token: &str) -> bool {
        token.is_ascii() && !token.is_empty()
    }
}

impl Variable {
    pub fn is_valid_var_name(token: &str) -> bool {
        token.is_ascii() && !token.is_empty()
    }
}
