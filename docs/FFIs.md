# Foreign Function Interface (FFI)

**Crabby** provides a simple, explicit, and readable FFI model designed to stay approachable while still
enabling **low-level** interoperability when needed.

> [!IMPORTANT]
> The FFI system is NOT yet implemented. ABI stability, platform-specific calling conventions, and safety guarantees will be finalized in future releases.

## Design Goals

- No `extern "C"` ceremony
- Explicit visibility and linkage
- Minimal surface syntax, OPT-in complexity
- Educational and readable by default

## External Function Declarations

External functions are **declared** using the `extern def` keyword.

```crab
extern def puts(ptr: *u8) -> Int
```

This declares a function symbol that **exist outside** of Crabby (e.g. in a shared library)

## Attributes

To **link** external functions, use the `@ffi` attribute.

```crab
@ffi("libc.so")
extern def puts(ptr: *u8) -> Int
```

> [!IMPORTANT]
> Library names are platform-dependent (e.g. `.dll`, `.dylib`, etc.)
> Resolution strategy will be configurable sooner on.

## Visibility

External symbols can be made **public** using the `pub` keyword.

```crab
@ffi("math.dll")
pub extern def sqrt(x: Float) -> Float
```

## Pointer Usage

Crabby allows **explicit pointer types** when working with FFI.

```crab
extern def memcpy(dst: *u8, src: *u8, len: usize) -> *u8
```

> [!WARNING]
> Pointer safety is the programmer's responsibility.

## No ABI Specifiers (YET)

Crabby intentionally avoids ABI annotations such as:

```crab
extern "C" { ... }
```

Instead, ABI handling will be standardized **internally**.

> [!IMPORTANT]
> Planned Feature: ABI blocks and platform-specific linkage rules.

## Examples

Here is how calling an external functions would look like:

```crbb
@ffi("libc.dll)
extern def puts(msg: *u8) -> Int

unsafe def {
    puts("Hello from Crabby!" as *u8)
}

```

## Future Plans

- ABI blocks
- Safer pointer wrappers
- FFI modules
- Static and dynamic linking modes
- Cross-platfrom resolution helpers
