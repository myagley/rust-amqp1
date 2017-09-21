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
    Symbol(Symbol),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Multiple<T>(pub Vec<T>);

impl<T> Multiple<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> ::std::slice::Iter<T> {
        self.0.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct List(pub Vec<Variant>);

impl List {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> ::std::slice::Iter<Variant> {
        self.0.iter()
    }
}
