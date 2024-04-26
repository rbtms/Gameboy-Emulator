#!/bin/bash
#
# Helper script to run tests
#

target_file=$1 # May be test, blargg or mooneye
target_test=$2
args=""

# Run specific test
if [ -n "$target_test" ]; then
    args=" tests::$target_test --exact --show-output"
fi

cmd="cargo test --package gb --test $target_file --$args --test-threads=1"

echo cmd: $cmd
echo

$cmd
