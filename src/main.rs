use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
    #[clap(default_value="index.lua")]
    filename:String
}
#[derive(Eq,Ord, PartialOrd, PartialEq,Debug)]
enum SearchingState {
    Looking = 0,
    InComment,
    Prefix,
    ImportWord,
    As
}
#[derive(Debug,PartialEq,Eq)]
enum ImportType {
    Unknown,
    Regular,
    Module
}

fn main() {
    let args:Cli = Cli::parse();

    let f = File::open(args.path.join(args.filename)).expect("The path you provided is invalid");
    let f = BufReader::new(f);

    let mut state = SearchingState::Looking;
    let mut last_char = '\0';
    let mut word = String::new();

    let mut new_file = Vec::<u8>::new();

    for res in f.lines() {
        if let Ok(line) = res {
            state=SearchingState::Looking;
            let mut import_fn = String::new();
            let mut import_t = ImportType::Unknown;
            for char in line.chars() {
                if state>SearchingState::Looking {
                    if char == '!' && last_char == '-' {state=SearchingState::Prefix}
                    else if state<SearchingState::Prefix { continue; };
                    if char.is_whitespace() && word.contains("@import") { state=SearchingState::ImportWord;import_t = ImportType::Regular; word.clear(); continue }
                    if char.is_whitespace() {
                        println!("WORD - {}",word);
                        match &*word {
                            "as" => {
                                state=SearchingState::As;
                                import_t=ImportType::Module;
                                word.clear();continue;
                            },
                            _ => {
                                import_fn = word.clone();
                                word.clear();continue;
                            }
                        }
                    }
                    word.push(char.to_ascii_lowercase());
                }
                if last_char == char && char == '-' {
                    state=SearchingState::InComment;
                }


                last_char = char;
            }
            let mut new_line = line.clone();
            if import_t != ImportType::Unknown && import_fn.is_empty() {import_fn = word.clone()}
            if import_t != ImportType::Unknown {
                let ref_file_path = args.path.join(if import_fn.contains(".") {import_fn} else {format!("{}.lua",import_fn)});
                if let Ok(mut ref_file) = File::open(&ref_file_path) {
                    new_line.clear();
                    ref_file.read_to_string(&mut new_line);
                    if import_t == ImportType::Module {
                        new_line = format!("local {} = {};",word,new_line)
                    }
                } else {
                    println!("unable to find a file @ {}",ref_file_path.to_str().unwrap_or("/"))
                }
            }
            new_file.append(&mut format!("{}\n",new_line).into_bytes())
        }
    }
    File::create(args.path.join("./dist.lua")).expect("what?").write_all(&new_file);
    println!("\npog:\n{}",String::from_utf8(new_file).unwrap())
}