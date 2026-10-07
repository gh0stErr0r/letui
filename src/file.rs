use std::{fs, io, path::Path};

pub fn load(path: Option<&Path>) -> io::Result<Vec<String>> {
    match path {
        Some(path) if path.exists() => {
            let content = fs::read_to_string(path)?;
            let mut lines: Vec<String> = content.lines().map(str::to_owned).collect();
            if lines.is_empty() {
                lines.push(String::new());
            }
            Ok(lines)
        }
        _ => Ok(vec![String::new()]),
    }
}

pub fn save(path: Option<&Path>, lines: &[String]) -> io::Result<()> {
    match path {
        Some(path) => fs::write(path, lines.join("\n")),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no file name; use a file path when launching letui",
        )),
    }
}
