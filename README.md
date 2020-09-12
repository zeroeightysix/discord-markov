# discord-markov
A rust program to extract messages from a discord data package, and then produce markov chains from it

## Building
Using cargo:
`cargo build --release`

Produced binary is in `target/release`

## Usage
Start by creating the message dump.

`discord-markov` assumes the current working directory is the `messages` directory of your discord data package.
Without any arguments, the program will just extract all the messages it can find, and write them to standard output. Write that to a file:

```shell
cd my_data_package/messages
discord-markov > messages
```

Then, run it again, but with the input file and amount of markov chains to produce as arguments.
```shell
discord-markov messages 50
```
