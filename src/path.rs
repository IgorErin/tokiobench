pub mod metrics {
    use csv::Writer;
    use serde::Serialize;
    use std::fs;
    use std::path::{Path, PathBuf};

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

    pub fn store_vec(prefix: &Path, name: &str, metrics: &[impl Serialize]) {
        let result_path = {
            let mut prefix = PathBuf::from(prefix);
            prefix.push(name);
            prefix
        };

        let mut wrt = Writer::from_path(result_path).unwrap();
        for m in metrics.iter() {
            wrt.serialize(m).unwrap()
        }
        wrt.flush().unwrap();
    }
}
