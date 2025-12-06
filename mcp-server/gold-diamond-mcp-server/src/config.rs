use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppState {
    pub reference_root: PathBuf,
    pub tests_out_root: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let reference_root = resolve_reference_root();
        let tests_out_root = std::env::var("DEX_OS_TESTS_OUT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./tests_out"));

        Self {
            reference_root,
            tests_out_root,
        }
    }
}

fn resolve_reference_root() -> PathBuf {
    if let Ok(value) = std::env::var("DEX_OS_REFERENCE_ROOT") {
        let path = PathBuf::from(&value);
        if path.exists() {
            return path;
        }
    }

    let candidates = [
        PathBuf::from("../DEX-OS-V2/.reference"),
        PathBuf::from("../../DEX-OS-V2/.reference"),
        PathBuf::from("../.reference"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from("../DEX-OS-V2/.reference")
}
