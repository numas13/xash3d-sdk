use std::{env, fs, io, path::Path};

struct Generate<'a> {
    root: &'a Path,
    path: &'a Path,
    pad: usize,
    filter: &'a [&'a str],
}

impl<'a> Generate<'a> {
    fn new(root: &'a Path, path: &'a Path, filter: &'a [&'a str]) -> Self {
        Self {
            root,
            path,
            pad: 0,
            filter,
        }
    }

    fn child(&self, path: &'a Path) -> Generate<'a> {
        Self {
            path,
            pad: self.pad + 4,
            ..*self
        }
    }

    fn visit(&self) -> io::Result<()> {
        let Some(current) = self.path.file_name().and_then(|s| s.to_str()) else {
            eprintln!("invalid file name {:?}", self.path);
            return Ok(());
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in fs::read_dir(self.path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if self.filter.is_empty() || self.filter.contains(&ext) {
                    files.push(path);
                }
            }
        }

        if dirs.is_empty() && files.is_empty() {
            return Ok(());
        }

        dirs.sort();
        files.sort();

        let mut pad = self.pad;
        if pad == 0 {
            println!("#[rustfmt::skip]");
        }
        let mod_name = escape_name(current).to_lowercase();
        println!("{:pad$}pub mod {mod_name} {{", "");
        pad += 4;
        if !files.is_empty() {
            println!("{:pad$}use core::ffi::CStr;", "");
            println!();
        }

        for (i, path) in dirs.iter().enumerate() {
            if i != 0 {
                println!();
            }
            self.child(path).visit()?;
        }

        if !dirs.is_empty() && !files.is_empty() {
            println!();
        }

        for path in files {
            let Ok(path) = path.strip_prefix(self.root) else {
                eprintln!("error: invalid path {path:?}");
                continue;
            };
            let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
                eprintln!("error: missing file name {path:?}");
                continue;
            };
            let name = escape_name(file_stem);
            println!("{:pad$}pub const {name}: &CStr = c{path:?};", "");
        }

        pad -= 4;
        println!("{:pad$}}}", "");

        Ok(())
    }
}

fn escape_name(file_stem: &str) -> String {
    let mut name = String::new();
    if file_stem.starts_with(|c: char| c.is_numeric()) {
        name.push('_');
    }
    for c in file_stem.chars() {
        if c == '_' || c.is_alphanumeric() {
            for c in c.to_uppercase() {
                name.push(c);
            }
        } else if c == '!' {
            name.push_str("EM");
        } else {
            name.push('_');
        }
    }
    name
}

fn process(root: &Path, name: &str, strip_name: bool, filter: &[&str]) -> io::Result<()> {
    let path = &Path::new(root).join(name);
    let root = if strip_name { path } else { root };
    if path.is_dir() {
        Generate::new(root, path, filter).visit()?;
    } else if !path.exists() {
        eprintln!("warning: {name} is not exists");
    } else {
        eprintln!("warning: {name} is not a directory");
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let root = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: gen-resources path/to/valve");
            std::process::exit(1);
        }
    };
    let root = Path::new(&root);

    println!("// DO NOT EDIT! AUTO-GENERATED!");
    println!();
    process(root, "events", false, &["sc"])?;
    println!();
    process(root, "gfx", true, &["tga"])?;
    println!();
    process(root, "models", false, &["mdl"])?;
    println!();
    process(root, "sound", true, &["wav"])?;
    println!();
    process(root, "sprites", false, &["spr"])?;

    Ok(())
}
