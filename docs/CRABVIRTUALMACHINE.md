# THE CRAB VIRTUAL MACHINE

The **Crabby Virtual Machine (CVM)** is a custom bytecode-based execution engine designed to run compiled Crabby programs. It aims to be fast, portable, and maintainable, serving as the runtime core for Crabby’s future compiled mode `(crabc)`.

> Status: ⚠️ Early design phase. This document serves as a working draft of the architecture and concepts for CVM.

## 1. ARCHITECTURE

- Stack-Based Execution Model
  - **CVM** will operate using a value stack for computation
  - Bytecode instructions *manipulate* the stack, much like the JVM or Lua VM.

- Registers (Optional)
- A simple register-based **optimization layer** may be introduced later.
- At minimum, and Instruction Pointer (IP) and Call Stack are maintained.
- Modules & Functions
- Crabby programs are compiled into modules with function tables, constant pools, and symbol tables.

## 2. BYTECODE STRUCTURE

**Each compiled instruction (opcode) is represented in binary/byte format as:**

```plaintext
[OPCODE] [OPERAND_1] [OPERAND_2] ...
```

- **Bytecode** is stored in a contiguous instruction list.
- Operands can represent:
  - CONSTANTS
  - STACK OFFSETS
  - LABELS (JUMPS)
  - FUNCTION INDICES
  - MEMORY ADDRESSES (if memory segment is implemented)

## 3. SAMPLE OPCODES

<table>
    <tr>
        <td>OPCODE</td>
        <td>HEX</td>
        <td>DESCRIPTION</td>
        <td>OPERANDS</td>
    </tr>
    <tr>
        <td>LOAD_CONST</td>
        <td>0x01</td>
        <td>Puts a constant to the stack</td>
        <td>const_index</td>
    </tr>
    <tr>
        <td>ADD</td>
        <td>0x02</td>
        <td>Pop 2 values, push sum</td>
        <td>-</td>
    </tr>
    <tr>
        <td>SUB</td>
        <td>0x03</td>
        <td>Pop 2 values, push difference</td>
        <td>-</td>
    </tr>
    <tr>
        <td>MUL</td>
        <td>0x04</td>
        <td>Pop 2 values, push product</td>
        <td>-</td>
    </tr>
    <tr>
        <td>DIV</td>
        <td>0x05</td>
        <td>Pop 2 values, push quotient</td>
        <td>-</td>
    </tr>
    <tr>
        <td>PRINT</td>
        <td>0x06</td>
        <td>Pop value and print</td>
        <td>-</td>
    </tr>
    <tr>
        <td>JUMP</td>
        <td>0x07</td>
        <td>Jump to instruction index</td>
        <td>address</td>
    </tr>
    <tr>
        <td>JUMP_IF</td>
        <td>0x08</td>
        <td>Conditional jump if top is true</td>
        <td>address</td>
    </tr>
    <tr>
        <td>CALL</td>
        <td>0x09</td>
        <td>Call function</td>
        <td>function_index</td>
    </tr>
    <tr>
        <td>RET</td>
        <td>0x0A</td>
        <td>Return from function</td>
        <td>-</td>
    </tr>
</table>

## 4. OPERAND & INSTRUCTION ENCODING

- Opcodes will be represented as **u8** values.
- Operands are **u16 or u32** depending on size.
- An instruction decoder will break the bytecode into `opcode + operands` for execution.

EXAMPLE:

```plain
[0x01][0x00][0x03] => LOAD_CONST 3

[0x02] => ADD

5. STACK BEHAVIOR

Example for 1 + 2:

INSTRUCTIONS:

LOAD_CONST 1

LOAD_CONST 2

ADD

PRINT

STACK TRACE:

[]

[1]

[1, 2]

[3]

<prints 3>
```

## 7. FUTURE FEATURES

- Garbage collection (mark-and-sweep or tracing GC)
- FFI’s to call native libraries
- JIT backend
- WASM target

## 8. STATUS & NOTES

- The **CVM** is still in pre-alpha state.
- Currently, it is being written in `src/vm/` and is not integrated into **Crabby’s main execution pipeline** yet.
- Planned to be activated by `crabby --vm` or gets integrated into the crabc compiler in the future.

## 9. RELATED FILES

- `src/vm/src/bytecode.rs`
- `src/vm/src/vm.rs`
- `src/value.rs (holds the *ValueVM* enum)`
- `docs/MEMORY.md`
