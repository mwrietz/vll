// todo:
//
// + refactor clear line
// maintain file list highlighted when quitting file view
//

use std::env;
use std::fs;
use std::path::PathBuf;

use crossterm::{
    style::Stylize,
    Result,
};
use std::fs::File;
use std::io::{self, BufRead};

mod tui_gen;
mod tui_menu;

struct ViewStatus {
    current_index: usize,
    current_line: usize,
    offset: usize,
    display_limit: usize,
}

const VIEWTOP: usize = 3;
const VIEWBOT: usize = 2;

fn main() {
    let mut log_files = find_log_files().expect(format!("{}", "log files not found - cd to log dir".red()).as_str());
    log_files.sort();
    let last_log_file = log_files.last().expect(format!("{}", "last log files not found".red()).as_str());

    if let Err(err) = display_log_file(last_log_file) {
        eprintln!("Error: {}", err);
    }

    let mut vs = ViewStatus {
        current_index: 0,
        current_line: 0,
        offset: 0,
        display_limit: 10,
    };

    loop {
        let log_file = &display_vector_items(&log_files, &mut vs);
        if let Err(err) = display_log_file(log_file) {
            eprintln!("Error: {}", err);
        }
    }
}

fn display_file_head(file_path: &PathBuf) {
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);
    let all_lines = reader.lines().collect::<Result<Vec<String>>>().unwrap();

    // strip lines containing directories only
    let mut lines = Vec::new();
    for l in all_lines {
        if !l.ends_with("/") && !l.starts_with(" ") {
            lines.push(l.clone());
        }
    }

    let (terminal_width, terminal_height) = tui_gen::tsize();
    let th = (terminal_height as usize - VIEWTOP - VIEWBOT - 20) - 1;

    tui_gen::cmove(0, VIEWTOP + 14);
    println!(" File preview...");
    println!();

    if lines.len() < th {
        for (i, line) in lines.iter().enumerate() {
            let l = line.as_str();
            let mut _buff = String::from("");
            let max_width = terminal_width - 12;
            if l.len() > max_width {
                _buff = format!("     {}: {}\r", format!("{:4}", i).red(), format!("{}", &l[..max_width]).grey());
            } else {
                _buff = format!("     {}: {}\r", format!("{:4}", i).red(), format!("{}", l).grey());
            }
            tui_gen::clear_line();
            println!("{}", _buff);
        }
        for _ in 0..(th - lines.len()) {
            tui_gen::clear_line();
            println!();
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let l = line.as_str();
            let mut _buff = String::from("");
            let max_width = terminal_width - 12;
            if l.len() > max_width {
                _buff = format!("     {}: {}\r", format!("{:4}", i).red(), format!("{}", &l[..max_width]).grey());
            } else {
                _buff = format!("     {}: {}\r", format!("{:4}", i).red(), format!("{}", l).grey());
            }
            tui_gen::clear_line();
            println!("{}", _buff);
        }
    }
}

fn display_log_file(file_path: &PathBuf) -> Result<()> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let all_lines = reader.lines().collect::<Result<Vec<String>>>()?;

    // strip lines containing directories only
    let mut lines = Vec::new();
    for l in all_lines {
        if !l.ends_with("/") && !l.starts_with(" ") {
            lines.push(l.clone());
        }
    }

    let (terminal_width, terminal_height) = tui_gen::tsize();
    let th = (terminal_height as usize - VIEWTOP - VIEWBOT) - 1;
    let mut offset = 0;

    let fname = file_path.file_name().unwrap().to_str().unwrap();

    tui_gen::cls();
    display_header(fname);
    tui_gen::cmove(0, VIEWTOP);

    if lines.len() < th {
        for (i, line) in lines[offset..(lines.len())].iter().enumerate() {
            let l = line.as_str();
            let mut _buff = String::from("");
            let max_width:usize = terminal_width - 6;
            
            if l.len() > max_width {
                _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), &l[0..max_width]);
            } else {
                _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), l);
            }
            println!("{}", _buff);
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let l = line.as_str();
            let mut _buff = String::from("");
            let max_width: usize = terminal_width - 6;
            if l.len() > max_width {
                _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), &l[..max_width]);
            } else {
                _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), l);
            }
            println!("{}", _buff);
        }
    }

    let menu_items = vec![
        ("j", "Scroll_DN"),
        ("k", "Scroll_UP"),
        ("d", "Page_DN"),
        ("g", "Top"),
        ("G", "Bottom"),
        ("q", "Quit"),
    ];

    loop {
        let mut update = false;
        let input = tui_menu::menu_horiz(&menu_items);
        match input {
            'q' => break,
            'k' if offset > 0 => {
                offset -= 1;
                update = true;
            }
            'j' => {
                if (offset + th - 1) < lines.len() - 1 {
                    offset += 1;
                    update = true;
                }
            }
            'g' => {
                offset = 0;
                update = true;
            }
            'G' => {
                if lines.len() > th {
                    offset = lines.len() - th;
                } else {
                    offset = 0;
                }
                update = true;
            }
            'd' => {
                if lines.len() > th {
                    if (offset + th) < (lines.len() - th - 1) {
                        offset += th;
                    } else {
                        offset = lines.len() - th;
                    }
                    update = true;
                }
            }
            _ => {}
        }
        if update {
            if lines.len() > th {
                tui_gen::cmove(0, VIEWTOP);
                for (i, line) in lines[offset..(offset + th)].iter().enumerate() {
                    let l = line.as_str();
                    let mut _buff = String::from("");
                    let max_width: usize = terminal_width - 6;
                    tui_gen::clear_line();
                    if l.len() > max_width {
                        _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), &l[..max_width]);
                    } else {
                        _buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), l);
                    }
                    println!("{}", _buff);
                }
            }
        }
    }

    Ok(())
}

