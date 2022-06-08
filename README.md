# Quickwit Build Index

Tool to build quickwit indices, record some stats and put them into a file database `db.json`.

### How to use


1. Configure `build_index.toml`, which contains path to the data, path to the config, and _optionally_ an name:

The name or as fallback the index_id is used as "index_id" in the result stored in db.json.

```toml
[[indices]]
data_path = 'hdfs-log.json'
index_config = 'hdfs_index_config.yaml'
name = 'hdfs'
```


2. 
The tool will clone and compile quickwit. This can be skipped.

Examples:
```
cargo run --release -- --machine-name c5n.2xlarge

cargo run --release -- --machine-name c5n.2xlarge --quickwit-commit-hash 5e200a3

cargo run --release -- --skip-quickwit-install --machine-name c5n.2xlarge
```

3. The compiled quickwit binary will be used to create the indices, gather some data. The results are appended to `db.json`.


### Results
Output of the runs will be stored in `db.json`, which is a list of json serialized `BuildIndexResult`.


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
