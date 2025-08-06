# Crabby's Memory Management

> ## ⚠️ Note: Still being planned and developed

Crabby uses a **hybrid memory model** that combines Rust-inspired ownership & borrowing with optional
Garbage Collection (GC). This allows Crabby to be **both safe for beginners** and **flexible for advance users**.

---

## Memory Model Overview

<details>

<summary><strong>Ownership && Borrowing</strong></summary>

- Every value in Crabby has a **single owner**
- Ownership is transferred on assignment or function calls (similar to **Rust**)
- Borrowing is supported using the `&` (immutable) and `mut` (mutable) borrow syntax.

Example:

```rs
let mut emoji = "🦀"
let ref alias = &emoji
```

</details>

<details>

<summary><strong>Garbage Collection</strong></summary>

- Crabby has a secondary GC layer for dynamically allocated objects like closures, lambdas, and runtime-created data structures.

- The garbage collector runs incrementally to prevent long pauses.

- Safe code cannot cause memory leaks, but `unsafe` blocks can bypass these guarantees.

</details>

## Unsafe Memory

**Crabby** allows advanced users to mark code as `unsafe` for **performance or FFI** purposes:

```rs
unsafe {
    move ptr
    ref externals
}

/* You can do unsafe def {} too */

unsafe def {
    /* unsafe function code */
}
```

> ### ⚠️ WARNING
>
> **Unsafe blocks bypass memory checks. We do not recommend using this if you're new to Crabby, use at your own risk.**

## Stack Overflow Prevention

Crabby Includes:

- Recursion depth limit (configurable)

- Runtime stack monitoring

- Planned tail-call optimizing (TCO) for deeply recursive functions

```rs
if call_stack_depth > 8192 {
    throw StackOverflowError
}
```

## Future Plans

- [ ] Generational GC
- [ ] Arena allocation for performance
- [ ] Memory-safe pointers
- [ ] Integration with Web Assembly (WASM) linear memory for portability

---
