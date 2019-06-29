# Formality

A upcoming efficient proof-gramming language. It aims to be:

- **Fast:** no garbage-collection, optimal beta-reduction and a massively parallel GPU compiler make it fast.

- **Safe:** a type system capable of proving mathematical theorems about its own programs make it secure.

- **Portable:** the full language is implemented in a 400-LOC runtime, making it easily available everywhere.

Formality isn't ready yet, but you can already use [Formality-Core](https://github.com/moonad/formality-core), our low-level compile target ([check its wiki!](https://github.com/moonad/formality-core/wiki)), and [EA-TT](https://github.com/moonad/elementary-affine-type-theory), our underlying proof language.

## Projects

For a better separation of concerns, Formality was broken down into sub-projects:

Features | **Calculus** | **Type-Theory** | **Runtime**
--- | --- | --- | ---
Lam, Box | [EA-Core](EA-Core) | [EA-TT](EA-TT) | [EA-Net](EA-Net)
Lam, Box, Pair, Uint32 | [FM-Core](FM-Core) | FM-TT | [FM-Net](FM-Net)

`FM-Core` is our low-level, untyped compile target, `FM-TT` is our raw type theory and `FM-Net` is our efficient interaction-net runtime. `EA-Core`, `EA-TT` and `EA-Net` are their formalization-friendly counterparts, excluding native ints to be easier to reason about. Formality will be a high-level, Python/Agda-inspired syntax built on top of those. [Here is a sneak peek!](https://gist.github.com/MaiaVictor/489a4119efd49f16605f8d4d09d421ad)

### Progress Table

Note: most reference implementations were written in **JavaScript** simply because it is a cross-platform language, but we plan to re-implement all those things inside **Formality-Core**. Once this is done, we will port them Haskell, Rust, Python etc. by a simple bootstrapping process: 1. implement just the runtime ([FM-Net, a 400-LOC file](https://github.com/moonad/Formality-Core/blob/master/javascript/fm-to-net.js)), 2. load the desired lib on it.

Project | Description | Implementation(s)
--- | --- | ---
EA-Core | Parser, interpreter. | [JavaScript](EA-Core/README.md) 
EA-Core | EA-Net compiler/decompiler. | [JavaScript](https://github.com/moonad/Elementary-Affine-Core/blob/master/javascript/ea-to-net.js)
EA-Core | Command-line interface. | [JavaScript](https://github.com/moonad/Elementary-Affine-Core/blob/master/javascript/main.js)
EA-Core | Formalization. | [Agda (ongoing)](https://github.com/moonad/Elementary-Affine-Core/blob/master/agda/Linear.agda)
EA-Core | Specification. | [Markdown (ongoing)](https://github.com/moonad/Elementary-Affine-Core/blob/master/spec.md)
EA-Net | Strict and Lazy runtime. | [JavaScript](https://github.com/moonad/Elementary-Affine-Net/blob/master/javascript/ea-net.js)
EA-TT | Parser, interpreter, type-checker. | [JavaScript](https://github.com/moonad/Elementary-Affine-Type-Theory/blob/master/javascript/ea-tt.js)
EA-TT | EA-Core compiler/decompiler. | [JavaScript](https://github.com/moonad/Elementary-Affine-Type-Theory/blob/master/javascript/ea-tt.js)
EA-TT | Command-line interface. | [JavaScript](https://github.com/moonad/Elementary-Affine-Type-Theory/blob/master/javascript/main.js)
EA-TT | Specification. | [Markdown (ongoing)](https://github.com/moonad/Elementary-Affine-Type-Theory/blob/master/spec.md)
FM-Core | Parser, interpreter. | [JavaScript](https://github.com/moonad/Formality-Core/blob/master/javascript/fm-core.js), [Formality-Core (ongoing)](https://github.com/moonad/Formality-Core/blob/master/examples/term.fmc)
FM-Core | FM-Net compiler/decompiler. | [JavaScript](https://github.com/moonad/Formality-Core/blob/master/javascript/fm-to-net.js)
FM-Core | JS compiler/decompiler. | [JavaScript](https://github.com/moonad/Formality-Core/blob/master/javascript/fm-to-js.js)
FM-Core | Command-line interface. | [JavaScript](https://github.com/moonad/Formality-Core/blob/master/javascript/main.js)
FM-Core | Documentation. | [Markdown (ongoing)](https://github.com/moonad/formality-core/wiki)
FM-Net | Strict and Lazy runtime. | [JavaScript](https://github.com/moonad/Formality-Net/blob/master/javascript/fm-net.js), [C (ongoing)](https://github.com/moonad/Formality-Net/blob/master/c/fm-net.c), [OpenCL (redo)](https://github.com/MaiaVictor/absal-rs/blob/parallel-test-3/src/main.rs), [CUDA (redo)](https://github.com/moonad/Formality/blob/nasic-optimization/cuda/main.cu)
FM-Net | Documentation. | [Markdown (ongoing)](https://github.com/moonad/formality-core/wiki/Formality-Net)
FM-TT | Parser, interpreter, type-checker. | Formality-Core (todo)
FM-TT | FM-Core compiler/decompiler. | Formality-Core (todo)
FM-TT | Specification. | Markdown (todo)
FM-Lang | Parser, interpreter, type-checker. | Formality-Core (todo)
FM-Lang | Documentation | Markdown (todo)
LIB | Mutable Arrays. | [Formality-Core](https://github.com/moonad/Formality-Core/blob/master/examples/array.fmc)
LIB | Linked lists. | [Formality-Core](https://github.com/moonad/Formality-Core/blob/master/examples/list.fmc)
LIB | UTF-8 strings. | [Formality-Core](https://github.com/moonad/Formality-Core/blob/master/examples/array.fmc)
LIB | Demo: numeric algorithms. | [Formality-Core](https://github.com/moonad/Formality-Core/blob/master/examples/num.fmc)
LIB | Demo: theorems and proofs. | [Elementary Affine Type Theory](https://github.com/moonad/Elementary-Affine-Type-Theory/blob/master/main.eatt)
