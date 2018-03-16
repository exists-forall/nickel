# The Nickel Target Language

Nickel is an experimental programming language intended to be used as a compile
target for high-performance functional languages.  Its main features are:

- Complete **type and memory safety** verified at compile-time.  Nickel programs
  cannot trigger undefined behavior except by linking against unsafe external
  libraries.
- Robust **functional purity** guarantees.  Nickel functions are deterministic
  and capture all output via return values.
- **Predictable performance** and precise control over **memory management.**
  Nickel can support languages without a garbage collector or even a heap
  allocator.
- Efficient **in-place mutation** of data structures, while ensuring functional
  purity via linear types.
- An **expressive type system** which includes first-class support for
  arbitrary-rank universal quantification, existential quantification, and
  generalized algebraic data types.

Nickel is currently in an extremely early stage of development.  At the moment,
it consists mainly of a preliminary design and a prototype compiler under
construction. Eventually, the Nickel toolchain will include:

- A Nickel **compiler** targeting **LLVM** and **WebAssembly**
- A standalone Nickel **type checker**
- Utilities for converting between multiple **equivalent representations** of
  Nickel programs, including:
  - A **human readable**, human writeable plain text format resembling a
    high level functional programming language
  - A **JSON** representation which can be easily generated or parsed from any
    language.
  - A compact **binary** format suitable for storing and distributing Nickel
    code.
- An optional **standard library** providing type and memory safe wrappers
  around platform funtionality like memory management and IO.

Questions and contributions are welcome!  Please contact [William
Brandon](https://github.com/selectricsimian/) to learn more.
