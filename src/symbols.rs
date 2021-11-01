use serde::{Serialize, Deserialize};

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use crate::blueprint::Signal;

struct SymbolInfo {
    signals: Vec<Signal>,
    ident_map: HashMap<String, u32>
}

#[derive(Debug,Serialize,Deserialize)]
struct ParseSymbol{
    id: String,
    signal: Signal
}

static SYMBOL_INFO: OnceCell<SymbolInfo> = OnceCell::new();

pub fn load_symbols(json: &str) {
    let symbols: Vec<ParseSymbol> = serde_json::from_str(&json).expect("bad json");
    let mut info = SymbolInfo{signals:Vec::new(),ident_map:HashMap::new()};
    for symbol in symbols {
        let index = info.signals.len() as u32;
        info.signals.push(symbol.signal);
        info.ident_map.insert(symbol.id.to_uppercase(),index);
    }
    SYMBOL_INFO.set(info).ok();
}

// TODO bidirectional mapping
pub fn signal_from_symbol_index(index: u32) -> Signal {
    let info = SYMBOL_INFO.get().expect("symbol info not loaded");
    info.signals[index as usize].clone()
}

pub fn symbol_index_from_identifier(ident: &str) -> u32 {
    let info = SYMBOL_INFO.get().expect("symbol info not loaded");
    if let Some(n) = info.ident_map.get(&ident.to_uppercase()) {
        *n
    } else {
        panic!("Signal name '{}' does not exist.",ident);
    }
}