fn display_vector_items(vector: &Vec<PathBuf>, vs: &mut ViewStatus) -> PathBuf {
    //let (_terminal_width, terminal_height) = crossterm::terminal::size().unwrap();
    let (_terminal_width, terminal_height) = tui_gen::tsize();
    // let mut offset = 0;
    // let mut display_limit = 10;
    // let mut current_line = 0;

    if vector.len() < vs.display_limit {
        vs.display_limit = vector.len();
    }

    tui_gen::cls();
    display_header("");

    let menu_items = vec![
        ("j", "Scroll_Dn"),
        ("k", "Scroll_Up"),
        ("s", "Select"),
        ("q", "Quit"),
    ];

    let mut v = vector.clone();
    v.reverse();

    vs.current_index = 0;

    loop {
        tui_gen::cmove(0, VIEWTOP);

        print!(" Select file to display: (");
        print!("{}", format!("{} logs", v.len()).red());
        println!(")");
        println!();
        //let mut current_index: usize = 0;
        for (index, item) in v.iter().enumerate().skip(vs.offset).take(vs.display_limit) {
            let buffer = format!("{:?}", item.as_path().file_name().unwrap());
            tui_gen::clear_line();
            print!("    {}: ", format!("{:5}", index).red());
            if index - vs.offset == vs.current_line {
                println!("{} {}", buffer.trim_matches('"').dark_green().bold(), "*".dark_green().bold());
                vs.current_index = index;
            } else {
                println!("{}", buffer.trim_matches('"'));
            }
        }
        display_file_head(&v[vs.current_index]);

        let input = tui_menu::menu_horiz(&menu_items);

        match input {
            'j' => {
                if vs.current_line < vs.display_limit - 1 && vs.current_line < vector.len() - vs.offset - 1 {
                    vs.current_line += 1;
                }
                if vs.current_line == vs.display_limit - 1 && vs.current_line < vector.len() - vs.offset - 1 {
                    vs.offset += 1;
                }
            }
            'k' => {
                if vs.current_line > 0 {
                    vs.current_line -= 1;
                }
                if vs.current_line == 0 && vs.offset > 0 {
                    vs.offset -= 1;
                }
            }
            'q' => {
                tui_gen::cmove(0, terminal_height as usize);
                tui_gen::clear_line();
                std::process::exit(1);
            }
            's' => break,
            _ => break,
        }
    }

    v.get(vs.offset + vs.current_line).unwrap().to_path_buf()
}

fn find_log_files() -> io::Result<Vec<PathBuf>> {
    let current_dir = env::current_dir()?;
    let log_files: Vec<PathBuf> = fs::read_dir(current_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file() && path.extension().is_some() && path.extension().unwrap() == "log"
        })
        .collect();
    Ok(log_files)
}

fn get_prog_name() -> String {
    let prog_name = env::current_exe()
        .expect("Can't get the exec path")
        .file_name()
        .expect("Can't get the exec name")
        .to_string_lossy()
        .into_owned();
    prog_name
}

fn display_header(file_name: &str) {
    println!(
        "{} {} {}{} {}",
        " View Last Log:".blue(),
        get_prog_name().dark_green().bold(),
        "v".green(),
        env!("CARGO_PKG_VERSION").dark_green().bold(),
        file_name
    );

    tui_gen::cmove(0, 1);
    tui_gen::horiz_line("blue");
}
