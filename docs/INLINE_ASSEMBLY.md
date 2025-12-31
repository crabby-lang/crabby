# Inline Assembly in Crabby

Welcome to Crabby's inline assembly syntax tour and concept, we appreciate if you'd love to
contribute to this documentation or to Crabby in general! 🩷

> [!WARNING]
> Still very early, conceptual, and experimental, low level Crabby will be implemented later on but for now this docs exist as a preview or **demo** for Crabby's low level capabilities, note also that changes will occur once major updates have been release, Thank you for your patience.

## Why Assembly in Crabby?

In Crabby, memory safety is enabled by default, meaning memory-related bugs and crashes rarely
happen due to Crabby's nature of handling variables, functions, and ownership very well.

However, you can handle low-level stuffs with **features** such as pointers, memory-related code,
`unsafe` blocks, `ref` and even Assembly coding!

Usually conceptual and experimental for now, but in Crabby coding with x86 or arm Assembly looks
like this:

```crab
// examples/low/assembly.crab

mode unstrict

unsafe {
    @asm(
        arch="native", // <-- this means the decorator/attribute detects what architecture you are on
        "mov eax, 1",
        "int 0x80",
    )
}
```

And by looking at the code you may be wondering:

```crab

   mode unstrict
// ^^^^ Wtf is this?
```

Well it's a **planned and experimental** feature that disables *some of* the safety mechanisms of
Crabby by allowing programmers to write unsafe or low-level code.

> Q: What happens if we don't apply the mode keyword?

Well... it'll throw:

```bash
crabby assembly.crab

ERR: Cannot use unsafe or low-level code blocks inside while 'mode' is missing!
    |
    | > unsafe { ... }
    |   ^^^^^^
    | note: add 'mode unstrict' in the first line of your file.   

```

## Supported Architecture

> [!Warning]
> CPU support will change vary in the future, expect some or few to be added when Crabby has implemented a Assembly feature into it, Thank you.

- **x86_64**

```crab
unsafe {
    @asm(arch="x86",
        "mov rax, 60",
        "mov rdi, 0",
        "syscall",
    )
}
```

- **arm64**

```crab
unsafe {
    @asm(arch="arm",
        "mov x8, 93",
        "mov x0, 0",
        "svc #0",
    )
}
```


- **Risc-V**

Planned, its currently **unofficial** to support the Risc-V architecture but once demands or are available we'll implement an official documentation for Risc-V

Right now, this is conceptual(just like the others):

```crab
unsafe {
    @asm(arch="riscv",
        "li a7, 93",
        "li a0, 0",
        "ecall",
    )
}
```
