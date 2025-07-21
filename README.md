# Crabby: The Modern Programming Language Written in Rust

![Crabby](https://avatars.githubusercontent.com/u/139462470?s=48&v=4)

![GitHub contributors](https://img.shields.io/github/contributors/crabby-lang/crabby?style=for-the-badge&color=blue)

![GitHub License](https://img.shields.io/github/license/crabby-lang/crabby?style=for-the-badge&logo=gnu&logoColor=%23A42E2B)

## Introduction

![Logo](https://github.com/crabby-lang/crabby/blob/main/crabbylogo.png)

**NOTE⚠️**: **Crabby** is still under `development` with new features and bugs being fixed, if you encounter an error then that's OUR fault, and we're still **fixing** it. Crabby is **very new** due to the `rewrite update` and is still is experiencing bugs and errors. **You can help fixing Crabby if you want to.**

Crabby is a **Modern High-level**, **Versatile**, **Multi-paradigm**, **general-purpose**, and a **hybrid approach** programming language.
It is designed to be *readable*, *ease-to-use* for beginners, and *powerful* enough for advanced users.

Crabby is leaning into *multi-paradigm*, supporting paradigms like **OOP** and **Procedural** style of programming.
But if you want the functional nature of Crabby, It is still a **Functional** programming language by default!

## What Change?

As you may notice, Crabby has been through phases of rewrite, and if you're thinking that
**Crabby** is abounded, well... You're Wrong!

Crabby has been officially and well be written in **Rust** due to its memory safety and type checking compared to *C*.

Here's what changed:

1. It has a Pythonic-style syntax BUT with a functional approach.
2. It's purely functional (not yet for now).
3. JIT interpret/runtime.
4. Runtime & Type checking on the work.
5. Error handling on the work.
6. More parsing and features!

## Installation

`Note`: **We highly recommend to use the nightly toolchain edition of Rust.**

`Note (again):` Due to compatibility errors and issues, Crabby is currently being supported/targeted in **Linux**, meaning for *Windows users* we highly recommend having a WSL distro with rust in it just in case.

Step 1: Make sure to have `git` and `rust/cargo` installed

```bash
git --version
cargo --version
```

Step 2: Git clone

```bash
git clone https://github.com/crabby-lang/crabby.git
```

Step 3: Build it and Test it for yourself

```bash
cargo build
cargo run examples/example.crab
```

OR

```bash
cd bin
./crabby ../examples/example.crab
```

## Syntax

In **Crabby🦀**, its syntax mostly resembles **Python** in general BUT there are `hints` of Functional Programming syntax since Crabby aims to be versatile, functional and powerful to use not just for advanced programmers but for beginners also!

It's default file format is a `.crab` or `.cb`
But for now it's `.crab`

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

helloworld.crab:

```js
print("Hello, World!")
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

Note: **Crabby** supports commenting, use `//` to comment out a code or leave a silly message :3

Speaking of comments, **Crabby** also support [`Docstrings`](https://www.geeksforgeeks.org/python/python-docstrings/)!

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
