mod symbol_table;
mod type_checker;

pub use symbol_table::{SymbolTable, SymbolKind, FunctionSignature};
pub use type_checker::TypeChecker;

use crate::ast::{
    Expr, Program, Stmt, BinOp, Variable, ProgramBody, 
    TypeName, VarDec, ProcDec, Param, DeclarePart, TypeDec,
};

use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndefinedIdentifier(String),
    RedefinedIdentifier(String),
    TypeMismatchInAssignment { expected: String, found: String, var_name: String },
    TypeMismatchInOperation { op: String, left: String, right: String },
    TypeMismatchInCondition { expected: String, found: String, context: String },
    ArgumentTypeMismatch { proc_name: String, param_index: usize, expected: String, found: String },
    IdentifierNotVariable(String),
    IdentifierNotProcedure(String),
    IdentifierNotType(String),
    ArgumentCountMismatch { proc_name: String, expected: usize, found: usize },
    ArrayIndexNotInteger(String),
    ArrayAccessOnNonArray(String),
    RecordFieldAccessOnNonRecord(String),
    InvalidRecordField { record_name: String, field_name: String },
    AssignmentToNonVariable(String),
    InvalidOperation(String),
    Other(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndefinedIdentifier(name) => write!(f, "错误：未定义的标识符 '{}'", name),
            SemanticError::RedefinedIdentifier(name) => write!(f, "错误：标识符 '{}' 在当前作用域中重复定义", name),
            SemanticError::TypeMismatchInAssignment { expected, found, var_name } => {
                write!(f, "错误：赋值给 '{}' 时类型不匹配。预期类型 '{}'，实际类型 '{}'", var_name, expected, found)
            }
            SemanticError::TypeMismatchInOperation { op, left, right } => {
                write!(f, "错误：操作 '{}' 中类型不匹配。运算符不能应用于 '{}' 和 '{}'", op, left, right)
            }
            SemanticError::TypeMismatchInCondition { expected, found, context } => {
                write!(f, "错误：{} 条件中类型不匹配。预期类型 '{}'，实际类型 '{}'", context, expected, found)
            }
            SemanticError::ArgumentTypeMismatch { proc_name, param_index, expected, found } => {
                write!(f, "错误：调用过程 '{}' 时，第 {} 个参数类型不匹配。预期类型 '{}'，实际类型 '{}'", proc_name, param_index + 1, expected, found)
            }
            SemanticError::IdentifierNotVariable(name) => write!(f, "错误：标识符 '{}' 不是变量", name),
            SemanticError::IdentifierNotProcedure(name) => write!(f, "错误：标识符 '{}' 不是过程", name),
            SemanticError::IdentifierNotType(name) => write!(f, "错误：标识符 '{}' 不是类型", name),
            SemanticError::ArgumentCountMismatch { proc_name, expected, found } => {
                write!(f, "错误：调用过程 '{}' 时参数数量不匹配。预期 {} 个，实际 {} 个", proc_name, expected, found)
            }
            SemanticError::ArrayIndexNotInteger(context) => write!(f, "错误：数组 '{}' 的索引必须是整数表达式", context),
            SemanticError::ArrayAccessOnNonArray(name) => write!(f, "错误：标识符 '{}' 不是数组，无法执行索引访问", name),
            SemanticError::RecordFieldAccessOnNonRecord(name) => write!(f, "错误：标识符 '{}' 不是记录，无法执行字段访问", name),
            SemanticError::InvalidRecordField { record_name, field_name } => {
                write!(f, "错误：记录 '{}' 没有名为 '{}' 的字段", record_name, field_name)
            }
            SemanticError::AssignmentToNonVariable(name) => write!(f, "错误：无法对 '{}'进行赋值，因为它不是变量", name),
            SemanticError::InvalidOperation(op_desc) => write!(f, "错误：无效操作：{}", op_desc),
            SemanticError::Other(msg) => write!(f, "错误：{}", msg),
        }
    }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
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

    fn resolve_type_string(&mut self, type_str: &str, visited: &mut HashSet<String>) -> Option<String> {
        let normalized_initial = TypeChecker::normalize_type(type_str);
        if ["int", "float", "char", "bool"].contains(&normalized_initial.as_str()) ||
           normalized_initial.starts_with("array of") ||
           (normalized_initial.starts_with("record ") && normalized_initial.ends_with(" end")) {
            return Some(normalized_initial);
        }

        if !visited.insert(type_str.to_string()) {
            self.errors.push(SemanticError::Other(format!("检测到类型别名 '{}' 存在循环定义", type_str)));
            return None;
        }

        let entry_info = self.symbol_table.lookup(type_str)
            .map(|entry| (entry.kind.clone(), entry.typ.clone()));

        let result = match entry_info {
            Some((SymbolKind::TypeIdentifier, aliased_type_str)) => {
                self.resolve_type_string(&aliased_type_str, visited)
            }
            Some((_, _)) => {
                self.errors.push(SemanticError::IdentifierNotType(type_str.to_string()));
                None
            }
            None => {
                self.errors.push(SemanticError::UndefinedIdentifier(format!("类型 '{}'", type_str)));
                None
            }
        };

        visited.remove(type_str);
        result
    }

    fn get_resolved_type_string_for_type_name(&mut self, type_name_node: &TypeName) -> Option<String> {
        match type_name_node {
            TypeName::Base(name) => {
                self.resolve_type_string(name, &mut HashSet::new())
            }
            TypeName::Array { low: _, high: _, base } => {
                self.resolve_type_string(base, &mut HashSet::new())
                    .map(|resolved_base_type| format!("array of {}", resolved_base_type))
            }
            TypeName::Record(fields) => {
                let mut resolved_field_parts = Vec::new();
                for (fname, ftype_name_str) in fields {
                    match self.resolve_type_string(ftype_name_str, &mut HashSet::new()) {
                        Some(resolved_ftype) => {
                            resolved_field_parts.push(format!("{}:{}", fname.to_lowercase(), resolved_ftype));
                        }
                        None => return None,
                    }
                }
                resolved_field_parts.sort();
                Some(format!("record {} end", resolved_field_parts.join(";")))
            }
            TypeName::Alias(name) => {
                self.resolve_type_string(name, &mut HashSet::new())
            }
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        self.errors.clear();

        match program {
            Program::Full { head: _, declare, procs, body } => {
                self.process_type_declarations(&declare.type_decs);
                self.process_variable_declarations(&declare.var_decs);
                self.process_procedure_signatures(procs);

                for proc_dec in procs {
                    self.analyze_procedure_declaration(proc_dec);
                }

                for stmt in &body.stmts {
                    self.analyze_statement(stmt);
                }
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn process_type_declarations(&mut self, type_decs: &Vec<TypeDec>) {
        for type_dec in type_decs {
            if self.get_resolved_type_string_for_type_name(&type_dec.typ).is_some() {
                let definition_string = type_dec.typ.to_string();
                if !self.symbol_table.insert(type_dec.name.clone(), definition_string, SymbolKind::TypeIdentifier) {
                    self.errors.push(SemanticError::RedefinedIdentifier(type_dec.name.clone()));
                }
            }
        }
    }

    fn process_variable_declarations(&mut self, var_decs: &Vec<VarDec>) {
        for var_dec in var_decs {
            match self.get_resolved_type_string_for_type_name(&var_dec.typ) {
                Some(resolved_type_str) => {
                    for name in &var_dec.names {
                        if !self.symbol_table.insert(name.clone(), resolved_type_str.clone(), SymbolKind::Variable) {
                            self.errors.push(SemanticError::RedefinedIdentifier(name.clone()));
                        }
                    }
                }
                None => {
                    for name in &var_dec.names {
                        self.errors.push(SemanticError::Other(format!("变量 '{}' 声明使用了无效或未定义的类型 '{}'", name, var_dec.typ.to_string())));
                    }
                }
            }
        }
    }

    fn process_procedure_signatures(&mut self, procs: &Vec<ProcDec>) {
        for proc_dec in procs {
            let mut param_types = Vec::new();
            let mut params_valid = true;

            for param_group in &proc_dec.params {
                match self.get_resolved_type_string_for_type_name(&param_group.typ) {
                    Some(resolved_param_type_str) => {
                        for _name in &param_group.names {
                            param_types.push(resolved_param_type_str.clone());
                        }
                    }
                    None => {
                        params_valid = false;
                        self.errors.push(SemanticError::Other(format!(
                            "过程 '{}' 的参数类型 '{}' 无效",
                            proc_dec.name, param_group.typ.to_string()
                        )));
                        break;
                    }
                }
            }

            if params_valid {
                if !self.symbol_table.add_function(proc_dec.name.clone(), param_types, None) {
                    self.errors.push(SemanticError::RedefinedIdentifier(proc_dec.name.clone()));
                }
            } else {
                self.errors.push(SemanticError::Other(format!("过程 '{}' 包含无法解析类型的参数，签名未添加。", proc_dec.name)));
            }
        }
    }

    fn analyze_procedure_declaration(&mut self, proc_dec: &ProcDec) {
        self.symbol_table.enter_scope();

        for param_group in &proc_dec.params {
            match self.get_resolved_type_string_for_type_name(&param_group.typ) {
                Some(resolved_param_type_str) => {
                    for name in &param_group.names {
                        if !self.symbol_table.insert(name.clone(), resolved_param_type_str.clone(), SymbolKind::Variable) {
                            self.errors.push(SemanticError::RedefinedIdentifier(name.clone()));
                        }
                    }
                }
                None => {}
            }
        }

        self.process_type_declarations(&proc_dec.declare_part.type_decs);
        self.process_variable_declarations(&proc_dec.declare_part.var_decs);

        for stmt in &proc_dec.body.stmts {
            self.analyze_statement(stmt);
        }

        self.symbol_table.exit_scope();
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { var, expr } => {
                let var_name_str = match var {
                    Variable::Simple(name) => name.clone(),
                    Variable::Array(name, _) => name.clone(),
                    Variable::Record(name, _) => name.clone(),
                };

                self.analyze_expression(expr);
                self.analyze_variable_access(var, true);

                let rhs_type_opt = self.type_checker.infer_type(expr, &self.symbol_table);
                let lhs_type_opt = self.type_checker.infer_type(&Expr::Var(var.clone()), &self.symbol_table);

                if let Some(var_symbol_entry) = self.symbol_table.lookup(&var_name_str) {
                    if var_symbol_entry.kind != SymbolKind::Variable {
                        self.errors.push(SemanticError::AssignmentToNonVariable(var_name_str.clone()));
                    }
                }

                if let (Some(lhs_type), Some(rhs_type)) = (lhs_type_opt, rhs_type_opt) {
                    let norm_lhs_type = TypeChecker::normalize_type(&lhs_type);
                    let norm_rhs_type = TypeChecker::normalize_type(&rhs_type);
                    let compatible = norm_lhs_type == norm_rhs_type ||
                                     (norm_lhs_type == "float" && norm_rhs_type == "int");
                    if !compatible {
                        self.errors.push(SemanticError::TypeMismatchInAssignment {
                            expected: norm_lhs_type,
                            found: norm_rhs_type,
                            var_name: var_name_str,
                        });
                    }
                }
            }
            Stmt::Call { name, args } => {
                for arg_expr in args.iter() {
                    self.analyze_expression(arg_expr);
                }

                match self.symbol_table.lookup(name) {
                    Some(entry) if entry.kind == SymbolKind::Procedure => {
                        if let Some(signature) = self.symbol_table.get_function_signature(name) {
                            let (expected_param_types, _) = signature;
                            if args.len() != expected_param_types.len() {
                                self.errors.push(SemanticError::ArgumentCountMismatch {
                                    proc_name: name.clone(),
                                    expected: expected_param_types.len(),
                                    found: args.len(),
                                });
                            } else {
                                for (i, arg_expr) in args.iter().enumerate() {
                                    if let Some(arg_type_str) = self.type_checker.infer_type(arg_expr, &self.symbol_table) {
                                        let norm_arg_type = TypeChecker::normalize_type(&arg_type_str);
                                        let norm_expected_type = TypeChecker::normalize_type(&expected_param_types[i]);
                                        
                                        let compatible = norm_arg_type == norm_expected_type ||
                                                         (norm_expected_type == "float" && norm_arg_type == "int");

                                        if !compatible {
                                            self.errors.push(SemanticError::ArgumentTypeMismatch {
                                                proc_name: name.clone(),
                                                param_index: i,
                                                expected: norm_expected_type,
                                                found: norm_arg_type,
                                            });
                                        }
                                    }
                                }
                            }
                        } else {
                            self.errors.push(SemanticError::Other(format!("过程 '{}' 在符号表中存在但缺少签名。", name)));
                        }
                    }
                    Some(_) => self.errors.push(SemanticError::IdentifierNotProcedure(name.clone())),
                    None => self.errors.push(SemanticError::UndefinedIdentifier(name.clone())),
                }
            }
            Stmt::If { cond, then, els } => {
                self.analyze_expression(cond);
                if let Some(cond_type) = self.type_checker.infer_type(cond, &self.symbol_table) {
                    if TypeChecker::normalize_type(&cond_type) != "bool" {
                        self.errors.push(SemanticError::TypeMismatchInCondition {
                            expected: "bool".to_string(),
                            found: TypeChecker::normalize_type(&cond_type),
                            context: "if statement".to_string(),
                        });
                    }
                }

                self.symbol_table.enter_scope();
                for stmt_then in then {
                    self.analyze_statement(stmt_then);
                }
                self.symbol_table.exit_scope();

                if !els.is_empty() {
                    self.symbol_table.enter_scope();
                    for stmt_else in els {
                        self.analyze_statement(stmt_else);
                    }
                    self.symbol_table.exit_scope();
                }
            }
            Stmt::While { cond, body } => {
                self.analyze_expression(cond);
                if let Some(cond_type) = self.type_checker.infer_type(cond, &self.symbol_table) {
                    if TypeChecker::normalize_type(&cond_type) != "bool" {
                        self.errors.push(SemanticError::TypeMismatchInCondition {
                            expected: "bool".to_string(),
                            found: TypeChecker::normalize_type(&cond_type),
                            context: "while statement".to_string(),
                        });
                    }
                }

                self.symbol_table.enter_scope();
                for stmt_body in body {
                    self.analyze_statement(stmt_body);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::Read(var) => {
                self.analyze_variable_access(var, true);
                let var_name_str = match var {
                    Variable::Simple(name) => name.clone(),
                    Variable::Array(name, _) => name.clone(),
                    Variable::Record(name, _) => name.clone(),
                };
                match self.symbol_table.lookup(&var_name_str) {
                    Some(entry) if entry.kind != SymbolKind::Variable => {
                        self.errors.push(SemanticError::IdentifierNotVariable(var_name_str));
                    }
                    None => {}
                    _ => {}
                }
            }
            Stmt::Write(expr) => {
                self.analyze_expression(expr);
            }
            Stmt::Return(expr_option) => {
                if let Some(expr) = expr_option {
                    self.analyze_expression(expr);
                }
            }
        }
    }

    fn analyze_variable_access(&mut self, var: &Variable, _is_lvalue: bool) {
        match var {
            Variable::Simple(name) => {
                match self.symbol_table.lookup(name) {
                    Some(entry) => {
                        if entry.kind != SymbolKind::Variable {
                            self.errors.push(SemanticError::IdentifierNotVariable(name.clone()));
                        }
                    }
                    None => {
                        self.errors.push(SemanticError::UndefinedIdentifier(name.clone()));
                    }
                }
            }
            Variable::Array(name, idx_expr) => {
                match self.symbol_table.lookup(name) {
                    Some(entry) => {
                        if entry.kind != SymbolKind::Variable {
                            self.errors.push(SemanticError::IdentifierNotVariable(name.clone()));
                            self.errors.push(SemanticError::ArrayAccessOnNonArray(name.clone()));
                        } else {
                            let var_type_str = TypeChecker::normalize_type(&entry.typ);
                            if var_type_str.starts_with("array of") {
                                self.analyze_expression(idx_expr);
                                if let Some(idx_type) = self.type_checker.infer_type(idx_expr, &self.symbol_table) {
                                    if TypeChecker::normalize_type(&idx_type) != "int" {
                                        self.errors.push(SemanticError::ArrayIndexNotInteger(name.clone()));
                                    }
                                } else {
                                    self.errors.push(SemanticError::Other(format!("无法推断数组 '{}' 索引的类型", name)));
                                }
                            } else {
                                self.errors.push(SemanticError::ArrayAccessOnNonArray(name.clone()));
                            }
                        }
                    }
                    None => {
                        self.errors.push(SemanticError::UndefinedIdentifier(name.clone()));
                    }
                }
            }
            Variable::Record(record_var_name, field_name_str) => {
                match self.symbol_table.lookup(record_var_name) {
                    Some(entry) => {
                        if entry.kind != SymbolKind::Variable {
                            self.errors.push(SemanticError::IdentifierNotVariable(record_var_name.clone()));
                            self.errors.push(SemanticError::RecordFieldAccessOnNonRecord(record_var_name.clone()));
                        } else {
                            let record_type_str = TypeChecker::normalize_type(&entry.typ);

                            if record_type_str.starts_with("record ") && record_type_str.ends_with(" end") {
                                if TypeChecker::get_record_field_type(&record_type_str, field_name_str).is_none() {
                                    self.errors.push(SemanticError::InvalidRecordField {
                                        record_name: record_var_name.clone(),
                                        field_name: field_name_str.clone(),
                                    });
                                }
                            } else {
                                self.errors.push(SemanticError::RecordFieldAccessOnNonRecord(record_var_name.clone()));
                            }
                        }
                    }
                    None => {
                        self.errors.push(SemanticError::UndefinedIdentifier(record_var_name.clone()));
                    }
                }
            }
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Var(var) => {
                self.analyze_variable_access(var, false);
            }
            Expr::Binary { left, op, right } => {
                self.analyze_expression(left);
                self.analyze_expression(right);

                let left_type_opt = self.type_checker.infer_type(left, &self.symbol_table);
                let right_type_opt = self.type_checker.infer_type(right, &self.symbol_table);

                if let (Some(left_type), Some(right_type)) = (left_type_opt, right_type_opt) {
                    if !self.type_checker.check_binary_op(&left_type, op, &right_type) {
                        self.errors.push(SemanticError::TypeMismatchInOperation {
                            op: format!("{:?}", op),
                            left: TypeChecker::normalize_type(&left_type),
                            right: TypeChecker::normalize_type(&right_type),
                        });
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
