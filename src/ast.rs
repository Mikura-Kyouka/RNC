use std::rc::Rc;

#[derive(Debug)]
pub enum Program {
    Full {
        head: ProgramHead,
        declare: DeclarePart,
        body: ProgramBody,
    },
}

#[derive(Debug)]
pub struct ProgramHead {
    pub name: String,
}

#[derive(Debug)]
pub struct DeclarePart {
    pub type_decs: Vec<TypeDec>,
    pub var_decs: Vec<VarDec>,
    pub proc_decs: Vec<ProcDec>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct VarDec {
    pub typ: TypeName,
    pub names: Vec<String>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ProgramBody {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Assign { var: Variable, expr: Expr },
    Call { name: String, args: Vec<Expr> },
    If { cond: Expr, then: Vec<Stmt>, els: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Read(String),
    Write(Expr),
    Return(Expr),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Eq,
}

#[derive(Debug)]
pub enum Variable {
    Simple(String),
    Array(String, Box<Expr>),
    Record(String, String),
}

#[derive(Debug)]
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
