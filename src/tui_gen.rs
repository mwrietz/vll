use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
    terminal::{Clear, ClearType},
};
use std::env;
use std::io::stdout;

pub fn cls() {
    std::process::Command::new("clear").status().unwrap();
}

pub fn clear_line() {
    execute!(stdout(), Clear(ClearType::CurrentLine)).unwrap();
}

pub fn cursor_move(x: usize, y: usize) {
    execute!(stdout(), cursor::MoveTo(x as u16, y as u16)).unwrap();
}

pub fn get_prog_name() -> String {
    let prog_name = env::current_exe()
        .expect("Can't get the exec path")
        .file_name()
        .expect("Can't get the exec name")
        .to_string_lossy()
        .into_owned();
    prog_name
}

pub fn horiz_line(color: Color) {
    let (width, _) = tsize();
    for _i in 0..width {
        print_color_bold("─", color);
    }
    println!();
}

pub fn print_color(my_str: &str, color: Color) {
    execute!(
        stdout(),
        SetForegroundColor(color),
        Print(my_str),
        ResetColor
    )
    .expect("print_color error");
}

pub fn print_color_bold(my_str: &str, color: Color) {
    execute!(
        stdout(),
        SetForegroundColor(color),
        Print(my_str.bold()),
        ResetColor
    )
    .expect("print_color_bold error");
}

pub fn print_page_header(title: &str) {
    print_title(title, Color::DarkBlue);

    // print version right justified
    let (w, _h) = tsize();
    let prog_name = get_prog_name();
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let offset = prog_name.len() + version.len() + 2;
    cursor_move(w - offset, 1);

    print_color(
        prog_name.as_str(),
        Color::Rgb {
            r: 255,
            g: 135,
            b: 0,
        },
    );
    print_color(" ", Color::Black);
    print_color(
        version.as_str(),
        Color::Rgb {
            r: 255,
            g: 135,
            b: 0,
        },
    );
    println!();
    horiz_line(Color::DarkBlue);
    cursor_move(0, 4);
}

pub fn print_title(title_string: &str, color: Color) {
    println!();
    for c in title_string.chars() {
        //print!(" ");
        print_color_bold(&c.to_string(), color);
    }
    println!();
    horiz_line(color);
    println!();
}

//
// TermStat usage:
// let mut termstat = TermStat::default();
//

pub struct TermStat {
    pub line_count: usize,
    pub width: usize,
    pub height: usize,
    pub xpos: usize,
    pub ypos: usize,
}

impl Default for TermStat {
    fn default() -> TermStat {
        let (w, h) = tsize();
        let (x, y) = tpos();
        TermStat {
            line_count: 0,
            width: w,
            height: h,
            xpos: x,
            ypos: y,
        }
    }
}

pub fn tpos() -> (usize, usize) {
    let pos = crossterm::cursor::position();
    let (x, y) = match pos {
        Ok((x, y)) => (x, y),
        Err(error) => panic!("tpos error: {:?}", error),
    };
    (x as usize, y as usize)
}

pub fn tsize() -> (usize, usize) {
    let size = crossterm::terminal::size();
    let (w, h) = match size {
        Ok((w, h)) => (w, h),
        Err(error) => panic!("tsize error: {:?}", error),
    };
    (w as usize, h as usize)
}
