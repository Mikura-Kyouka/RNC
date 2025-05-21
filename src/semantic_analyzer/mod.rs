mod symbol_table;
mod type_checker;

pub use symbol_table::SymbolTable;
pub use type_checker::TypeChecker;

// 适配 ast.rs 的结构
use crate::ast::{
    Expr, Program, Stmt, BinOp, Variable, ProgramBody, 
    TypeName, VarDec, ProcDec, Param,
};

use std::fmt;

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable(String),
    TypeMismatch(String, String, String),
    InvalidOperation(String),
    RedefinedVariable(String),
    InvalidFunctionCall(String),
    Other(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            SemanticError::TypeMismatch(expected, found, context) => {
                write!(f, "Type mismatch in {}: expected {}, found {}", context, expected, found)
            }
            SemanticError::InvalidOperation(op) => write!(f, "Invalid operation: {}", op),
            SemanticError::RedefinedVariable(name) => {
                write!(f, "Variable redefined: {}", name)
            }
            SemanticError::InvalidFunctionCall(name) => {
                write!(f, "Invalid function call: {}", name)
            }
            SemanticError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_checker: TypeChecker,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            type_checker: TypeChecker::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        self.errors.clear();
        self.symbol_table.enter_scope();

        match program {
            Program::Full { body, declare, procs, .. } => {
                // 1. 先插入全局类型、变量和过程声明
                self.process_declarations(declare, procs);

                // 2. 再分析主程序体语句
                for stmt in &body.stmts {
                    self.analyze_statement(stmt);
                }
            }
        }

        self.symbol_table.exit_scope();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn process_declarations(&mut self, declare: &crate::ast::DeclarePart, procs: &Vec<ProcDec>) {
        // 处理变量声明
        for var_dec in &declare.var_decs {
            let typ_str = var_dec.typ.to_string();
            for name in &var_dec.names {
                if self.symbol_table.is_declared_in_current_scope(name) {
                    self.errors.push(SemanticError::RedefinedVariable(name.clone()));
                } else {
                    self.symbol_table.insert(name.clone(), typ_str.clone());
                }
            }
        }
        // 处理过程声明
        for proc_dec in procs {
            let mut param_types = Vec::new();
            for param in &proc_dec.params {
                let typ_str = param.typ.to_string();
                for _ in &param.names {
                    param_types.push(typ_str.clone());
                }
            }
            self.symbol_table.add_function(proc_dec.name.clone(), param_types, None);
        }
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { var, expr } => {
                match var {
                    Variable::Simple(name) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                    }
                    Variable::Array(name, _) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                    }
                    Variable::Record(name, _) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                    }
                }
                self.analyze_expression(expr);
            }
            Stmt::Call { name, args } => {
                if !self.symbol_table.is_function(name) {
                    self.errors.push(SemanticError::InvalidFunctionCall(name.clone()));
                } else {
                    for arg in args {
                        self.analyze_expression(arg);
                    }
                }
            }
            Stmt::If { cond, then, els } => {
                self.analyze_expression(cond);
                self.symbol_table.enter_scope();
                for stmt in then {
                    self.analyze_statement(stmt);
                }
                self.symbol_table.exit_scope();
                self.symbol_table.enter_scope();
                for stmt in els {
                    self.analyze_statement(stmt);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::While { cond, body } => {
                self.analyze_expression(cond);
                self.symbol_table.enter_scope();
                for stmt in body {
                    self.analyze_statement(stmt);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::Read(_name) => {}
            Stmt::Write(expr) => {
                self.analyze_expression(expr);
            }
            Stmt::Return(expr) => {
                self.analyze_expression(expr);
            }
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                    }
                    Variable::Array(name, idx) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                        self.analyze_expression(idx);
                    }
                    Variable::Record(name, _field) => {
                        if !self.symbol_table.is_declared(name) {
                            self.errors.push(SemanticError::UndefinedVariable(name.clone()));
                        }
                    }
                }
            }
            Expr::Binary { left, op, right } => {
                self.analyze_expression(left);
                self.analyze_expression(right);

                let left_type = self.type_checker.infer_type(left, &self.symbol_table);
                let right_type = self.type_checker.infer_type(right, &self.symbol_table);

                if let (Some(left_type), Some(right_type)) = (left_type, right_type) {
                    if !self.type_checker.check_binary_op(&left_type, op, &right_type) {
                        self.errors.push(SemanticError::InvalidOperation(format!(
                            "Cannot apply '{:?}' to types '{}' and '{}'",
                            op, left_type, right_type
                        )));
                    }
                }
            }
            Expr::Int(_) => {}
            Expr::Paren(inner) => {
                self.analyze_expression(inner);
            }
        }
    }
}
