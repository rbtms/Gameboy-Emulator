#!/bin/bash
#
# Helper script to run tests
#

TESTS_TO_RUN=() # Array of tests to run
target_file=$1  # File containing the tests
target_test=$2  # Tests containing this word will be executed


# Gets a list of all available tests and filters those containing the first argument
function filter_tests() {
    filter=$1

    for test in $(cargo test -- --list 2>/dev/null | grep $filter); do
        if [[ "$test" =~ ^"tests::" ]]; then
            TESTS_TO_RUN+=(${test::-1})
        fi
    done
}


# Run specific test
if [ -n "$target_test" ]; then
    filter_tests $target_test
    
    cmd="cargo test --package gb --test $target_file -- ${TESTS_TO_RUN[@]} --exact --show-output --test-threads=1"
# Run all tests of a file
else
    cmd="cargo test --package gb --test $target_file -- --show-output --test-threads=1"
fi

echo cmd: $cmd
echo

$cmd
