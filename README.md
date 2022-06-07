# qw_build_index
Build an quickwit index and track some statistics about it.

The project will clone quickwit, compile it and use the provided `build_index.toml` to build some indices.

`cargo run --release -- --machine-name G513`

`cargo run --release -- --skip-quickwit-install true --machine-name G513`


### Results
Output of the runs will be stored in `db.json`, which is a list of json serialized`BuildIndexResult`.
