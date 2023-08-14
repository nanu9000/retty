# Retty
Initial learnings at handling connections (just TCP for now) via Rust.

### Usage
```
# Terminal 1
cargo run

# Terminal 2
./scripts/bench.sh
```

### TODOs (not the ones in the code)
* Impl with a non-blocking TcpListener
* Add some observability into each process spawned in the Bash script (might have to spawn the threads via another Rust program)
* Actually benchmark
