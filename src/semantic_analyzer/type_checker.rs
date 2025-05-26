use crate::ast::{Expr, BinOp, Variable};
use super::symbol_table::{SymbolTable, SymbolKind};

pub struct TypeChecker {}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {}
    }

    pub fn get_array_element_type(array_type_str: &str) -> Option<String> {
        if array_type_str.to_lowercase().starts_with("array of ") {
            Some(array_type_str[9..].trim().to_string())
        } else {
            None
        }
    }

    pub(crate) fn get_record_field_type(record_type_str: &str, field_name: &str) -> Option<String> {
        let lower_record_str = record_type_str.to_lowercase();
        if lower_record_str.starts_with("record ") && lower_record_str.ends_with(" end") {
            let fields_part = &lower_record_str[7..lower_record_str.len() - 4];
            for field_declaration in fields_part.split(';') {
                let parts: Vec<&str> = field_declaration.trim().split(':').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim();
                    let typ = parts[1].trim();
                    if name == field_name.to_lowercase() {
                        return Some(typ.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn infer_type(&self, expr: &Expr, symbol_table: &SymbolTable) -> Option<String> {
        match expr {
            Expr::Int(_) => Some("int".to_string()),
            Expr::Var(var) => {
                match var {
                    Variable::Simple(name) => {
                        symbol_table.lookup(name).and_then(|entry| {
                            if entry.kind == SymbolKind::Variable {
                                Some(TypeChecker::normalize_type(&entry.typ))
                            } else {
                                None
                            }
                        })
                    }
                    Variable::Array(name, index_expr) => {
                        if let Some(entry) = symbol_table.lookup(name) {
                            if entry.kind == SymbolKind::Variable {
                                if let Some(element_type) = TypeChecker::get_array_element_type(&entry.typ) {
                                    Some(TypeChecker::normalize_type(&element_type))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Variable::Record(record_name, field_name) => {
                        if let Some(entry) = symbol_table.lookup(record_name) {
                            if entry.kind == SymbolKind::Variable {
                                if let Some(field_type) = TypeChecker::get_record_field_type(&entry.typ, field_name) {
                                    Some(TypeChecker::normalize_type(&field_type))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let left_type = self.infer_type(left, symbol_table);
                let right_type = self.infer_type(right, symbol_table);

                match (left_type, right_type) {
                    (Some(lt), Some(rt)) => {
                        self.infer_binary_op_type(&lt, op, &rt)
                    }
                    _ => None,
                }
            }
            Expr::Paren(inner) => self.infer_type(inner, symbol_table),
        }
    }

    pub fn check_binary_op(&self, left_type: &str, op: &BinOp, right_type: &str) -> bool {
        let norm_left_type = TypeChecker::normalize_type(left_type);
        let norm_right_type = TypeChecker::normalize_type(right_type);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                matches!((&norm_left_type[..], &norm_right_type[..]), ("int", "int")) ||
                matches!((&norm_left_type[..], &norm_right_type[..]), ("float", "float")) ||
                matches!((&norm_left_type[..], &norm_right_type[..]), ("int", "float")) ||
                matches!((&norm_left_type[..], &norm_right_type[..]), ("float", "int"))
            }
            BinOp::Lt => {
                (matches!((&norm_left_type[..], &norm_right_type[..]), ("int", "int")) ||
                 matches!((&norm_left_type[..], &norm_right_type[..]), ("float", "float")) ||
                 matches!((&norm_left_type[..], &norm_right_type[..]), ("char", "char")))
            }
            BinOp::Eq => {
                norm_left_type == norm_right_type ||
                (norm_left_type == "int" && norm_right_type == "float") ||
                (norm_left_type == "float" && norm_right_type == "int")
            }
        }
    }

    fn infer_binary_op_type(&self, left_type: &str, op: &BinOp, right_type: &str) -> Option<String> {
        let norm_left_type = TypeChecker::normalize_type(left_type);
        let norm_right_type = TypeChecker::normalize_type(right_type);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if (norm_left_type == "int" || norm_left_type == "float") &&
                   (norm_right_type == "int" || norm_right_type == "float") {
                    if norm_left_type == "float" || norm_right_type == "float" {
                        Some("float".to_string())
                    } else {
                        Some("int".to_string())
                    }
                } else {
                    None
                }
            }
            BinOp::Lt | BinOp::Eq => {
                if self.check_binary_op(&norm_left_type, op, &norm_right_type) {
                    Some("bool".to_string())
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn normalize_type(typ: &str) -> String {
        match typ.trim().to_lowercase().as_str() {
            "integer" | "int" | "longint" => "int".to_string(),
            "real" | "float" | "double" => "float".to_string(),
            "char" => "char".to_string(),
            "boolean" | "bool" => "bool".to_string(),
            s if s.starts_with("array of ") => {
                let element_type = TypeChecker::normalize_type(&s[9..].trim());
                format!("array of {}", element_type)
            }
            s if s.starts_with("record ") && s.ends_with(" end") => {
                typ.to_lowercase()
            }
            other => other.trim().to_lowercase().to_string(),
        }
    }
}
