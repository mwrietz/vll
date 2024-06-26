use std::env;
use std::fs::{self, File};
use std::io::{self, stdout, BufRead};
use std::path::PathBuf;

use crossterm::{
    cursor, execute,
    style::{Color, Stylize},
    Result,
};

mod summarylog;
mod tui_gen;
mod tui_menu;

struct ViewStatus {
    current_index: usize,
    current_line: usize,
    offset: usize,
    display_limit: usize,
}

const HEADERHEIGHT: usize = 3;
const FOOTERHEIGHT: usize = 2;

fn main() {
    let _ = summarylog::create_summary_log();

    let mut log_files = find_log_files()
        .unwrap_or_else(|_| panic!("{}", "log files not found - cd to log dir".dark_red()));
    log_files.sort();
    let last_log_file = log_files.last().unwrap_or_else(|| {
        println!("{}", "last log file not found".dark_red());
        std::process::exit(1);
    });

    display_log_file(last_log_file);

    let mut vs = ViewStatus {
        current_index: 0,
        current_line: 0,
        offset: 0,
        display_limit: 10,
    };

    loop {
        let log_file = &select_log_file(&log_files, &mut vs);
        display_log_file(log_file);
    }
}

fn display_file_head(file_path: &PathBuf) {
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);
    let all_lines = reader.lines().collect::<Result<Vec<String>>>().unwrap();

    // strip lines containing directories only
    let mut lines = Vec::new();
    for l in all_lines {
        if !l.ends_with('/') && !l.starts_with(' ') {
            lines.push(l.clone());
        }
    }

    let (_terminal_width, terminal_height) = tui_gen::tsize();
    let th = terminal_height - HEADERHEIGHT - FOOTERHEIGHT - 16;

    println!();
    println!(" File preview...");
    println!();

    if lines.len() < th {
        for (i, line) in lines.iter().enumerate() {
            let l = line.as_str();
            display_line_grey(i, l);
        }
        for _ in 0..(th - lines.len()) {
            tui_gen::clear_line();
            println!();
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let l = line.as_str();
            display_line_grey(i, l);
        }
    }
    tui_gen::clear_line();
}

fn display_header(file_name: &str) {
    tui_gen::print_page_header("View Latest Log:");
    tui_gen::cursor_move(17, 1);
    tui_gen::print_color(file_name, Color::DarkGreen);
}

fn display_line(i: usize, l: &str) {
    let (terminal_width, _terminal_height) = tui_gen::tsize();

    let mut _buff = String::from("");
    let max_width = terminal_width - 14;
    if l.len() > max_width {
        _buff = format!("{}: {}\r", format!("{:4}", i).dark_red(), &l[..max_width]);
    } else {
        _buff = format!("{}: {}\r", format!("{:4}", i).dark_red(), l);
    }
    tui_gen::clear_line();
    println!("{}", _buff);
}

fn display_line_grey(i: usize, l: &str) {
    let (terminal_width, _terminal_height) = tui_gen::tsize();

    let mut _buff = String::from("");
    let max_width = terminal_width - 14;
    if l.len() > max_width {
        _buff = format!(
            "     {}: {}\r",
            format!("{:4}", i).dark_red(),
            (l[..max_width]).to_string().dark_grey()
        );
    } else {
        _buff = format!(
            "     {}: {}\r",
            format!("{:4}", i).dark_red(),
            l.to_string().dark_grey()
        );
    }
    tui_gen::clear_line();
    println!("{}", _buff);
}

