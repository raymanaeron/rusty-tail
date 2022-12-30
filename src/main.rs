use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rev_buf_reader::RevBufReader;
use std::fs::File;
use std::io::{BufRead, Seek, SeekFrom};
use std::path::Path;
use std::process;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    // Get the path to the file to tail from the command line arguments
    let path = std::env::args().nth(1).expect("Missing file path..");
    println!("Watching file: {}. Press CTRL+C to exit.", path);

    // Tail the file
    if let Err(e) = watch(path) {
        println!("error: {:?}", e);
        process::exit(1)
    }
}

// Watches the file for changes and reads the last line
fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    // Open the file
    let mut file = File::open(path.as_ref()).unwrap();

    // Move the cursor to the end of the file
    file.seek(SeekFrom::End(0)).unwrap();

    // Send/receive channels
    let (tx, rx) = channel();

    // Configure the file watcher
    let config = Config::default()
        .with_compare_contents(false)
        .with_poll_interval(Duration::from_secs(1));

    // Create a file watcher
    let mut watcher = RecommendedWatcher::new(tx, config)?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(_event) => {
                // Read the last line from the file
                let lines = lines_from_file(&mut file, 1);

                // Whatever we want to do with the read. In this case, we are just writing it to the console
                if lines.len() > 0 {
                    println!("{:?}", lines[0]);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }

        // Pause the program for a short time between each file read
        std::thread::sleep(Duration::from_millis(100));
    }
}

// Reads the last line from the file
// Assumes that each line was saved by the file owner before writing a new line
fn lines_from_file(file: &mut File, limit: usize) -> Vec<String> {
    let reader = RevBufReader::new(file);
    reader
        .lines()
        .take(limit)
        .map(|l| l.expect("Could not parse line"))
        .collect()
}
