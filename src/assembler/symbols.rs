#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: Option<u32>,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType) -> Self {
        Self {
            name,
            offset: None,
            symbol_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Add symbol to table
    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s)
    }

    /// If contain symbol
    pub fn contain_symbol(&self, name: &str) -> bool {
        self.symbols.iter().find(|s| s.name == name).is_some()
    }

    /// Get symbol offset by name
    pub fn symbol_value(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|s| s.name == name)
            .map_or(None, |s| s.offset)
    }

    /// Set symbol offset
    pub fn set_symbol_offset(&mut self, name: &str, offset: u32) -> bool {
        self.symbols
            .iter_mut()
            .find(|s| s.name == name)
            .map_or(false, |s| {
                s.offset = Some(offset);
                true
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), SymbolType::Label);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }
}
