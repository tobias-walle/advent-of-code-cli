# Advent of Code CLI

CLI client for advent of code.

The purpose of this tool is so you don't have to leave your editor while participating
(if you are lazy like me) or if you want to save seconds if you are trying to be competitive.

**Please do not use this tool for automated AI assisted submissions.**

This is actively [discouraged by the AoC team](https://adventofcode.com/about#faq_ai_leaderboard)
and against the spirit of AoC.

Please also be careful to not send too many requests.
There is a note about this in the html source code (view-source:https://adventofcode.com/):

> Please be careful with automated requests; I'm not a massive company, and I can
> only take so much traffic. Please be considerate so that everyone gets to play.\
> ...\
> \- Eric Wastl

Also consider [donating to the makers of AoC](https://adventofcode.com/support).

## Example

```sh
# You can get the session from the cookies of https://adventofcode.com
# It has a relatively long lifetime so you can set it in your .bashrc and co
export AOC_SESSION="<your-session>"
mkdir ./template # Feel free to add your code boilerplate in this folder
aoc new -y 2020 -d 1 # This will create the "day_1" folder and downloads the problem into it
# After you solved the problem
cd day_1
aoc submit -l 1
# Download the second problem
aoc download
# Submit it
aoc submit -l 2
```

## Installation

Install it with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```
cargo install --git https://github.com/tobias-walle/advent-of-code-cli
```

## Documentation

`aoc --help`:

````
Tool to automate downloading and submitting advent of code problems.

It requires the AOC_SESSION environment variable to be defined. You can get the session value from the cookies on the advent of code page.

You also need to define a `./template` folder which gets copied then creating a problem folder.

Usage: aoc [OPTIONS] <COMMAND>

Commands:
  new       Creates a new folder and downloads the problem into it
  download  Download the problem statement, input and example
  submit    Submit your result
  help      Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>
          The path of the configuration file. If defined you don't need to define the year and day in the commands. Example: ```toml year: 2020 day: 1 ``` It can contain the fields "year" and "day"

          [default: ./aoc.toml]

  -h, --help
          Print help (see a summary with '-h')
````
