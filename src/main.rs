use std::env;
use std::fs;

fn main() {
    let mut path = match env::args().nth(1) {
        Some(path) => path,
        None => String::from(""),
    };

    if path.eq("") {
        let current_dir_result = match env::current_dir() {
            Ok(cur) => cur,
            Err(err) => panic!("unable to get current directory: {err:?}"),
        };
        let current_dir_buf = current_dir_result.to_string_lossy().to_string();
        path = String::from(current_dir_buf.trim_ascii_end().trim_ascii_start());
    }

    let entries = list_current_directory(&path);

    for entry in entries {
        println!("{}", entry);
    }
}

fn list_current_directory(path: &str) -> Vec<String> {
    let dir_result = fs::read_dir(path);
    let dir = match dir_result {
        Ok(dir) => dir,
        Err(err) => panic!("unable to read current dirctory: {err:?}"),
    };

    let dir_entries = dir
        // for now, it's okay to ignore the errors.
        // TODO: when the time is right, do not ignore these errors and display something else that
        // would be better
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    return dir_entries;
}