fn display_log_file(file_path: &PathBuf) {
    let file = File::open(file_path).expect("cannot open file_path");
    let reader = io::BufReader::new(file);
    let all_lines = reader
        .lines()
        .collect::<Result<Vec<String>>>()
        .expect("cannot read file");

    let mut lines = Vec::new();
    for l in all_lines {
        if !l.ends_with('/') && !l.starts_with(' ') {
            lines.push(l.clone());
        }
    }

    let (_terminal_width, terminal_height) = tui_gen::tsize();
    let th = (terminal_height - HEADERHEIGHT - FOOTERHEIGHT) - 1;
    let mut offset = 0;

    let fname = file_path.file_name().unwrap().to_str().unwrap();

    tui_gen::cls();
    display_header(fname);
    tui_gen::cursor_move(0, HEADERHEIGHT);

    if lines.len() < th {
        for (i, line) in lines[offset..(lines.len())].iter().enumerate() {
            let l = line.as_str();
            display_line(i + offset, l);
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let l = line.as_str();
            display_line(i + offset, l);
        }
    }

    let menu_items = vec![
        ("j", "Scroll_DN"),
        ("k", "Scroll_UP"),
        ("d", "Page_DN"),
        ("b", "Page_UP"),
        ("g", "Top"),
        ("G", "Bottom"),
        ("s", "Select_Log"),
        ("q", "Quit"),
    ];

    loop {
        let mut update = false;
        let input = tui_menu::menu_horiz(&menu_items);
        match input {
            's' => break,
            'q' => {
                tui_gen::cursor_move(0, terminal_height);
                tui_gen::clear_line();
                show_cursor();
                std::process::exit(1);
            }
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
            'd' | ' ' => {
                if lines.len() > th {
                    if (offset + th) < (lines.len() - th - 1) {
                        offset += th;
                    } else {
                        offset = lines.len() - th;
                    }
                    update = true;
                }
            }
            'b' => {
                if lines.len() > th {
                    if offset > th {
                        offset -= th;
                    } else {
                        offset = 0;
                    }
                    update = true;
                }
            }
            _ => {}
        }
        if update && lines.len() > th {
            tui_gen::cursor_move(0, HEADERHEIGHT);
            for (i, line) in lines[offset..(offset + th)].iter().enumerate() {
                let l = line.as_str();
                display_line(i + offset, l);
            }
        }
        tui_gen::clear_line();
    }
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

fn select_log_file(vector: &[PathBuf], vs: &mut ViewStatus) -> PathBuf {
    let (_, terminal_height) = tui_gen::tsize();

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

    let mut v = vector.to_owned();
    v.reverse();

    vs.current_index = 0;

    loop {
        tui_gen::cursor_move(0, HEADERHEIGHT);

        print!(" Select file to display: (");
        print!("{}", format!("{} logs", v.len()).dark_red());
        println!(")");
        tui_gen::clear_line();

        println!();
        for (index, item) in v.iter().enumerate().skip(vs.offset).take(vs.display_limit) {
            let buffer = format!("{:?}", item.as_path().file_name().unwrap());
            tui_gen::clear_line();
            print!("    {}: ", format!("{:5}", index).dark_red());
            if index - vs.offset == vs.current_line {
                println!(
                    "{} {}",
                    buffer.trim_matches('"').dark_green().bold(),
                    "*".dark_green().bold()
                );
                vs.current_index = index;
            } else {
                println!("{}", buffer.trim_matches('"'));
            }
        }

        tui_gen::clear_line();

        display_file_head(&v[vs.current_index]);

        let input = tui_menu::menu_horiz(&menu_items);

        match input {
            'j' => {
                if vs.current_line < vs.display_limit - 1
                    && vs.current_line < vector.len() - vs.offset - 1
                {
                    vs.current_line += 1;
                }
                if vs.current_line == vs.display_limit - 1
                    && vs.current_line < vector.len() - vs.offset - 1
                {
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
                tui_gen::cursor_move(0, terminal_height);
                tui_gen::clear_line();
                show_cursor();
                std::process::exit(1);
            }
            's' | '\x0A' => break,
            _ => break,
        }
    }

    v.get(vs.offset + vs.current_line).unwrap().to_path_buf()
}

fn show_cursor() {
    execute!(stdout(), cursor::Show).unwrap();
}
