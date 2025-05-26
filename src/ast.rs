use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Program {
    Full {
        head: ProgramHead,
        declare: DeclarePart,
        procs: Vec<ProcDec>,
        body: ProgramBody,
    }
}

#[derive(Debug, Clone)]
pub struct ProgramHead {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DeclarePart {
    pub type_decs: Vec<TypeDec>,
    pub var_decs: Vec<VarDec>,
    pub proc_decs: Vec<ProcDec>,
}

#[derive(Debug, Clone)]
pub struct TypeDec {
    pub name: String,
    pub typ: TypeName,
}

#[derive(Debug, Clone)]
pub enum TypeName {
    Base(String),
    Array { low: i32, high: i32, base: String },
    Record(Vec<(String, String)>),
    Alias(String),
}

impl TypeName {
    pub fn to_string(&self) -> String {
        match self {
            TypeName::Base(base) => base.clone(),
            TypeName::Array { low: _, high: _, base } => format!("array of {}", base), // 符合 TypeChecker 期望的��式
            TypeName::Record(fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(name, typ_name)| format!("{}:{}", name, typ_name)) // typ_name 应为字段类型的字符串��示
                    .collect();
                format!("record {} end", field_strs.join(";")) // 符合 TypeChecker 期望的格式
            }
            TypeName::Alias(name) => name.clone(), // 假设别名在使用此字符串之前已被解析
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarDec {
    pub typ: TypeName,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProcDec {
    pub name: String,
    pub params: Vec<Param>,
    pub declare_part: DeclarePart,
    pub body: ProgramBody,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub is_var: bool,
    pub typ: TypeName,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProgramBody {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign { var: Variable, expr: Expr },
    Call { name: String, args: Vec<Expr> },
    If { cond: Expr, then: Vec<Stmt>, els: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Read(Variable),
    Write(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Int(i32),
    Var(Variable),
    Paren(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Eq,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variable {
    Simple(String),
    Array(String, Box<Expr>),
    Record(String, String), // record_name, field_name
}

#[derive(Debug, Clone)]
pub enum VarSuffix {
    None,
    Array(Box<Expr>),
    Record(String),
    RecordArray(String, Box<Expr>),
}

// 定义帮助枚举类型
#[derive(Debug)]
pub enum AssCallRest {
    Assign(VariMoreEnum, Box<Expr>),
    Call(Vec<Expr>)
}

#[derive(Debug)]
pub enum VariMoreEnum {
    None,
    Array(Box<Expr>),
    Field(String)
}

pub fn make_variable(name: String, suffix: VarSuffix) -> Variable {
    match suffix {
        VarSuffix::None => Variable::Simple(name),
        VarSuffix::Array(index) => Variable::Array(name, index),
        VarSuffix::Record(field) => Variable::Record(name, field),
        VarSuffix::RecordArray(field, index) => {
            // a.b[i] => treated as b[i], not supported directly
            // here we just use Record and expect elaboration to handle it
            Variable::Record(name, field) // Further elaboration can make this accurate
        }
    }
}
