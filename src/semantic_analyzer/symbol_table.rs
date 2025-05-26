use std::collections::HashMap;

// 表示函数签名：(参数类型列表, 返回类型)
pub type FunctionSignature = (Vec<String>, Option<String>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Procedure,
    TypeIdentifier,
    // Potentially others like Constant, etc.
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub typ: String,          // Type of the symbol (e.g., "int", "array of int", "procedure")
    pub kind: SymbolKind,
    // pub definition_node: Option<AstNodeId>, // For more detailed error reporting or go-to-definition
    // pub details: Option<TypeDetails>, // For array bounds, record fields, etc.
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolEntry>>,
    functions: HashMap<String, FunctionSignature>, // Stores procedure signatures specifically
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
        if self.scopes.len() > 1 { // Keep at least the global scope
            self.scopes.pop();
        }
    }

    // Inserts a symbol into the current scope.
    // Returns true if insertion was successful, false if already declared in current scope.
    pub fn insert(&mut self, name: String, typ: String, kind: SymbolKind) -> bool {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return false; // Already declared in current scope
            }
            scope.insert(name, SymbolEntry { typ, kind });
            true
        } else {
            // Should not happen if enter_scope is called initially
            false
        }
    }

    // Looks up a symbol in all scopes, from innermost to outermost.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
    
    pub fn get_type(&self, name: &str) -> Option<String> {
        self.lookup(name).map(|entry| entry.typ.clone())
    }

    pub fn get_kind(&self, name: &str) -> Option<SymbolKind> {
        self.lookup(name).map(|entry| entry.kind.clone())
    }


    pub fn is_declared(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    pub fn is_declared_in_current_scope(&self, name: &str) -> bool {
        if let Some(scope) = self.scopes.last() {
            scope.contains_key(name)
        } else {
            false
        }
    }

    // Adds a function/procedure. Also adds its name to the symbol table.
    pub fn add_function(&mut self, name: String, params: Vec<String>, return_type: Option<String>) -> bool {
        if self.functions.contains_key(&name) || self.is_declared_in_current_scope(&name) {
            return false; // Function/Procedure already declared
        }
        // Store the signature
        self.functions.insert(name.clone(), (params, return_type));
        // Add to symbol table as a procedure
        if let Some(scope) = self.scopes.last_mut() {
             // Typically, procedure type is just "procedure" or its signature stringified
            scope.insert(name, SymbolEntry { typ: "procedure".to_string(), kind: SymbolKind::Procedure });
            true
        } else {
            false
        }
    }

    pub fn is_function(&self, name: &str) -> bool {
        // Check both the functions map and the symbol kind for consistency
        self.functions.contains_key(name) && 
        self.lookup(name).map_or(false, |e| e.kind == SymbolKind::Procedure)
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
