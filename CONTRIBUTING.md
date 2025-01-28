# Contributing to Crabby

Welcome to the Contributing Guidelines for Crabby!
Here, you can learn how to contribute to the `development` of Crabby.

## How Crabby is Build

Crabby is built in **Rust**, using crates like `logos`, `clap`, and more. Rust memory safety and
security, not to mention its *speed*, is appropriate to build languages like **Crabby**.

You also make sure to have the Rust **nightly toolchain** installed, if not yet you can run:

```bash
rustup toolchain install nightly
```

Then make it **default**:

```bash
rustup default nightly
```

## How to Contribute to Crabby?

We welcome a wide variety of contributions. Here's how you can get started:

### 1. Fork Crabby

Start by [forking the repository](https://github.com/crabby-lang/crabby/fork) to your own GitHub account.

### 2. Clone Crabby

Clone your `fork` locally so you can begin making changes.

```bash
git clone https://github.com/crabby-lang/crabby.git
cd crabby
```

### 3. Create a New Branch

For each new feature, bug fix, or improvement, create a separate branch to keep your changes organized.

```bash
git checkout -b feature/my-new-feature
```

### 4. Commit Your Changes

Make sure to test your changes thoroughly and follow the project's coding standards. Once ready, commit your changes.

```bash
git add .
git commit -m "Description of the changes"
```

### 5. Push to Your Fork and Submit a Pull Request

Push your changes to your forked repository and submit a pull request to the main LunarDB repository.

```bash
git push origin feature/my-new-feature
```

### 6. Review Process

In Crabby, we review your changes or features carefully, making sure no **malicious** or **sneaky**
code is written to the language, it may take up to a couple of minutes to hours, or in some cases, days.
