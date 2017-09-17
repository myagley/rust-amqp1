mod str;
mod symbol;
mod variant;

pub use self::str::ByteStr;
pub use self::symbol::Symbol;
pub use self::variant::Variant;
pub use self::variant::VariantMap;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Descriptor {
    Ulong(u64),
    Symbol(Symbol)
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Multiple<T>(pub Vec<T>);