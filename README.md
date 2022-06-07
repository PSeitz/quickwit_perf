# qw_build_index
Build an quickwit index and track some statistics about it.

The project will clone quickwit, compile it and use the provided `build_index.toml` to build some indices.

`cargo run --release -- --machine-name G513`

`cargo run --release -- --skip-quickwit-install true --machine-name G513`


### Results
Output of the runs will be stored in `db.json`, which is a list of json serialized`BuildIndexResult`.


### CLI Help


```
➜  track_index_size git:(master) cargo run --release -- --help
   Compiling track_index_size v0.1.0 (/home/pascal/LinuxData/Development/track_index_size)
    Finished release [optimized] target(s) in 1.67s
     Running `target/release/track_index_size --help`
Usage: track_index_size [--skip-quickwit-install <skip-quickwit-install>] [--build-indices-config-path <build-indices-config-path>] [--quickwit-commit-hash <quickwit-commit-hash>] --machine-name <machine-name>

Options to configure a run.

Options:
  --skip-quickwit-install
                    skip quickwit installation
  --build-indices-config-path
                    the path to the config to build indices. See
                    `BuildIndicesConfig` for parameters. Defaults to
                    `build_index.toml`.
  --quickwit-commit-hash
                    optional quickwit_commit_hash to checkout after cloning.
  --machine-name    the machine name. To differentiate different runners
                    committing into the same db.json.
  --help            display usage information
```
