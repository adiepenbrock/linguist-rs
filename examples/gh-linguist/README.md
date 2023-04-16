# gh-linguist example

This example demonstrates how to use [Linguist](https://github.com/github/linguist) 
definitions to identify programming languages.


## Prerequisities

To run this example, you must have local copies of `languages.yml` and `heuristics.yml` on your
machine. Additionally, to use the `matcher` feature, if you don't have it already, you can download
them using the following `curl` commands:

```
curl -o languages.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml
curl -o heuristics.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/heuristics.yml
curl -o vendors.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/vendor.yml
```

## How to run

This example tool can be used to determine the programming languages of a directory on your machine,
using the GitHub Linguist definitions. Besides the paths to both definition files, the tool expects
a path to a directory on your machine. You can run the tool by executing the following command:

```
LANGUAGE_DEF_PATH=/path/to/languages.yml HEURISTIC_DEF_PATH=/path/to/heuristics.yml VENDOR_DEF_PATH=/path/to/vendors.yml cargo run /path/to/directory
```

Make sure to replace `/path/to/languages.yml`, `/path/to/heuristics.yml`, and `/path/to/vendors.yml` with the actual paths to
these files on your machine.
