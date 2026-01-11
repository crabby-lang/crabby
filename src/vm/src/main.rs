mod bytecode;
pub mod value;
mod vm;

use bytecode::BytecodeFile;
use compiler::Compiler;
use vm::VM;

fn main() {
    println!("=== Bytecode VM ===\n");

    let source1 = "10 + 5 * 2 ";
    println!("Source: {}", source1);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(source1);

    println!("Rust Debug: {:?}", bytecode);

    let mut vm = VM::new();
    vm.constants = compiler.get_constants();

    let raw_bytecode: Vec<u8> = vm.to_raw_bytecode(bytecode);
    println!("Raw bytecode (hex): {:02X?}", raw_bytecode);

    let bytecode_file: BytecodeFile = BytecodeFile::new(bytecode.to_vec(), vm.constants.clone());
    bytecode_file
        .save_to_file("crabby1.cby")
        .expect("Failed to save bytecode file");

    let result: Option<_> = vm.execute(bytecode);
    println!("Result: {:?}\n", result);

    println!("=== Loading from .cby file ===");
    let loaded_file: BytecodeFile = bytecode_file
        .load_from_file("crabby1.cby")
        .expect("Failed to load bytecode file!");
    println!("Loaded bytecode from crabby1.cby");

    let mut vm2: VM = VM::new();
    vm2.constants = loaded_file.constants;
    vm2.print_bytecode(&loaded_file.instructions);

    let result2: Option<_> = vm2.execute(&loaded_file.instructions);
    println!("Result from loaded file: {:?}\n", result2);

    let source2: &'static str = "x = 42; y = 8; x + y";
    println!("Source: {}", source2);

    let mut compiler = Compiler::new();
    let bytecode: [vm::Instructions] = compiler.compile(source2);

    let mut vm = VM::new();
    vm.constants = compiler.get_constants();
    vm.print_bytecode(&bytecode);

    let bytecode_file: BytecodeFile = BytecodeFile::new(bytecode.clone(), vm.constants.clone());
    bytecode_file
        .save_to_file("variables.cby")
        .expect("Failed to save bytecode file");
    println!("Saved to variables.cby");

    let result: Option<_> = vm.execute(&bytecode);
    println!("Result: {:?}", result);
}
