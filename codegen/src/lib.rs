mod parse;
mod gen;

use std::path::Path;
use parse::parse_opcodes;

fn write_to_path(code: String, path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    ::std::fs::write(path, code)
        .unwrap_or_else(|err| panic!(
            "Failed to write generated code to `{}`: {}",
            path.display(), err,
        ));
    Ok(()) 
}

pub fn prepare_execute(source_path: impl AsRef<Path>, target_path: impl AsRef<Path>) {
    let instructions = parse_opcodes(source_path);
    // println!("{instructions:?}");

    let code = gen::execute(instructions).unwrap();
    // println!("{code}"); 

    
    write_to_path(code, target_path).unwrap();
}

pub fn prepare_mapping() { 

}