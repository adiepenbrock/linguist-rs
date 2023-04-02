# gh-linguist example

This example demonstrates how to use [Linguist](https://github.com/github/linguist) language 
definitions located in [languages.yml](https://github.com/github/linguist/blob/master/lib/linguist/languages.yml)
file to identify the programming languages of files.


## Prerequisities

To run this example, you must have a local copy of the `languages.yml` file on your machine. If 
you don't have it already, you can download it using the following `curl` command:

```
curl -o languages.yml https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml
```

## How to run

To run the example, execute the following command in your terminal:

```
LANGUAGE_DEF_PATH=/path/to/languages.yml cargo run 
```

Make sure to replace `/path/to/languages.yml` with the actual path to the `languages.yml` file on 
your machine.
