use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime},
};

use crate::common::exec_stream;
use fs_extra::dir::get_size;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BuildIndex {
    index_config: String,
    data_path: String,
}

#[derive(Serialize, Deserialize)]
struct BuildIndicesConfig {
    indices: Vec<BuildIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Stats {
    indexing_time: Duration,
    index_size: u64,
    input_size: u64,
    compression_ratio: f32,
}

#[derive(Serialize, Deserialize)]
struct IndexConfig {
    index_id: String,
}

#[derive(Serialize, Deserialize)]
struct RunInfo {
    commit_hash: String,
    run_date: SystemTime,
    stats_per_index: Vec<Stats>,
}

impl RunInfo {
    pub(crate) fn new() -> _ {
        todo!()
    }
}

pub fn build_index_and_get_size(build_indices_config_path: Option<PathBuf>) -> std::io::Result<()> {
    let path = build_indices_config_path.unwrap_or("build_index.toml".into());

    let config: BuildIndicesConfig = toml::from_str(&fs::read_to_string(path)?).unwrap();

    if !Path::new("qwdata").exists() {
        fs::create_dir("qwdata")?;
    }

    let mut run_info = RunInfo::new();
    let mut stats_per_index = vec![];

    let qw_binary = "./quickwit/target/release/quickwit";

    for build_index_config in config.indices {
        let index_config: IndexConfig =
            serde_yaml::from_str(&fs::read_to_string(&build_index_config.index_config)?).unwrap();

        // Get Split ids

        let split_ids = fs::read_dir(format!("./qwdata/indexes/{}/", index_config.index_id))?
            .map(|path| {
                path.unwrap()
                    .path()
                    .file_stem()
                    //.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();

        //println!("{:?}", paths);

        for split_id in split_ids {
            let output = Command::new(qw_binary)
                .args([
                    "split",
                    "describe",
                    "--index",
                    &index_config.index_id,
                    "--config=quickwit.yaml",
                    "--split",
                    &split_id,
                    "--verbose",
                ])
                .output()
                .expect("failed to execute process");

            println!("{}", String::from_utf8_lossy(&output.stdout))
        }

        panic!("waaa");

        let output = Command::new("git")
            .args([qw_binary, "https://github.com/quickwit-oss/quickwit.git"])
            .output()
            .expect("failed to execute process");

        // delete index if it exists
        exec_stream(
            qw_binary,
            &[
                "index",
                "delete",
                "--index",
                &index_config.index_id,
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
                &index_config.index_id,
                "--config=quickwit.yaml",
                "--input-path",
                &build_index_config.data_path,
            ],
        );

        let duration = Instant::now() - start;

        let input_size = get_size(&build_index_config.data_path).unwrap();

        let index_size =
            get_size("./qwdata/indexes/".to_string() + &index_config.index_id).unwrap();
        let stats = Stats {
            indexing_time: duration,
            index_size,
            input_size,
            compression_ratio: index_size as f32 / input_size as f32,
        };

        stats_per_index.push((index_config.index_id, stats));
    }

    println!("{:?}", stats_per_index);

    Ok(())
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
