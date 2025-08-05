# c2e — C → English translator

[![GitHub Actions Workflow Status bagde](https://img.shields.io/github/actions/workflow/status/kdkasad/c2e/ci.yml?logo=github&label=CI)](https://github.com/kdkasad/c2e/actions/workflows/ci.yml)
[![Codecov badge](https://img.shields.io/codecov/c/github/kdkasad/c2e?logo=codecov&color=%23F01F7A)](https://app.codecov.io/gh/kdkasad/c2e)
[![License badge](https://img.shields.io/github/license/kdkasad/c2e?color=blue)](COPYING)
[![Crates.io Version badge](https://img.shields.io/crates/v/c2e?logo=rust)](https://crates.io/crates/c2e)



c2e is a library, CLI tool, and [website](https://c2e.kasad.com) that can translate C type/variable declarations into English explanations.

```
$ c2e
> char *const (*(*foo)(const int *[]))[3]
a pointer named foo to a function that takes (an array of pointers to const ints)
and returns a pointer to an array of 3 const pointers to chars
```

## Usage

### Website

Head to <https://c2e.kasad.com> and start typing!
The site requires JavaScript and WASM support from your browser, as all translation happens in your browser.

### CLI

c2e's CLI tool provides the same capabilities as the website, but in a simple UNIX-style CLI.

#### Installation (pre-compiled binary)

Download the binary for your platform from the [GitHub repository's release page](https://github.com/kdkasad/c2e/releases).

#### Installation (from source using Cargo)

Ensure you have [Cargo](https://doc.rust-lang.org/cargo/index.html) installed, then run one of the following commands.

```
# latest release
$ cargo install c2e-cli

# latest development version 
$ cargo install --git https://github.com/kdkasad/c2e c2e-cli
```

#### Using the CLI

Run `c2e` to enter the interactive interface.
Enter declarations and the explanations will be printed to the standard output stream.
Multiple declarations can be entered on the same line, separated by semicolons.

Pipes can also be used to direct input/output from/to other programs, just like with standard UNIX tools.

Output will be colored if being printed to a terminal that supports color. Piped output will not be colored.

## Copyright/license

c2e is written and copyrighted by [Kian Kasad].
It is made a available under the terms of the [GNU General Public License, version 3](COPYING).

[Kian Kasad]: https://github.com/kdkasad

## Contributing

All contributions submitted for inclusion in this project will licensed as described above.
By submitting a contribution, you agree to make it available under the terms of the license.
