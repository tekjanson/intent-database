use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const MAX_LINE_LEN: usize = 100;
const MAX_FILE_LINES: usize = 300;

fn main() {
    let root = Path::new(".");
    let mut violations = Vec::new();
    visit_dirs(root, &mut |path| {
        if let Some(ext) = path.extension() {
            if ext == "rs" {
                if let Err(errs) = check_file(path) {
                    for e in errs {
                        violations.push((path.to_path_buf(), e));
                    }
                }
            }
        }
    })
    .unwrap();

    if violations.is_empty() {
        println!("Lint: no violations found");
        std::process::exit(0);
    }

    eprintln!("Found {} lint violations:", violations.len());
    for (path, msg) in violations {
        eprintln!("{}: {}", path.display(), msg);
    }
    std::process::exit(1);
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) -> io::Result<()> {
    if dir.ends_with("target") || dir.ends_with(".git") {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // skip target and .git
            if path.ends_with("target") || path.ends_with(".git") {
                continue;
            }
            visit_dirs(&path, cb)?;
        } else {
            cb(&path);
        }
    }
    Ok(())
}

fn check_file(path: &Path) -> Result<(), Vec<String>> {
    let file = File::open(path).map_err(|e| vec![format!("open error: {}", e)])?;
    let reader = io::BufReader::new(file);
    let mut errs = Vec::new();
    let mut line_count: usize = 0;
    for (idx, line_res) in reader.lines().enumerate() {
        line_count = idx + 1;
        let line = match line_res {
            Ok(l) => l,
            Err(e) => {
                errs.push(format!("line {} read error: {}", idx + 1, e));
                continue;
            }
        };
        let char_count = line.chars().count();
        if char_count > MAX_LINE_LEN {
            errs.push(format!("line {}: length {} > {}", idx + 1, char_count, MAX_LINE_LEN));
        }
    }
    if line_count > MAX_FILE_LINES {
        errs.push(format!("file has {} lines > {}", line_count, MAX_FILE_LINES));
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
