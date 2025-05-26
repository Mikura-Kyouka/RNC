use std::collections::HashMap;

// 表示函数签名：(参数类型列表, 返回类型)
type FunctionSignature = (Vec<String>, Option<String>);

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, String>>,
    functions: HashMap<String, FunctionSignature>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable {
            scopes: Vec::new(),
            functions: HashMap::new(),
        };
        // 初始化一个全局作用域
        symbol_table.enter_scope();
        symbol_table
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if !self.scopes.is_empty() {
            self.scopes.pop();
        }
        // 确保至少有一个作用域
        if self.scopes.is_empty() {
            self.enter_scope();
        }
    }

    pub fn insert(&mut self, name: String, typ: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, typ);
        }
    }

    pub fn get_type(&self, name: &str) -> Option<String> {
        // 从最内层作用域向外层查找
        for scope in self.scopes.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Some(typ.clone());
            }
        }
        None
    }

    pub fn is_declared(&self, name: &str) -> bool {
        self.get_type(name).is_some()
    }

    pub fn is_declared_in_current_scope(&self, name: &str) -> bool {
        if let Some(scope) = self.scopes.last() {
            scope.contains_key(name)
        } else {
            false
        }
    }

    // 添加函数声明
    pub fn add_function(&mut self, name: String, params: Vec<String>, return_type: Option<String>) {
        self.functions.insert(name, (params, return_type));
    }

    pub fn is_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn get_function_return_type(&self, name: &str) -> Option<String> {
        if let Some((_, return_type)) = self.get_function_signature(name) {
            return_type.clone()
        } else {
            None
        }
    }
}
