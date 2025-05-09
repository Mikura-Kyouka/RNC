use std::collections::{HashMap, HashSet};
use crate::ast::*;

#[derive(Debug, Clone)]
pub enum SemanticError {
    DuplicateIdentifier(String),
    UndefinedIdentifier(String),
    TypeMismatch(String),
    InvalidProcedureCall(String),
    Other(String),
}

#[derive(Default, Clone)]
struct SymbolTable {
    vars: HashMap<String, TypeName>,
    types: HashMap<String, TypeName>,
    procedures: HashMap<String, ProcSignature>,
}

#[derive(Debug, Clone)]
struct ProcSignature {
    params: Vec<Param>,
}

pub fn analyze(program: &Program) -> Result<(), Vec<SemanticError>> {
    let mut table = SymbolTable::default();
    let mut errors = Vec::new();

    if let Program::Full { head: _, declare, body } = program {
        process_declarations(declare, &mut table, &mut errors);
        check_statements(&body.stmts, &mut table, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn process_declarations(declare: &DeclarePart, table: &mut SymbolTable, errors: &mut Vec<SemanticError>) {
    for td in &declare.type_decs {
        if table.types.contains_key(&td.name) {
            errors.push(SemanticError::DuplicateIdentifier(td.name.clone()));
        } else {
            table.types.insert(td.name.clone(), td.typ.clone());
        }
    }

    for vd in &declare.var_decs {
        for name in &vd.names {
            if table.vars.contains_key(name) {
                errors.push(SemanticError::DuplicateIdentifier(name.clone()));
            } else {
                table.vars.insert(name.clone(), vd.typ.clone());
            }
        }
    }

    for proc in &declare.proc_decs {
        if table.procedures.contains_key(&proc.name) {
            errors.push(SemanticError::DuplicateIdentifier(proc.name.clone()));
        } else {
            let sig = ProcSignature {
                params: proc.params.clone(),
            };
            table.procedures.insert(proc.name.clone(), sig);
        }

        // recurse into proc
        let mut proc_table = table.clone();
        for param in &proc.params {
            for name in &param.names {
                proc_table.vars.insert(name.clone(), param.typ.clone());
            }
        }
        process_declarations(&proc.declare_part, &mut proc_table, errors);
        check_statements(&proc.body.stmts, &mut proc_table, errors);
    }
}

fn check_statements(stmts: &[Stmt], table: &mut SymbolTable, errors: &mut Vec<SemanticError>) {
    for stmt in stmts {
        match stmt {
            Stmt::Assign { var, expr } => {
                check_variable(var, table, errors);
                check_expr(expr, table, errors);
            }
            Stmt::Call { name, args } => {
                if let Some(sig) = table.procedures.get(name) {
                    if sig.params.len() != args.len() {
                        errors.push(SemanticError::InvalidProcedureCall(name.clone()));
                    }
                    for expr in args {
                        check_expr(expr, table, errors);
                    }
                } else {
                    errors.push(SemanticError::UndefinedIdentifier(name.clone()));
                }
            }
            Stmt::If { cond, then, els } => {
                check_expr(cond, table, errors);
                check_statements(then, table, errors);
                check_statements(els, table, errors);
            }
            Stmt::While { cond, body } => {
                check_expr(cond, table, errors);
                check_statements(body, table, errors);
            }
            Stmt::Read(name) => {
                if !table.vars.contains_key(name) {
                    errors.push(SemanticError::UndefinedIdentifier(name.clone()));
                }
            }
            Stmt::Write(expr) => {
                check_expr(expr, table, errors);
            }
            Stmt::Return(expr) => {
                check_expr(expr, table, errors);
            }
        }
    }
}

fn check_variable(var: &Variable, table: &SymbolTable, errors: &mut Vec<SemanticError>) {
    match var {
        Variable::Simple(name) => {
            if !table.vars.contains_key(name) {
                errors.push(SemanticError::UndefinedIdentifier(name.clone()));
            }
        }
        Variable::Array(name, expr) => {
            if !table.vars.contains_key(name) {
                errors.push(SemanticError::UndefinedIdentifier(name.clone()));
            }
            check_expr(expr, table, errors);
        }
        Variable::Record(name, field) => {
            if !table.vars.contains_key(name) {
                errors.push(SemanticError::UndefinedIdentifier(name.clone()));
            } else {
                // 暂不验证 field 是否在结构体中
            }
        }
    }
}

fn check_expr(expr: &Expr, table: &SymbolTable, errors: &mut Vec<SemanticError>) {
    match expr {
        Expr::Binary { left, right, .. } => {
            check_expr(left, table, errors);
            check_expr(right, table, errors);
        }
        Expr::Int(_) => {}
        Expr::Var(v) => check_variable(v, table, errors),
        Expr::Paren(e) => check_expr(e, table, errors),
    }
}
