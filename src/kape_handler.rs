
extern crate serde_json;

use serde::{Deserialize, Serialize};

use std::path::Path;
use std::fs;
use std::result::Result;
use std::io::{self, BufRead};
use std::fs::File;

#[derive(Serialize, Deserialize, Default)]
pub struct MKapeEntry {
    pub executable : String,
    pub commad_line : String
}

#[derive(Serialize, Deserialize, Default)]
pub struct TKapeEntry {
    pub description : String,
    pub recreate_directories : bool,
    pub targets : Vec<JSONTarget>
}

#[derive(Serialize, Deserialize, Default)]
pub struct JSONTarget {
    pub name : String,
    pub comment : String,
    pub category : String,

    pub path: String,
    pub file_mask: String,
    pub recursive : bool,
}

pub fn parse_mkape(path : &Path) -> std::result::Result<Vec<MKapeEntry>, io::Error> {

    let file = File::open(path)?;
    let lines = io::BufReader::new(file).lines().collect::<Result<_, _>>().unwrap();

    let mut cur = Vec::<KapePair>::new();

    parse_config(0, 0, &mut cur, &lines);
    
    let mut kapes = Vec::<MKapeEntry>::new();

    for var in cur {

        match var {
            KapePair::Multiple(x, y) => {
                match x.as_ref() {
                    "Processors" | "processors" => {
                        for subvar in y {
                            let mut processor : MKapeEntry = Default::default();

                            for subvarr in subvar {
                                match subvarr {
                                    KapePair::Single(name, value) => {
                                        let lower_name = name.to_lowercase();
                                        match lower_name.as_ref() {
                                            "executable" => { processor.executable = value.to_string() },
                                            "commandline" => { processor.commad_line = value.to_string() },
                                            _ => {}
                                        };
                                    }
                                    _ => {}
                                }
                            }

                            kapes.push(processor);
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }

    Ok(kapes)
}

enum KapePair {
    Single(String, String),
    Multiple(String, Vec<Vec<KapePair>>)
}

fn remove_leading_whitspaces(in_string: &str) -> (String, usize) {
    
    let mut leading_count = 0usize;
    let bytes = in_string.as_bytes();

    while leading_count < in_string.len() && bytes[leading_count].is_ascii_whitespace() {
        leading_count = leading_count + 1;
    }

    (in_string.chars().into_iter().skip(leading_count).take(in_string.len() + 1 - leading_count).collect(), leading_count)
}

fn split_once(in_string: &str) -> Result<(&str, &str), io::Error> {
    let mut splitter = in_string.splitn(2, ':');
    
    let first: &str;

    match splitter.next() {
        Some(x) => first = x,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Error"))
    }

    match splitter.next() {
        Some(x) => Ok((first, x)),
        None => Err(io::Error::new(io::ErrorKind::Other, "Error"))
    }
}

fn is_empty_or_comment(in_string : &str) -> bool {
    return in_string.starts_with("#") || in_string.len() == 0 || in_string.chars().all(|x| x.is_ascii_whitespace());
}

fn parse_config(index : usize, mut level : usize, current_pair : &mut Vec<KapePair>, lines : &Vec<String>) -> usize {
    
    let mut has_group = false;
    let mut group = KapePair::Multiple("".to_string(), Vec::<Vec::<KapePair>>::new());
    let mut  handle_multiple = true;
    for mut i in index..(lines.len() - 1) {
        let line = &lines[i];

        if is_empty_or_comment(line) {
            continue;
        }

        if let Ok((key, value)) = split_once(&line) {
            if has_group {
                current_pair.push(group);
                group = KapePair::Multiple("".to_string(), Vec::<Vec::<KapePair>>::new());
                has_group = false;
            }

            let (new_key, leading_count) = remove_leading_whitspaces(&key);
            let (new_value, _) = remove_leading_whitspaces(&value);
            
            if leading_count < level {
                return i;
            } else if new_value.len() == 0 {
                group = KapePair::Multiple(new_key.to_string(), Vec::<Vec::<KapePair>>::new());
                handle_multiple = true;
                has_group = true;
                continue;
            } else {
                level = leading_count;
                
                let new_key_str = new_key.to_string();
                let mut new_value_str = new_value.to_string();

                if (new_value_str.chars().nth(0).unwrap() == '\'' && new_value.chars().last().unwrap() == '\'') ||
                  (new_value_str.chars().nth(0).unwrap() == '"' && new_value.chars().last().unwrap() == '"') {
                    new_value_str = new_value[1..new_value.len() - 1].to_string();
                }

                current_pair.push(KapePair::Single(new_key_str, new_value_str));
                handle_multiple = false;
            }

        }
        else if handle_multiple && line.contains("-") {
            loop {
                if i >= lines.len() || lines[i] != *line || is_empty_or_comment(&lines[i]) {
                    break;
                }

                let mut sub_pair = Vec::<KapePair>::new();

                i = parse_config(i + 1, level + 1, &mut sub_pair, &lines);
                if sub_pair.len() > 0 && has_group {

                    match &mut group {
                        KapePair::Multiple(_, x) => {
                            x.push(sub_pair);
                        } 
                        _ => {}
                    }   
                }
            }
        } 
        else {
            return i;
        }
    } 

    if has_group {
        current_pair.push(group);
    }

    return lines.len();
}

pub fn parse_tkape(path : &Path) -> std::result::Result<TKapeEntry, io::Error> {
    println!("Parsing {}", path.display());

    let file = File::open(path)?;
    let lines = io::BufReader::new(file).lines().collect::<Result<_, _>>().unwrap();

    let mut cur = Vec::<KapePair>::new();

    parse_config(0, 0, &mut cur, &lines);
    
    let mut entry : TKapeEntry = Default::default();

    for var in cur {

        match var {
            KapePair::Multiple(x, y) => {
                match x.as_ref() {
                    "Targets" | "targets" => {
                        for subvar in y {
                            let mut target  : JSONTarget = Default::default();

                            for subvarr in subvar {
                                match subvarr {
                                    KapePair::Single(name, value) => {
                                        let lower_name = name.to_lowercase();

                                        match lower_name.as_ref() {
                                            "name" => { target.name = value.to_string() },
                                            "comment" => { target.comment = value.to_string() },
                                            "category" => { target.category = value.to_string() },
                                            "path" => {target.path = value.to_string() },
                                            "filemask" => {target.file_mask = value.to_string() },
                                            "recursive" => { target.recursive = match value.as_ref() {
                                                "true" | "True" => true,
                                                "false" | "False" => false,
                                                _ => false
                                                }
                                            }
                                            _ => {}
                                        };
                                    }
                                    _ => {}
                                }
                            }

                            entry.targets.push(target);
                        }
                    }
                    _ => {}
                }
            },
            KapePair::Single(name, value) => {
                match name.as_ref() {
                    "Description" => entry.description = value,
                    "RecreateDirectories" => {
                        entry.recreate_directories = match value.as_ref() {
                            "true" | "True" => true,
                            "false" | "False" => false,
                            _ => false
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(entry)
}

pub fn convert_kape_config(str_path : &String) -> std::result::Result<(), io::Error> {
    let path = Path::new(str_path);

    if !path.exists() || !path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Directory {}  does not exist!", str_path)));
    }

    let mut tkapes = Vec::<TKapeEntry>::new();
    let mut mkapes = Vec::<MKapeEntry>::new();

    fn handle_dir(path : &Path, tkapes : &mut Vec::<TKapeEntry>, mkapes: &mut Vec::<MKapeEntry>) -> Result<(), io::Error> {
        for entry_res in fs::read_dir(path)? {
            let entry = entry_res?;
            let i_sub_path = entry.path();
            let sub_path = i_sub_path.as_path();

            if sub_path.is_dir() {
                let _ = handle_dir(&sub_path, tkapes, mkapes);
            }
            else {
                let ext = sub_path.extension().unwrap_or_default();

                // Skip disabled
                for comp in i_sub_path.iter() {
                    if comp.to_str().unwrap().chars().nth(0).unwrap() == '!' {
                        continue;
                    }
                }

                if ext == "mkape" {
                    match &mut parse_mkape(&sub_path) {
                        Ok(new_mkapes) => mkapes.append(new_mkapes),
                        _ => println!("Error parsing {}", sub_path.display())
                    }
                } else if  ext == "tkape" {

                    match parse_tkape(&sub_path) {
                        Ok(tkape) => tkapes.push(tkape),
                        _ => println!("Error parsing {}", sub_path.display())
                    }
                }
            }
        }
        Ok(())
    }

    let _ = handle_dir(&path, &mut tkapes, &mut mkapes);

    //TODO: Do something with mkapes
    for x in mkapes.iter(){
        println!("{} {}", x.executable, x.commad_line);
    }

    let _ = fs::write("converted.json", serde_json::to_string(&tkapes).unwrap());
    Ok(())
}
