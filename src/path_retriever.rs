use std::io;
use std::fs;
use std::str;
use wildmatch::WildMatch;

use kape_handler::{TKapeEntry};
use std::path::Path;

use serde::{Deserialize, Serialize};
use clap::Clap;

mod kape_handler;
extern crate serde_json;

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "LIFARS LLC")]
pub struct Opts {
    /// Path to the output file, if not specified stdout is used
    #[clap(
        short = 'o',
        long = "output_file",
        )]
    pub output: Option<String>,

    /// Path to the input file
    #[clap(
        short = 'i',
        long = "input_file",
        default_value = "converted.json"
        )]
    pub input: String,
}

enum MatchType {
    ExactMatch(String),
    WildcardMatch(WildMatch),
    Anything()
}

struct Pattern {
    components : Vec::<MatchType>,
    file_mask : WildMatch,
    recursive : bool
}

impl Pattern {
    pub fn new(path : &str, file_mask : &str, recursive : bool) -> Pattern {

        let path_copy = path.to_string().to_lowercase();

        let p = Path::new(&path_copy);
        let pb = p.to_path_buf();

        let mut components =  Vec::<MatchType>::new();

        for a in pb.iter() {
            let as_str = a.to_str().unwrap();

            // For some reason it sometimes splits into empty \
            if as_str == "\\" {
                continue;
            }

            if   as_str == "%user%" || as_str == "*" {
                components.push(MatchType::Anything());
            }
            else if as_str.contains("*")  {
                components.push(MatchType::WildcardMatch(WildMatch::new(as_str)));
            }
            else {
                components.push(MatchType::ExactMatch(as_str.to_string()));
            }

        }

        let mut mask = file_mask;

        if mask.is_empty() {
            mask = "*"; 
        }

        Pattern { components: components, file_mask: WildMatch::new(mask), recursive: recursive }
    }



    pub fn parent_matches(&self, path : &Vec<String>) -> bool {
        if path.len() < self.components.len() {
            return false;
        }

        if !self.recursive && path.len() != self.components.len() {
            return false;
        }

        for (i, comp) in self.components.iter().enumerate() {
            match &comp {
                MatchType::Anything() => continue,
                MatchType::ExactMatch(x) => if *x != path[i] { return false },
                MatchType::WildcardMatch(x) => if !x.matches(&path[i]) { return false },
            }
        }

        true
    }


    pub fn matches(&self, path : &Vec<String>) -> bool {
        if path.len() <= self.components.len() {
            return false;
        }

        if !self.recursive && path.len() - 1 != self.components.len() {
            return false;
        }

        if !self.file_mask.matches(&path[path.len() -1]) {
            return false;
        }

        for (i, comp) in self.components.iter().enumerate() {
            match &comp {
                MatchType::Anything() => continue,
                MatchType::ExactMatch(x) => if *x != path[i] { return false },
                MatchType::WildcardMatch(x) => if !x.matches(&path[i]) { return false },
            }
        };

        true
    }
}

fn convert_path_for_matching(path : &Path) -> Vec<String> {
    path.to_path_buf().iter().map(|f| f.to_str().unwrap().to_string().to_lowercase()).filter(|f| f != "\\").collect()
}

fn iterate_fs(current_path : &Path, patterns : &Vec<Pattern>, output : &mut io::Write) {

    if let Ok(dirs) = fs::read_dir(current_path) {
        for dir in dirs {
            if let Ok(entry) = dir {
                let path = entry.path();
    
                let converted_path = convert_path_for_matching(&path);

                if path.is_dir() {

                    for pattern in patterns {
                        if pattern.parent_matches(&converted_path) {
                            iterate_fs(&path, patterns, output);
                            break;
                        }
                    }
                }
                else {
                    for pattern in patterns {
                        if pattern.matches(&converted_path) {
                            let display = path.display().to_string() + "\n";
                            let _ = output.write(display.as_bytes());
                            break;
                        }
                    }
                }

            }
        }
    }
}

fn main() -> Result<(), io::Error> {

    let opts: Opts = Opts::parse();

    let mut output = match &opts.output {
        Some(x) => {
            let path = Path::new(x);
            Box::new(fs::File::create(&path).unwrap()) as Box<dyn io::Write>
        }
        None => Box::new(io::stdout()) as Box<dyn io::Write>,
    };

    let mut patterns = Vec::<Pattern>::new();
    let mut drive_letters = Vec::<String>::new();

    if let Ok(contents) =  fs::read_to_string(opts.input) {

        let deserialized : Vec<TKapeEntry> = serde_json::from_str(&contents).unwrap_or_default();

        if deserialized.len() == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "Eror reading JSON"));
        }

        for config in deserialized {
            for target in config.targets {
                let pattern = Pattern::new(target.path.as_str(), target.file_mask.as_str(), target.recursive);

                for component in &pattern.components {
                    if let MatchType::ExactMatch(x) = &component {
                        let drive_letter = x.clone().to_uppercase() + "\\";
                        if !drive_letters.contains(&drive_letter) {
                            drive_letters.push(drive_letter);
                        }
                    }
                }

                patterns.push(pattern);

            }
        }
    }
    else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }

    for drive_letter in drive_letters {
        iterate_fs(Path::new(&drive_letter), &patterns, &mut *output);
    }
    Ok(())
}
