

const ENTRIES_PER_LINE: usize = 8;

pub fn make_rom(data: &[u8], offset: u32) -> String {
    
    let mut result = String::from("mod main(addr: $A) -> $X { output(match(addr) {\n");

    for (index,x) in data.iter().enumerate() {
        result.push_str(&format!("\t {} => {},",index+offset as usize,x));
        if index % ENTRIES_PER_LINE == ENTRIES_PER_LINE-1 {
            result.push('\n');
        }
    }

    result.push_str("})}\n");

    println!("{}",result);
    result
}
