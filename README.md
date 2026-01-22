# langs-detector

A simple and not optimized algorithm to detect the lang of a given sentence

## Rust version

The project has been rewrote in rust, but it will not be merged into the main branch

## Installation

1. Clone the repository ( `git clone https://github.com/Greensky-gs/langs-detector` )
1. Open a terminal and navigate to the cloned repository
1. Install cargo if you don't have it
1. If you don't have MySQL installed, intall it ( I personnaly use [XAMPP](https://www.apachefriends.org/) )
1. Create a database named `langs`
1. Fill the .env file with the database url ( something like `mysql://user:password@localhost:port/langs` )
1. Compile the code using `cargo build` ( or the provided makefile : `make dev` or `make release` )
1. Run the created executable (probably in `target/debug` or `target/release`

## Specifications

The algorithm is a KNN algorithm

## Contact

If you have any question, you can join the [discord server](https://discord.gg/fHyN5w84g6), ask me on [instagram](https://instagram.com/draverindustries) or by mail at `draver.industries@proton.me`
