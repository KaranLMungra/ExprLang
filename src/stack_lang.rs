#[derive(Debug)]
pub struct Engine {
    global_stack: GlobalStack,
    variable_stack: VariableStack,
    ans: usize,
}

pub type GlobalStack = Vec<Item>;
pub type VariableStack = Vec<Variable>;

#[derive(Debug, Clone)]
pub enum Item {
    Literal(usize),
    Variable(Variable),
    Function(OperatorFunctions),
}

#[derive(Debug, Clone)]
pub enum OperatorFunctions {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Assignment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: usize,
}

#[derive(Debug)]
pub enum SyntaxParseError {
    NotALiteralError,
    NotAGoodVariableName,
    NotAValidSyntaxItem,
    InvalidFunctionNameFound,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            global_stack: Vec::new(),
            variable_stack: Vec::new(),
            ans: usize::default(),
        }
    }

    pub fn res(&self) -> usize {
        self.ans
    }

    pub fn syntax_parse(&mut self, expr: &str) -> Result<(), SyntaxParseError> {
        let mut expr: Vec<&str> = expr.trim().split(' ').collect();
        while let Some(item) = expr.pop() {
            self.try_syntax_parse(item)?;
        }
        println!("Engine: {:#?}", self);
        Ok(())
    }

    fn try_syntax_parse(&mut self, item: &str) -> Result<(), SyntaxParseError> {
        let mut is_valid_syntax_item = false;
        if !is_valid_syntax_item {
            match Self::try_parse_literal(item) {
                Ok(literal_item) => {
                    self.global_stack.push(Item::Literal(literal_item));
                    is_valid_syntax_item = true;
                }
                _ => {}
            }
        }
        if !is_valid_syntax_item {
            match Self::try_parse_function(item) {
                Ok(function_item) => {
                    self.global_stack.push(Item::Function(function_item));
                    is_valid_syntax_item = true;
                }
                _ => {}
            }
        }

        if !is_valid_syntax_item {
            match Self::try_parse_variable(item) {
                Ok(variable_item) => {
                    self.global_stack
                        .push(Item::Variable(variable_item.clone()));
                    if !self
                        .variable_stack
                        .iter()
                        .any(|var| var.name == variable_item.name)
                    {
                        self.variable_stack.push(variable_item);
                    }
                    is_valid_syntax_item = true;
                }
                _ => {}
            }
        }

        if !is_valid_syntax_item {
            Err(SyntaxParseError::NotAValidSyntaxItem)
        } else {
            Ok(())
        }
    }

    fn try_parse_literal(item: &str) -> Result<usize, std::num::ParseIntError> {
        item.trim().parse()
    }
    // FIXME: 1. Remove empty variable names and add no action for enter
    fn try_parse_variable(item: &str) -> Result<Variable, SyntaxParseError> {
        let mut var: Variable = Variable::new();
        if !item.is_ascii() {
            return Err(SyntaxParseError::NotAGoodVariableName);
        }
        var.name = item.to_owned();
        Ok(var)
    }

    fn try_parse_function(item: &str) -> Result<OperatorFunctions, SyntaxParseError> {
        let item = item.trim();
        match item {
            "+" => Ok(OperatorFunctions::Add),
            "-" => Ok(OperatorFunctions::Sub),
            "*" => Ok(OperatorFunctions::Mul),
            "/" => Ok(OperatorFunctions::Div),
            "%" => Ok(OperatorFunctions::Mod),
            "=" => Ok(OperatorFunctions::Assignment),
            _ => Err(SyntaxParseError::InvalidFunctionNameFound),
        }
    }
}

impl Engine {
    pub fn eval(&mut self) -> Result<(), SyntaxParseError> {
        let mut e = self.global_stack.len();
        let mut i;
        while e > 0 {
            i = self.global_stack[e - 1].clone();
            match i {
                Item::Literal(val) => {
                    if self.global_stack.len() == 1 {
                        self.ans = val;
                        self.global_stack.push(Item::Literal(self.ans));
                        return Ok(());
                    }
                }
                Item::Function(_) => {
                    if self.global_stack.len() < 3 {
                        return Err(SyntaxParseError::NotAValidSyntaxItem);
                    }
                    let a = self.global_stack.pop().unwrap();
                    let b = self.global_stack.pop().unwrap();
                    let op = self.global_stack.pop().unwrap();
                    self.run(a, b, op)?;
                    self.global_stack.push(Item::Literal(self.ans));
                }
                Item::Variable(_) => match self.global_stack.len() {
                    1 => {
                        let var = self.global_stack.pop().unwrap();
                        let mut var_found = false;
                        let var = match var {
                            Item::Variable(v) => v.name,
                            _ => "".to_owned(),
                        };
                        for j in self.variable_stack.iter_mut() {
                            if j.name == var {
                                self.ans = j.value;
                                var_found = true;
                            }
                        }
                        if !var_found {
                            return Err(SyntaxParseError::NotAValidSyntaxItem);
                        }
                        self.global_stack.push(Item::Literal(self.ans));
                    }
                    _ => {}
                },
            }
            e -= 1;
        }
        if self.global_stack.len() != 1 {
            return Err(SyntaxParseError::NotAValidSyntaxItem);
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.global_stack.clear();
    }

    pub fn run(&mut self, a: Item, b: Item, op: Item) -> Result<(), SyntaxParseError> {
        let a = match a {
            Item::Literal(a) => a,
            Item::Variable(v) => {
                let mut value = 0;
                if self.variable_stack.iter().any(|var| var.name == v.name) {
                    self.variable_stack.iter().for_each(|var| {
                        if var.name == v.name {
                            value = var.value;
                        }
                    });
                } else {
                    return Err(SyntaxParseError::NotAValidSyntaxItem);
                }
                value
            }
            _ => return Err(SyntaxParseError::NotAValidSyntaxItem),
        };
        let b = match b {
            Item::Literal(b) => b,
            Item::Variable(v) => {
                let mut value = 0;
                if self.variable_stack.iter().any(|var| var.name == v.name) {
                    self.variable_stack.iter().for_each(|var| {
                        if var.name == v.name {
                            value = var.value;
                        }
                    });
                } else {
                    return Err(SyntaxParseError::NotAValidSyntaxItem);
                }
                value
            }
            _ => return Err(SyntaxParseError::NotAValidSyntaxItem),
        };
        match op {
            Item::Function(func) => match func {
                OperatorFunctions::Add => self.ans = a + b,
                OperatorFunctions::Sub => self.ans = a - b,
                OperatorFunctions::Mul => self.ans = a * b,
                OperatorFunctions::Div => self.ans = a / b,
                OperatorFunctions::Mod => self.ans = a % b,
                OperatorFunctions::Assignment => {
                    self.ans = a;
                    let length = self.variable_stack.len();
                    self.variable_stack[length - 1].value = a;
                }
            },
            _ => return Err(SyntaxParseError::NotAValidSyntaxItem),
        }
        Ok(())
    }
}

impl Variable {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: usize::default(),
        }
    }
}
