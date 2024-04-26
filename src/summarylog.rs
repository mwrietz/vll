use std::env;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{self, Error, Read, Write};
use std::sync::Mutex;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

lazy_static::lazy_static! {
  static ref LOG_FILE: Mutex<File> = Mutex::new(File::create("summary.log").unwrap());
}

pub fn create_summary_log() -> Result<(), Error> {

    // Get list of files to process
    let mut log_files = find_log_files()
        .unwrap_or_else(|_| panic!("{}", "log files not found - cd to log dir"));
    log_files.sort();

    let last_file = log_files
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .split('/')
        .last()
        .unwrap();
    if last_file == "summary.log" {
        log_files.pop();
    }
    log_files.reverse();

    // Print header line to summary.log
    let _ = write_summary_log_header(&log_files[0]);
    
    // Process files in parallel
    log_files.into_par_iter().for_each(|file| {
        process_log_file(&file).unwrap();
    });

    // Sort the summary.log file
    if let Err(err) = sort_and_rewrite_file("summary.log") {
        eprintln!("Error sorting and rewriting file: {}", err);
    } else {
        println!("Successfully sorted and rewrote file: {}", "summary.log");
    }

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

fn write_summary_log_header(filename: &PathBuf) -> Result<(), Error> {
    
    // Determine field width for filenames
    let fname_width = filename 
        .as_os_str()
        .to_str()
        .unwrap()
        .split('/')
        .last()
        .unwrap()
        .len();

    let mut buf = format!("{:width$}", "legend:", width = (fname_width + 1));
    buf.push_str("| f:   files | r:     reg | d:     dir | l:    link | c: created | d: deleted | x:reg.xfer |");

    // Acquire the lock on the log file
    let mut lock = LOG_FILE.lock().unwrap();

    // Write the header line to the log file
    writeln!(lock, "{}", buf)?;

    Ok(())
}

fn process_log_file(filename: &PathBuf) -> Result<(), Error> {

    // Open the file for reading
    let mut file = File::open(filename.clone())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let fields: Vec<(&str, &str)> = vec![
        ("of files:", "f"),
        ("created files:", "c"),
        ("deleted files:", "d"),
        ("regular files transferred:", "x"),
    ];
    let subfields: Vec<(&str, &str)> = vec![("reg:", "r"), ("dir:", "d"), ("link:", "l")];

    // Process the content 
    let mut buffer = String::from("");
    let lines: Vec<&str> = contents.lines().collect();
    let fname = filename.as_os_str().to_str().unwrap();

    buffer.push_str(format!("{} | ", fname.split('/').last().unwrap()).as_str());
    for line in lines {
        for field in fields.iter() {
            if line.contains(field.0) {
                let s: Vec<&str> = line.split(field.0).collect();
                let value = s[1].trim_start().split(' ').next().unwrap().trim_end();
                buffer.push_str(format!("{:1}: {:>7} | ", field.1, value).as_str());
                if field.0 == "of files:" {
                    for subfield in subfields.iter() {
                        if line.contains(subfield.0) {
                            let sf: Vec<&str> = line.split(subfield.0).collect();
                            let svalue = sf[1]
                                .trim_start()
                                .split(' ')
                                .next()
                                .unwrap()
                                .trim_end()
                                .trim_end_matches([',', ')']);
                            buffer.push_str(
                                format!("{:1}: {:>7} | ", subfield.1, svalue).as_str(),
                            );
                        }
                    }
                }
            }
        }
    }

    // Acquire the lock on the log file
    let mut lock = LOG_FILE.lock().unwrap();

    // Write the processed content to the log file
    writeln!(lock, "{}", buffer)?;

    Ok(())
}

fn sort_and_rewrite_file(file_path: &str) -> Result<(), Error> {
    let mut contents = fs::read_to_string(file_path)?;
    let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

    lines.sort();
    lines.reverse();
    contents = lines.join("\n");

    let mut file = fs::File::create(file_path)?;
    write!(file, "{}", contents)?;

    Ok(())
}

