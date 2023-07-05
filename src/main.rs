use std::env;
use std::fs;
use std::path::PathBuf;

use crossterm::{
    execute,
    style::Stylize,
    terminal::{Clear, ClearType},
    Result,
};
use std::fs::File;
use std::io::{self, BufRead};

mod tui_gen;
mod tui_menu;

const VIEWTOP: usize = 3;
const VIEWBOT: usize = 2;

fn main() {
    let mut log_files = find_log_files().expect(format!("{}", "log files not found - cd to log dir".red()).as_str());
    log_files.sort();
    let last_log_file = log_files.last().expect(format!("{}", "last log files not found".red()).as_str());

    if let Err(err) = display_log_file(last_log_file) {
        eprintln!("Error: {}", err);
    }

    loop {
        let log_file = &display_vector_items(&log_files);
        if let Err(err) = display_log_file(log_file) {
            eprintln!("Error: {}", err);
        }
    }
}

fn display_log_file(file_path: &PathBuf) -> Result<()> {
    let mut stdout = io::stdout();

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

    let (terminal_width, terminal_height) = crossterm::terminal::size()?;
    let th = (terminal_height as usize - VIEWTOP - VIEWBOT) - 1;
    let mut offset = 0;

    let fname = file_path.file_name().unwrap().to_str().unwrap();

    tui_gen::cls();
    display_header(fname);
    tui_gen::cmove(0, VIEWTOP);

    if lines.len() < th {
        for (i, line) in lines[offset..(lines.len())].iter().enumerate() {
            let buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), line);
            print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), line);
            print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
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
                    execute!(stdout, Clear(ClearType::CurrentLine))?;
                    let buff = format!("{}: {}\r", format!("{:4}", i + offset).red(), line);
                    print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
                }
            }
        }
    }

    Ok(())
}

fn display_vector_items(vector: &Vec<PathBuf>) -> PathBuf {
    let (_terminal_width, terminal_height) = crossterm::terminal::size().unwrap();
    let mut offset = 0;
    let mut display_limit = terminal_height as usize - 8;
    let mut current_line = 0;

    if vector.len() < display_limit {
        display_limit = vector.len();
    }

    tui_gen::cls();
    display_header("");

    let menu_items = vec![
        ("j", "Scroll_Dn"),
        ("k", "Scroll_Up"),
        ("s", "Select"),
        ("q", "Quit"),
    ];

    let mut stdout = io::stdout();

    let mut v = vector.clone();
    v.reverse();

    loop {
        tui_gen::cmove(0, VIEWTOP);

        print!(" Select file to display: (");
        print!("{}", format!("{} logs", v.len()).red());
        println!(")");
        println!();
        for (index, item) in v.iter().enumerate().skip(offset).take(display_limit) {
            let buffer = format!("{:?}", item.as_path().file_name().unwrap());
            execute!(stdout, Clear(ClearType::CurrentLine)).unwrap();
            print!("    {}: ", format!("{:5}", index).red());
            if index - offset == current_line {
                println!("{}", buffer.trim_matches('"').green());
            } else {
                println!("{}", buffer.trim_matches('"'));
            }
        }

        let input = tui_menu::menu_horiz(&menu_items);

        match input {
            'j' => {
                if current_line < display_limit - 1 && current_line < vector.len() - offset - 1 {
                    current_line += 1;
                }
                if current_line == display_limit - 1 && current_line < vector.len() - offset - 1 {
                    offset += 1;
                }
            }
            'k' => {
                if current_line > 0 {
                    current_line -= 1;
                }
                if current_line == 0 && offset > 0 {
                    offset -= 1;
                }
            }
            'q' => {
                tui_gen::cmove(0, terminal_height as usize);
                execute!(stdout, Clear(ClearType::CurrentLine)).unwrap();
                std::process::exit(1);
            }
            's' => break,
            _ => break,
        }
    }

    v.get(offset + current_line).unwrap().to_path_buf()
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

fn print_without_wrapping(text: &str, max_width: usize) {
    let remaining = text;

    if remaining.len() <= max_width {
        println!("{}", remaining);
    } else {
        println!("{}\r", &remaining[..max_width]);
    }
}

fn display_header(file_name: &str) {
    println!(
        "{} {} {}{} {}",
        " View Last Log:".blue(),
        get_prog_name().green(),
        "v".green(),
        env!("CARGO_PKG_VERSION").green(),
        file_name
    );

    tui_gen::cmove(0, 1);
    tui_gen::horiz_line("blue");
}
