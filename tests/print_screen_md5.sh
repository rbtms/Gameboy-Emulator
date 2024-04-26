#!/bin/bash
#
# Script to print the md5 of the screen state
# at emulator exit.
#

export GB_ROM_PATH=$1
cargo test --package gb --test print_screen_md5 -- tests::print_screen_md5 --exact --show-output
