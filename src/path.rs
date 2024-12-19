pub mod metrics {
    use std::fs::{self};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use csv::Writer;
    use crate::serializer::MetricsSerializable;

    fn path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();

        path.push("target");
        path.push("metrics");

        path
    }

    pub fn mk_prefix(folder: &str) -> PathBuf {
        let mut path = path();
        path.push(folder);

        if !Path::exists(&path) {
            fs::create_dir_all(&path).unwrap();
        }

        path
    }

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
}
