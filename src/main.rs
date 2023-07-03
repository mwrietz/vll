use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::Stylize,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    Result,
};
use std::fs::File;
use std::io::{self, BufRead};

mod tui_gen;
mod tui_menu;

const VIEWTOP: u16 = 2;

fn main() -> io::Result<()> {
    let mut log_files = find_log_files()?;
    log_files.sort();
    let last_log_file = log_files.last().expect("last log file not found");

    if let Err(err) = display_log_file(last_log_file) {
        eprintln!("Error: {}", err);
    }

    let menu_items = vec![("n", "Next"), ("p", "Previous"),("l", "Last"), ("q", "Quit")];

    let mut n_log = log_files.len() - 1;
    loop {
        tui_gen::cls();
        println!(
            "{} {} {}{}",
            "View Last Log:".blue(),
            get_prog_name().green(),
            "v".green(),
            env!("CARGO_PKG_VERSION").green(),
        );
        tui_gen::horiz_line("blue");

        println!();
        print!("{}", "       # logs: ".blue());
        println!("{}", format!("{:5}", log_files.len()).red());
        println!();

        println!("{}", "                 #          log-name".blue());
        println!("{}", "               ----- -----------------------".blue());
        print!("{} ", "     previous:".blue());
        if n_log < (log_files.len() - 1) {
            print!("{}", format!("{:5} ", n_log + 1 + 1).red());
            println!("{:?}", &log_files[n_log + 1].file_name().unwrap()); 
        } else {
            print!("{}", format!("{:5} ", n_log + 1).red());
            println!("{}", format!("{:?}", &log_files[n_log].file_name().unwrap()).yellow()); 
        }

        print!("{} ", "  last viewed:".blue());
        if n_log == (log_files.len() - 1) || n_log == 0 {
            print!("{}", format!("{:5} ", n_log + 1).red());
            println!("{}", format!("{:?}", &log_files[n_log].file_name().unwrap()).yellow()); 
        } else {
            print!("{}", format!("{:5} ", n_log + 1).red());
            println!("{:?}", &log_files[n_log].file_name().unwrap()); 
        }

        print!("{} ", "         next:".blue());
        if n_log > 0 {
            print!("{}", format!("{:5} ", n_log - 1 + 1).red());
            println!("{:?}", &log_files[n_log - 1].file_name().unwrap()); 
        } else {
            print!("{}", format!("{:5} ", n_log + 1).red());
            println!("{}", format!("{:?}", &log_files[n_log].file_name().unwrap()).yellow()); 
        }
    

        let val = tui_menu::menu_horiz(&menu_items);
        match val {
            'n' => {
                if n_log > 0 {
                    n_log -= 1;
                }
            }
            'p' => {
                if n_log < log_files.len() - 1 {
                    n_log += 1;
                }
            }
            'q' => break,
            _ => {},
        };
        let log_file = &log_files[n_log];

        if let Err(err) = display_log_file(log_file) {
            eprintln!("Error: {}", err);
        }
    }

    Ok(())
}

fn display_log_file(file_path: &PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

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
    let th = (terminal_height - VIEWTOP) as usize - 1;
    let mut offset = 0;

    execute!(stdout, Hide)?;
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, MoveTo(0, 0))?;
    println!(
        "{} {} {}{} {:?}",
        "View Last Log:".blue(),
        get_prog_name().green(),
        "v".green(),
        env!("CARGO_PKG_VERSION").green(),
        file_path.file_name().unwrap()
    );

    execute!(stdout, MoveTo(0, VIEWTOP))?;

    if lines.len() < th {
        for (i, line) in lines[offset..(lines.len())].iter().enumerate() {
            let buff = format!("{}: {}\r", format!("{}", i + offset).red(), line);
            print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
        }
    } else {
        for (i, line) in lines.iter().take(th).enumerate() {
            let buff = format!("{}: {}\r", format!("{}", i + offset).red(), line);
            print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
        }
    }

    stdout.flush()?;

    loop {
        let mut update = false;
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('k') if offset > 0 => {
                    offset -= 1;
                    update = true;
                }
                KeyCode::Char('j') => {
                    if (offset + th - 1) < lines.len() - 1 {
                        offset += 1;
                        update = true;
                    }
                }
                KeyCode::Char('g') => {
                    offset = 0;
                    update = true;
                }
                KeyCode::Char('G') => {
                    if lines.len() > th {
                        offset = lines.len() - th;
                    } else {
                        offset = 0;
                    }
                    update = true;
                }
                KeyCode::Char(' ') => {
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
                    execute!(stdout, MoveTo(0, VIEWTOP))?;
                    for (i, line) in lines[offset..(offset + th)].iter().enumerate() {
                        execute!(stdout, Clear(ClearType::CurrentLine))?;
                        let buff = format!("{}: {}\r", format!("{}", i + offset).red(), line);
                        print_without_wrapping(buff.as_str(), (terminal_width - 1) as usize);
                    }
                }
            }
            stdout.flush()?;
        }
    }

    execute!(stdout, Show)?;
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
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
