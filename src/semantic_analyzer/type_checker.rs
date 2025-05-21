use crate::ast::{Expr, BinOp, Variable};
use super::symbol_table::SymbolTable;

pub struct TypeChecker {}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {}
    }

    pub fn infer_type(&self, expr: &Expr, symbol_table: &SymbolTable) -> Option<String> {
        match expr {
            Expr::Int(_) => Some("int".to_string()),
            Expr::Var(var) => {
                match var {
                    Variable::Simple(name) => symbol_table.get_type(name).map(|t| Self::normalize_type(&t)),
                    Variable::Array(name, _) => symbol_table.get_type(name).map(|t| Self::normalize_type(&t)),
                    Variable::Record(name, _) => symbol_table.get_type(name).map(|t| Self::normalize_type(&t)),
                }
            }
            Expr::Binary { op, left, right } => {
                let left_type = self.infer_type(left, symbol_table);
                let right_type = self.infer_type(right, symbol_table);

                match (left_type, right_type) {
                    (Some(left_type), Some(right_type)) => {
                        self.infer_binary_op_type(&left_type, op, &right_type)
                    }
                    _ => None,
                }
            }
            Expr::Paren(inner) => self.infer_type(inner, symbol_table),
        }
    }

    pub fn check_binary_op(&self, left_type: &str, op: &BinOp, right_type: &str) -> bool {
        let left_type = Self::normalize_type(left_type);
        let right_type = Self::normalize_type(right_type);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                matches!((&left_type[..], &right_type[..]), ("int", "int"))
            }
            BinOp::Lt => {
                matches!((&left_type[..], &right_type[..]), ("int", "int"))
            }
            BinOp::Eq => {
                left_type == right_type
            }
        }
    }

    fn infer_binary_op_type(&self, left_type: &str, op: &BinOp, right_type: &str) -> Option<String> {
        let left_type = Self::normalize_type(left_type);
        let right_type = Self::normalize_type(right_type);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if left_type == "int" && right_type == "int" {
                    Some("int".to_string())
                } else {
                    None
                }
            }
            BinOp::Lt | BinOp::Eq => Some("int".to_string()),
        }
    }

    fn normalize_type(typ: &str) -> String {
        match typ.trim().to_lowercase().as_str() {
            "integer" | "int" | "longint" => "int".to_string(),
            "real" | "float" | "double" => "float".to_string(),
            "char" => "char".to_string(),
            "boolean" | "bool" => "bool".to_string(),
            other => other.to_string(),
        }
    }
}
