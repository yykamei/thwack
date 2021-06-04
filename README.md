# thwack

<a href="https://github.com/yykamei/thwack/actions/workflows/ci.yml"><img alt="GitHub Actions workflow status" src="https://github.com/yykamei/thwack/actions/workflows/ci.yml/badge.svg"></a>
<a href="https://crates.io/crates/thwack"><img alt="Crates.io" src="https://img.shields.io/crates/v/thwack"></a>

thwack is the Command-line utility similar to "Go To File" on GitHub. You can search for a file in a directory and invoke an arbitrary command on it.

## Why is this useful?

In many cases, we know the name of a file when we want to open it, but we may not be sure where it's located. In this case, we can search it with our file explorer. However, if we're in the terminal console, it's cumbersome to switch to another application. Besides, there might be few command-line tools to support this functionality, and we have to combine some commands with pipes to get things done.

thwack is the solution! You can find and open a file quickly with it 🚀

## Installation

If you have a Rust environment set up, you can use the `cargo install` command:

```console
cargo install thwack
```

Note we consider providing easier ways to install thwack. This should be used without the knowledge of Rust.

## Getting Started

Just run this command and type the name of the file you'd like to find.
By hitting the Enter key on the file you wanted in the list of the results, you can exeucte an arbitrary command on it.

```console
thwack
```

Run `thwack --help` for more options.

## Contributing

Thank you for considering contributing!

This project is so immature that you may wonder how to contribute.
Currently, all actions are welcome!
Open issues if you want to comment or ask something.
Open pull requests directly if you have any ideas.
There are many chances to improve this project, so don't hesitate to do something here 😄

Take a look at the [CONTRIBUTING.md](https://github.com/yykamei/thwack/blob/main/CONTRIBUTING.md), too.

## License

Copyright © 2021, Yutaka Kamei.

thwack is available under the terms of either the Apache License 2.0 or the MIT License, at your option.

See [LICENSE-APACHE](https://github.com/yykamei/thwack/blob/main/LICENSE-APACHE), [LICENSE-MIT](https://github.com/yykamei/thwack/blob/main/LICENSE-MIT) for details.
