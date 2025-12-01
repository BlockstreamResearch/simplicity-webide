# SimplicityHL Web IDE

[SimplicityHL](https://github.com/BlockstreamResearch/simplicityhl) is a high-level language for writing Bitcoin smart contracts.

SimplicityHL looks and feels like [Rust](https://www.rust-lang.org). Just how Rust compiles down to assembly language, SimplicityHL compiles down to Simplicity bytecode. Developers write SimplicityHL, full nodes execute Simplicity.

[A live demo is running](https://ide.simplicity-lang.org).

This project should build on Rust **1.78.0**.

## Develop the project

### Using Docker

Build
```
docker build -t simplicity-webide .
```

Run and naviate to `http://localhost:8080` in your web browser.
```
docker run -p 8080:80 simplicity-webide
```

### Using Nix

First install nix.

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

_You might need to open a new terminal for the changes to take effect._

Then enter the nix developer environment.

```bash
nix develop
```

Now you can use all `just` commands.

```bash
just --list
```

### Deploy a local website

Compile the website and serve it on localhost.

```bash
just serve
```

You can instruct the compiler to open the website on the default browser.

```bash
just open
```

The compiler will keep running in the background and make live updates to the website as you change the code.

### Run CI

Run a single command to run all CI jobs.

```bash
just check
```

There are `just` commands that can help you fix broken code.

```bash
just --list
```
