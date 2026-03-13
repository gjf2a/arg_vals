# arg_vals

This is a simple library for parsing command-line arguments. 

It represents command-line arguments in two ways. Key-value relationships are set up using an `=` sign within the argument. Any other command-line argument is treated as a simple symbol.

On the right of the `=` sign, a value can be comma-separated, thus producing a duple value. This allows straightforward parsing of an argument formatted along the following lines:

`-position=3,4`