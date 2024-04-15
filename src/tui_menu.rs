use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use getch::Getch;
use std::io;
use std::io::{stdout, Write};

use crate::tui_gen;
use crate::tui_gen::cursor_move;
use crate::tui_gen::print_color;
use crate::tui_gen::tsize;

pub fn menu_horiz(items: &[(&str, &str)]) -> char {
    let (_width, height) = tsize();
    cursor_move(0, height - 1);

    print_title_block();

    for item in items.iter() {
        let buffer = format!("{:>3}", item.0);
        print_color(&buffer, Color::DarkGreen);
        let buffer = format!(":{}", item.1);
        print_color(&buffer, Color::Grey);
    }
    execute!(stdout(), cursor::Hide).unwrap();
    io::stdout().flush().unwrap();

    let mut _a: u8 = 0;
    loop {
        let mut flag = false;
        let g = Getch::new();
        _a = g.getch().unwrap();

        for item in items.iter() {
            let ch = item.0.chars().next().unwrap();
            if (_a as char) == ch || (_a as char) == ' ' || (_a as char) == '\x0A' {
                flag = true;
                break;
            }
        }
        if flag {
            break;
        }
    }

    _a as char
}

fn print_title_block() {
    let prog_name = tui_gen::get_prog_name();
    execute!(
        stdout(),
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::Rgb {
            r: 255,
            g: 135,
            b: 0
        }),
        Print(format!(" {} ", prog_name)),
        ResetColor
    )
    .expect("print_title_block error");
}
