use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::bail;
use itertools::Itertools;
use twenty_first::shared_math::b_field_element::BFieldElement;
use twenty_first::shared_math::rescue_prime_digest::Digest;
use twenty_first::shared_math::x_field_element::XFieldElement;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Fn<T> {
    pub name: String,
    pub args: Vec<FnArg>,
    pub body: Vec<Stmt<T>>,
    pub output: DataType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FnArg {
    pub name: String,
    pub data_type: DataType,
}

impl Display for FnArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.data_type)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Stmt<T> {
    Let(LetStmt<T>),
    Assign(AssignStmt<T>),
    Return(Expr<T>),
    // FIXME: Type-check that functions not bound to variables don't return anything
    FnCall(FnCall<T>),
    While(WhileStmt<T>), // TODO: Control-flow operators: if-else, while, etc.
    If(IfStmt<T>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WhileStmt<T> {
    pub condition: Expr<T>,
    pub stmts: Vec<Stmt<T>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IfStmt<T> {
    pub condition: Expr<T>,
    pub if_branch: Vec<Stmt<T>>,
    pub else_branch: Vec<Stmt<T>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ExprLit {
    Bool(bool),
    U32(u32),
    U64(u64),
    BFE(BFieldElement),
    XFE(XFieldElement),
    Digest(Digest),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BinOp {
    Add,
    And,
    BitAnd,
    BitXor,
    Div,
    Eq,
    Lt,
    Mul,
    Neq,
    Or,
    Rem,
    Shl,
    Shr,
    Sub,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Expr<T> {
    Lit(ExprLit, T),
    Var(Identifier<T>), // x[i]
    // Index(Box<Expr<T>>, Box<Expr<T>>), // a_expr[i_expr]    (a + 5)[3]
    FlatList(Vec<Expr<T>>),
    FnCall(FnCall<T>),
    Binop(Box<Expr<T>>, BinOp, Box<Expr<T>>, T),
    If(ExprIf<T>),
    // TODO: Overloaded arithmetic operators
    // TODO: VM-specific intrinsics (hash, absorb, squeeze, etc.)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ExprIf<T> {
    pub condition: Box<Expr<T>>,
    pub then_branch: Box<Expr<T>>,
    pub else_branch: Box<Expr<T>>,
}

pub struct SymTable(HashMap<String, (u8, DataType)>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DataType {
    Bool,
    U32,
    U64,
    BFE,
    XFE,
    Digest,
    List(Box<DataType>),
    FlatList(Vec<DataType>),
}

impl FromStr for DataType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bool" => Ok(DataType::Bool),
            "u32" => Ok(DataType::U32),
            "u64" => Ok(DataType::U64),
            "BFieldElement" => Ok(DataType::BFE),
            "XFieldElement" => Ok(DataType::XFE),
            "Digest" => Ok(DataType::Digest),
            ty => bail!("Unsupported type {}", ty),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DataType::*;
        write!(
            f,
            "{}",
            match self {
                Bool => "bool".to_string(),
                U32 => "u32".to_string(),
                U64 => "u64".to_string(),
                BFE => "BField".to_string(),
                XFE => "XField".to_string(),
                Digest => "Digest".to_string(),
                List(ty) => format!("List({})", ty),
                FlatList(tys) => tys.iter().map(|ty| format!("{}", ty)).join(" "),
            }
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Identifier<T> {
    String(String, T),                           // x
    TupleIndex(Box<Identifier<T>>, usize),       // x.0
    ListIndex(Box<Identifier<T>>, Box<Expr<T>>), // x[0]
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AssignStmt<T> {
    pub identifier: Identifier<T>,
    pub expr: Expr<T>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LetStmt<T> {
    pub var_name: String,
    pub data_type: DataType,
    pub expr: Expr<T>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FnCall<T> {
    pub name: String,
    pub args: Vec<Expr<T>>, // FIXME: type-check that this is flat
    pub annot: T,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Typing {
    UnknownType,
    KnownType(DataType),
}

impl Default for Typing {
    fn default() -> Self {
        Typing::UnknownType
    }
}
