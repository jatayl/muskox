# muskox

muskox is a stateful checkers engine written in rust that uses the minmax algorith with alpha-beta pruning. the evaluation function can either be a classical hand picked heuristic or NNUE (efficiently updated neural network). However, the NNUE is yet to be implemented!

## Usage

To build and run the binary, execute the following command in the terminal.

`$ cargo run --release`

This will yield another command line. Note that currently the command line interaction is unsafe and will fail ungracefully if an invalid command is inputted. This is due to my lack of time invested into error management for the command line interaction. It isn't a proble  if you type a valid command in though! There are several important commands that one can use to interact with the chess engine. Try the following.

`[0]: print`

An ascii version of a new checkers board will print to the console. From here we can take actions of the checkers board. Lowercase b's and w's represent black and white single pieces respectively while uppercase B's and W's represent black and white kings pieces. The initial turn is blacks. Run the following command to get all of the moves for the player with the current turn (in this case blacks).

`[1]: generate`

Outputted is a list of moves that black can make. They are all in the notation `POS1xPOS2x...xPOSN` where `POS1` is the initial position of a checker, `POSN` is the destination position of a checker and the positions in between are all of the intermediate placements of a piece if a jump is occuring. Additionally, the positions are numbered from the top left moving right on the board. To take a particular position run the command

`[2]: take POS`

where POS is a valid position to take. You could now `print` to see the updated board. Now let's say that you want to generate the best move to take given a particular position for the player of the current turn. Run the following command

`[3]: best`

It will output the best move to take that will maximize future return for the current player. There are several options one can use to customize this command.

`[4]: best timed 10000`

`[5]: best depth 20`

The 4th command will run the search command with a limit of 10000 milliseconds. The 5th command runs the search with a maximum depth of 20 moves into the future. A function to determine the current evaluation of the board exists.

`[6]: evaluate`

This will return a signed number that scores the board. The higher the number the better for black. Conversly, the lower the number, the better for white. An even board is represented as 0. The same customizations on the search that exist for the `best` command also exist for `evaluate`.

You have now learned the most important commands to interact with muskox! Below are some supplementary commands that are also useful.

* `fen STRING`: load a checker board state from a FEN string. Read more about formatting [here](https://en.wikipedia.org/wiki/Portable_Draughts_Notation).
* `gamestate`: retrieves the current state of the game. Will state a winner/draw or will print that the game is in progress
* `turn`: print the color of the player of the current turn
* `reset`: resets the checkers board to default position
* `exit`: terminates the muskox program

### Testing and benchmarking

To run tests, execute the following command

`$ cargo test`

To run benchmarks, execute the following command

`$ cargo bench`

## Overview of underlying implementation

### Checkers board architechture

The checkers boards are internally represented using Bitboards. This makes the implementation harder to understand but execution much faster.
* Blacks: 32 bit integer mask that represents all of the black positions. the i'th bit indicates the precence of black on the i'th square
* Whites: 32 bit integer like above but for white
* Kings: 32 bit integer like above but indicate presence of a king on the board
* turn: single byte represents current turn

### Action representation

Each action can be represented by a 32 bit integer. The usage breakdown is as follows
* 5 bits: source position
* 5 bits: destination position
* 5 bits: jump length. how many jumps are made (if any)
* 8 * 2 = 16 bits: jump directions (if any). there are four possible directions for each jump (TL, TR, BL, BR). Can store up to four different jumps
* 1 bit: unused

### Search Algorithm

The search algorith uses standard minmax with alpha beta pruning. Additionally, for the timed searched, iterative deepening depth first search is used to compute at different depths until we reach the time threshold. Currently, the search uses a single thread. I plan on sharding the search problem into subprograms of depth `d-1` and enable multiple threads to tackle each of the subproblems.

### Transposition Table

The transposition table is just a `std::sync::RwLock` wrapped around a `hashbrown::HashMap`. Read more about hashbrown [here](https://github.com/rust-lang/hashbrown). It's basically just a faster implementation of the rust standard library hashmap. The later plan is to implement it with zobrist hashing and clusters but my initial implementation was extremely weak so I git stashed it for later :)

### Evaluation Functions

The current evaluation function is extremely simple. It just counts pieces. One of my next goals is to consult checkers theory (of which I know none) and try to learn how to construct evaluation functions. Afterwards, I want to have an alternative NNUE evaluation function. I plan on training it on middepth analysis of boards. Really excited about this.