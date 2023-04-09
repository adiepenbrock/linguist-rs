# gh-linguist example

This example demonstrates how to use [Linguist](https://github.com/github/linguist) language 
definitions located in [languages.yml](https://github.com/github/linguist/blob/master/lib/linguist/languages.yml), 
and [heuristics.yml](https://github.com/github/linguist/blob/master/lib/linguist/heuristics.yml)
files to identify programming languages.


## Prerequisities

To run this example, you must have local copies of `languages.yml` and `heuristics.yml` on your
machine. Additionally, to use the `matcher` feature, if you don't have it already, you can download
them using the following `curl` commands:

```
curl -o languages.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml
curl -o heuristics.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/heuristics.yml
```

## How to run

This example tool can be used to determine the programming language of a specific file on your machine,
using the GitHub Linguist definitions. Besides the paths to both definition files, the tool expects
a path to a file on your machine. You can run the tool by executing the following command:

```
LANGUAGE_DEF_PATH=/path/to/languages.yml HEURISTIC_DEF_PATH=/path/to/heuristics.yml cargo run /path/to/file
```

Make sure to replace `/path/to/languages.yml` and `/path/to/heuristics.yml` with the actual paths to
these files on your machine.
