use std::{
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use crate::common::exec_stream;
use chrono::{DateTime, Utc};
use fs_extra::dir::get_size;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIndexResult {
    index_id: String,
    indexing_time_ms: u64,
    index_size_in_bytes: u64,
    input_size_in_bytes: u64,
    // The input size, vs the total output size, with all indices. TODO: Only compare docstore
    compression_ratio: f32,
    throughput_mbs: f32,
    // TODO add sizes of fast field docstore etc.
    split_info: SplitDetails,
    commit_hash: String,
    run_date_ts: i64,
    run_date: String,
    machine_name: String,
    rustc_version: String,
}

#[derive(Serialize, Deserialize)]
struct BuildIndex {
    // Optional custom name for the run
    name: Option<String>,
    index_config: String,
    data_path: String,
}

#[derive(Serialize, Deserialize)]
struct BuildIndicesConfig {
    indices: Vec<BuildIndex>,
}

#[derive(Serialize, Deserialize)]
struct IndexConfig {
    index_id: String,
}

fn get_rustc_version() -> String {
    let output = Command::new("rustc")
        .args(["--version"])
        .output()
        .expect("failed to execute process");

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn build_index_and_get_size(
    build_indices_config_path: PathBuf,
    machine_name: &str,
    commit_hash: &str,
) -> std::io::Result<()> {
    let config: BuildIndicesConfig =
        toml::from_str(&fs::read_to_string(build_indices_config_path)?).unwrap();

    let qw_data_path = "./qwdata";
    let qw_data_index_path = format!("{}/indexes", qw_data_path);

    let rustc_version = get_rustc_version();

    if !Path::new(qw_data_path).exists() {
        fs::create_dir(qw_data_path)?;
    }

    let qw_binary = "./quickwit/target/release/quickwit";

    let run_date: DateTime<Utc> = Utc::now();

    let mut build_index_results = vec![];

    for build_index_config in config.indices {
        let index_config: IndexConfig =
            serde_yaml::from_str(&fs::read_to_string(&build_index_config.index_config)?).unwrap();

        let index_id = &index_config.index_id;

        // delete index if it exists
        exec_stream(
            qw_binary,
            &[
                "index",
                "delete",
                "--index",
                index_id,
                "--config=quickwit.yaml",
            ],
        );

        // create index
        exec_stream(
            "./quickwit/target/release/quickwit",
            &[
                "index",
                "create",
                "--index-config",
                &build_index_config.index_config,
                "--config=quickwit.yaml",
            ],
        );
        let start = Instant::now();

        // ingest
        exec_stream(
            "./quickwit/target/release/quickwit",
            &[
                "index",
                "ingest",
                "--index",
                index_id,
                "--config=quickwit.yaml",
                "--input-path",
                &build_index_config.data_path,
            ],
        );

        let duration = Instant::now() - start;
        let input_size = get_size(&build_index_config.data_path).unwrap();
        let split_info = get_details_from_splits(qw_binary, &qw_data_index_path, index_id)?;

        let index_size = get_size(format!("{}/{}", qw_data_index_path, index_id)).unwrap();
        let build_index_result = BuildIndexResult {
            index_id: build_index_config.name.unwrap_or(index_config.index_id),
            indexing_time_ms: duration.as_millis() as u64,
            index_size_in_bytes: index_size,
            input_size_in_bytes: input_size,
            compression_ratio: index_size as f32 / input_size as f32,
            throughput_mbs: (input_size as f32) / 1_000_000.0 / duration.as_secs_f32(),
            commit_hash: commit_hash.to_string(),
            run_date_ts: run_date.timestamp(),
            run_date: run_date.to_string(),
            split_info,
            machine_name: machine_name.to_string(),
            rustc_version: rustc_version.to_string(),
        };

        build_index_results.push(build_index_result);
    }

    let mut db_json = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("db.json")?,
    );

    for build_index_result in build_index_results {
        let json = serde_json::to_string(&build_index_result).unwrap();
        db_json.write_all(json.as_bytes())?;
        db_json.write_all("\n".as_bytes())?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SplitDetails {
    num_splits: usize,
}

fn get_details_from_splits(
    qw_binary: &str,
    qw_data_index_path: &str,
    index_id: &str,
) -> io::Result<SplitDetails> {
    let split_ids = get_split_ids(&qw_data_index_path, &index_id)?;

    // Todo parse and aggregate fast, fieldnorm, idx, pos, store and hotcache sizes
    for split_id in &split_ids {
        let _output = describe_split(qw_binary, split_id, &index_id);
        //println!("{}", output);
    }

    Ok(SplitDetails {
        num_splits: split_ids.len(),
    })
}

fn get_split_ids(qw_data_index_path: &str, index_id: &str) -> io::Result<Vec<String>> {
    let splits = fs::read_dir(format!("{}/{}/", qw_data_index_path, index_id))?
        .map(|path| {
            path.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();

    Ok(splits)
}

fn describe_split(qw_binary: &str, split_id: &str, index_id: &str) -> String {
    let output = Command::new(qw_binary)
        .args([
            "split",
            "describe",
            "--index",
            &index_id,
            "--config=quickwit.yaml",
            "--split",
            &split_id,
            "--verbose",
        ])
        .output()
        .expect("failed to execute process");

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser_test() {
        let config: BuildIndicesConfig = toml::from_str(
            r#"

        [[indices]]
        data_path = 'a.json'
        index_config = 'a.yaml'

        [[indices]]
        data_path = 'b.json'
        index_config = 'b.yaml'

    "#,
        )
        .unwrap();

        assert_eq!(config.indices.len(), 2);
        assert_eq!(config.indices[0].data_path, "a.json");
        assert_eq!(config.indices[0].index_config, "a.yaml");
        assert_eq!(config.indices[1].data_path, "b.json");
        assert_eq!(config.indices[1].index_config, "b.yaml");
    }
}
