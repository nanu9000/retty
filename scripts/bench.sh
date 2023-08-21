#!/bin/bash

# As a fairly janky test, this runs curl sequentially and in parallel 5 times respectively.
# If you see COMPLETED! 5 times, then each call successfully returned.
# TODO Figure out why the output of parallel_calls isn't jumbled
#  up for blocking_listener_primitive or blocking_listener_spawns_threads.

sequential_calls() {
    for i in {1..50}; do
        curl localhost:3000/$i 2>/dev/null
    done
}

parallel_calls() {
    for i in {1..50}; do
        curl localhost:3000/$i 2>/dev/null &
    done
}

# echo SEQUENTIAL CALLS
# echo $(sequential_calls)

echo -e "\n\nPARALLEL CALLS"
echo $(parallel_calls)
