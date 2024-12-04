use std::path::{Path, PathBuf};
use csv::Writer;
use crate::serializer::MetricsSerializable;

pub fn store(prefix: &Path, name: &str, metrics : &Vec<MetricsSerializable>) {

    let result_path = {
        let mut prefix = PathBuf::from(prefix);
        prefix.push(name);
        prefix
    };
    let mut wrt = Writer::from_path(result_path).unwrap();
    for metric in metrics {
        wrt.serialize(metric).unwrap();
    }
    wrt.flush().unwrap();
}