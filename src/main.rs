use glob::glob;
use io::{BufRead, BufReader};
use process::{Command, Stdio};
use std::{collections, io, process, thread, time};
extern crate scoped_threadpool;

fn process_path(path: &str) {
    let mut child_shell = Command::new("transcode-video")
        .args(["-v", "--quick", path])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");
    let child_out = BufReader::new(child_shell.stdout.as_mut().unwrap());
    let mut pool = scoped_threadpool::Pool::new(1);

    pool.scoped(|scope| {
        scope.execute(move || {
            let reader = std::io::BufReader::new(child_out);
            for line in reader.lines() {
                println!("{}", line.unwrap());
            }
        });
    });
    child_shell.wait().unwrap();
}

fn watch_directory(path: &str) -> Result<(), io::Error> {
    let one_minute = time::Duration::from_secs(60);

    loop {
        let mut already_processd = collections::HashSet::new();
        let path = path.to_string();
        let file_types = "/**/*.mkv";
        let pattern = path + file_types;

        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path = &path.into_os_string().into_string().unwrap();
                    if already_processd.contains(&path.to_string()) {
                        continue;
                    }
                    process_path(path);
                    already_processd.insert(path.to_string());
                }
                Err(e) => println!("Error while looking for patern  {:?}", e),
            }
        }
        thread::sleep(one_minute);
    }
}

fn main() {
    let mut args = argv::iter();
    args.next();

    let path = args.next();
    match path {
        Some(p) => {
            watch_directory(&p.to_string_lossy()).unwrap();
        }
        None => println!("No path provided"),
    }
}
