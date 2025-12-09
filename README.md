# Crabby: The Modern Programming Language Written in Rust

![Crabby](https://avatars.githubusercontent.com/u/139462470?s=48&v=4)

![GitHub contributors](https://img.shields.io/github/contributors/crabby-lang/crabby?style=for-the-badge&color=blue)

![GitHub License](https://img.shields.io/github/license/crabby-lang/crabby?style=for-the-badge&logo=gnu&logoColor=%23A42E2B)

![GitHub Repo stars](https://img.shields.io/github/stars/Kazooki123/crabby?style=for-the-badge&logo=github&color=orange)

## Introduction

![Logo](https://github.com/crabby-lang/crabby/blob/main/crabbylogo.png)

> [!WARNING]
> **Crabby** is *still under development* with new features and bugs being fixed, if you encounter an error then that's **OUR** fault, and **we're still fixing it**. Crabby is **very new** due to the `rewrite update` and is still experiencing bugs and errors. **If you want to help out, please do!**

<h3><b>Shout out to these wonderful programmers helping:</b></h3>

<table>
  <tr>
    <td><a href="https://github.com/grngxd">@grngxd</a></td>
    <td><a href="https://github.com/Aleksandere84">@Aleksandere84</a></td>
    <td><a href="https://github.com/jamiw1">@jamiw1</a></td>
  </tr>
  <tr>
    <td><a href="https://github.com/FlameF0X">@FlameFox</a></td>
  </tr>
</table>

Crabby is a modern, **high-level**, **versatile**, **multi-paradigm**, general purpose language, with a hybrid approach!
It is designed to be *readable*, *ease-to-use* for beginners, and *powerful* enough for advanced users.

Crabby is leaning into *multi-paradigm*, supporting paradigms like object oriented and procedural programming.
But if you want the functional nature of Crabby, It is still a **Functional** programming language by default!

As you may notice, Crabby has been through a few stages of rewrites, if you're thinking that it is abandoned, think again!
**Crabby** has been entirely rewritten in **Rust**, benefitting from its memory safety and type checking!

## What's Changed?

1. A new Pythonic-style syntax but with a functional approach!
2. Runtime & Type checking is in the works!
3. Error handling is in the works!
4. More features!

## Installation

If you are on Linux and wish to use a pre-compiled binary of Crabby, you can skip right to step 4, and use the second option!

> [!TIP]
> For building from source, we highly recommend that you use the **nightly** version of Rust!

Step 1: Make sure you have `git` and `rust`/`cargo` installed and up-to-date.

```bash
git --version
cargo --version
```

Step 2: Clone the repository.

```bash
git clone https://github.com/crabby-lang/crabby.git
```

Step 3: Build it!

```bash
cargo build
```

Step 4: Test it for yourself!

```bash
cargo run examples/example.crab
```

OR

```bash
cd bin
./crabby ../examples/example.crab
```

## Syntax

**Crabby**'s syntax generally resembles **Python**, but there are some *hints* of functional programming syntax!

Crabby reserves the `.crab` and `.cb` file extensions, but for now only `.crab` is used!

helloworld.crab:

```js
print("Hello, World!")
```

example.crab:

```js
let x = 42
let y = 314
var z = 10 // You can use the 'var' keyword too!
let message = "Hello, Crabby!"

print(x)
print(y)
print(z)
print(message)
```

functions.crab:

```rs
pub def foo() {
    print("Hello!")
}

foo()
```

arrays.crab:

```rs
let numbers: [Int] = [1, 2, 3, 4, 5]

print(numbers[0])
print(numbers[4])

```

math.crab:

```js
// addition
let x1 = 1
let y1 = 2

// multiplication
let x2 = 4
let y2 = 7

// subtraction
let x3 = 10
let y3 = 3

// division
let x4 = 10
let y4 = 3

print(x1 + y1)
print(x2 * y2)
print(x3 - y3)
print(x4 / y4)
```

## **Data Types?**

datatypes.crab:

```js
// Lists of Data Types and Types in Crabby

String

Float

Int

bool

null
```

ifelse.crab:

```js
// if-else statements

let x = true
let y = false

if x {
    print("True!")
} else {
    print("Nope!")
}

```

loops.crab:

```py
let x = range(10)

for i in x {
    print(i) // Prints it 10 times
}

let y = 10

for i in range(y) {
    print(i)
}
```

structs.crab:

```rs
pub struct Person {
    name: String,
    age: Int,
    hobbies: String,
    birthday: Date,
}
```

enums.crab:

```rs
pub enum Colors {
    Red,
    Green,
    Pink,
    Purple,
    Blue,
    RGB(r: Int, g: Int, b: Int),
}
```

matching.crab:

```rs
match value {
    case 1 => print("one"),
    case _ => print("default"),
}

// Match multiple values:

let number = 5

match number {
    case 1 | 3 | 5 | 7  => print("This is a odd number"),
    case _ => print("Not an odd number")
}
```

Note: **Crabby** supports commenting, use `//` to comment out a code or leave a silly message :3

Speaking of comments, **Crabby** also support [`Docstrings`](https://www.geeksforgeeks.org/python/python-docstrings/)!

## FEATURES

> [!IMPORTANT]
> Many features listed here are still in development, and may not work or be missing components!

1. Asynchronous awaits
2. Inheritance and Polymorphism
3. Encapsulation
4. Composition (rust style)
5. Ability to do unsafe code (be careful though)
6. Try catch error handling
7. Python-style `finally` keyword
8. Import and exports (W.I.P)
9. Anonymous functions (with Lambdas)
10. Higher-order Functions
11. Pipe Operators (Elixir/Ruby style)
12. Type Safety
13. Pattern matching with `match` and `case`
14. The use of `extern` for using C code
15. `foreign` keyword to code non-crabby code inside crabby (can load shared libraries like DLLs/.so files and reuse it)

## FAQs

> `Q`: Is Crabby going to be the new Python or Rust?

Not really! This is just a hobby and fun project of mine, doesn't mean it's going to be the next big thing. But i build crabby because i love learning and experimenting how programming languages are made.

> `Q:` What problems Crabby will fix?

I'm going to very honest and be humble on this one, but Crabby will try to fix the problems that
languages face like complexity in their syntax and slow runtime/compile time.

Will it happen right now? Not yet, but could be, maybe one day you as the developer reading this
could contribute the future of Crabby 😎

> `Q`: Why does it have brackets and other non-Pythonic syntaxes if it aims to be one?

Well i didn't say it's going to be FULLY pythonic, there are hints of pythonic style syntaxes
but **Crabby** is multi-paradigm meaning you don't alway expect everything to be fully pythonic.
It is great for python users that wants a functional approach language.

## Package Manager📦

The closest package manager `Crabby` can have is [crab](https://github.com/crabby-lang/crab/)🦀 (which is still in development),
it functions likely the same as Rust **cargo** but for **Crabby**

## Contributing

Crabby is open to contributions! Feel free to open an issue or a pull request.
Make sure to read the [Contributing Guidelines](CONTRIBUTING.md) before getting started.

## LICENSE

Crabby is licensed under the GNU General Public License v3.0.

## Old Contributors ✨

Thanks goes to these wonderful people that used to help this project! 👨‍💻💻:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Satvik-2727"><img src="https://avatars.githubusercontent.com/u/87568817?v=4?s=100" width="100px;" alt="Mr.Coder"/><br /><sub><b>Mr.Coder</b></sub></a><br /><a href="https://github.com/crabby-lang/crabby/commits?author=Satvik-2727" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Scarleyegaming"><img src="https://avatars.githubusercontent.com/u/93965392?v=4?s=100" width="100px;" alt="Saturo"/><br /><sub><b>Saturo</b></sub></a><br /><a href="https://github.com/crabby-lang/crabby/commits?author=Scarleyegaming" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://tiramify.dev"><img src="https://avatars.githubusercontent.com/u/94789999?v=4?s=100" width="100px;" alt="Trnx"/><br /><sub><b>Trnx</b></sub></a><br /><a href="https://github.com/crabby-lang/crabby/commits?author=trnxdev" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

