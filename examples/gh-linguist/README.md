# gh-linguist example

This example demonstrates how to use [Linguist](https://github.com/github/linguist) 
definitions to identify programming languages.

## How to run

This example tool can be used to determine the programming languages of a directory on your machine,
using the GitHub Linguist definitions. The tool uses the `linguist-rs-build` crate to generate the 
language definitions based on the corresponding github repository. Furthermore, the tool expects a 
path to a directory on your machine. You can run the tool by executing the following command:

```
cargo run /path/to/directory
```