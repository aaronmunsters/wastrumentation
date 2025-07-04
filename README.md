# Wastrumentation

[![DOI](https://img.shields.io/badge/DOI-10.4230%2FLIPIcs.ECOOP.2025.23-blue)](https://doi.org/10.4230/LIPIcs.ECOOP.2025.23)

![The Wastrumentation Architecture](./media/architecture.svg)

Portable WebAssembly Dynamic Analysis with Support for Intercession

## About
Wastrumentation is a dynamic analysis platform for WebAssembly that supports intercession.
Wastrumentation is based on source code instrumentation, which weaves the analysis code directly into the target program code.
Inlining the analysis into the target’s source code avoids dependencies on the runtime environment, making analyses portable across different Wasm VMs.
Wastrumentation's approach enables the implementation of analyses in any Wasm-compatible language.

The architecture figure above describes the inner workings of Wastrumentation.
Writing the input analysis directly in WebAssembly is tedious, this repository includes two DSLs that allow you to write your analysis in a high-level language, one for [Rust](https://www.rust-lang.org/) and one for [AssemblyScript](https://www.assemblyscript.org/).

## Usage

Wastrumentation's dependencies can update over time.
The CI file (in `.github/workflows`) installs the dependencies and verifies all tests still pass.
Consider it the source of truth for the dependencies.
The dependencies are [NodeJS](https://nodejs.org/), `wasm-merge` (from [binaryen](https://github.com/WebAssembly/binaryen)), and [Rust + Cargo](https://www.rust-lang.org/).

The following describes the steps to execute Wastrumentation as a command-line executable:
```bash
# Verify dependencies are present
$ node --version && wasm-merge --version && rustc --version && cargo --version

# Clone the repository
$ git clone https://github.com/aaronmunsters/wastrumentation.git
$ cd wastrumentation

# Execute Wastrumentation
$ cargo run --bin wastrumentation-cli                     \
    --input-program-path <INPUT_PROGRAM_PATH>             \
    --rust-analysis-toml-path <RUST_ANALYSIS_TOML_PATH>   \
    --output-path <OUTPUT_PATH>
```

You can view example analyses developed in Rust [here](./wastrumentation-instr-lib/tests/analyses/rust) and those developed in AssemblyScript [here](./wastrumentation-instr-lib/tests/analyses/wasp-as).

## Publication Reference
This platform and the related research was published at ECOOP 2025:

- [Wastrumentation: Portable WebAssembly Dynamic Analysis with Support for Intercession](https://doi.org/10.4230/LIPIcs.ECOOP.2025.23)

### Citation (BibTeX)

Use the following BibTeX entry to cite this work:

```
@InProceedings{munsters_et_al:LIPIcs.ECOOP.2025.23,
  author =	{Munsters, A\"{a}ron and Scull Pupo, Angel Luis and {Gonzalez Boix}, Elisa},
  title =	{{Wastrumentation: Portable WebAssembly Dynamic Analysis with Support for Intercession}},
  booktitle =	{39th European Conference on Object-Oriented Programming (ECOOP 2025)},
  pages =	{23:1--23:29},
  series =	{Leibniz International Proceedings in Informatics (LIPIcs)},
  ISBN =	{978-3-95977-373-7},
  ISSN =	{1868-8969},
  year =	{2025},
  volume =	{333},
  editor =	{Aldrich, Jonathan and Silva, Alexandra},
  publisher =	{Schloss Dagstuhl -- Leibniz-Zentrum f{\"u}r Informatik},
  address =	{Dagstuhl, Germany},
  URL =		{https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2025.23},
  URN =		{urn:nbn:de:0030-drops-233153},
  doi =		{10.4230/LIPIcs.ECOOP.2025.23},
  annote =	{Keywords: WebAssembly, dynamic analysis, instrumentation platform, intercession}
}
```
