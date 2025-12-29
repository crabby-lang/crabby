; hello with LLVM
; compile or run it with lli -opaque-pointers hello.ll
; just basic llvm stuffs for testing

declare i32 @puts(ptr)

@str = constant [14 x i8] c"Hello, World!\00"

define i32 @main() {
    call i32 @puts(ptr @str)

    ret i32 0
}
