extern crate argparse;
extern crate crypto;
extern crate git2;
extern crate rand;
extern crate time;

use std::env;
use std::fs::File;
use std::io::*;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use git2::Repository;

use worker::Worker;

mod worker;

static DEFAULT_THREAD_COUNT : u32 = 8;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("usage: {} <path-to-repo> [target]\n", args[0]);
    }

    let thread_count = DEFAULT_THREAD_COUNT;
    let target = if args.len() > 2 { args[2].clone() } else { "00000".to_string() };
    let (tx, rx) = channel();
    let start = time::get_time();
    let mut repo = match Repository::open(&args[1]) {
        Ok(r) => r,
        Err(e) => panic!("failed to open {}: {}", &args[1], e)
    };
    let (tree, parent) = get_repo_info(&mut repo);

    for i in 0..thread_count {
        let thread_tx     = tx.clone();
        let thread_target = target.clone();
        let (t, p) = (tree.clone(), parent.clone());

        thread::spawn(move || {
            Worker::new(
                i,
                thread_target,
                t,
                p,
                "message".to_string(),
                thread_tx).work();
        });
    };

    let (id, blob, hash) = rx.recv().ok().expect("Recv failed");
    let duration = time::get_time() - start;
    println!("success! worker {:02} generated commit {} in {}s",
             id,
             hash,
             duration.num_seconds());

    let tmpfile = format!("/tmp/{}-{}.tmp", id, hash);

    let mut file = File::create(&Path::new(&tmpfile)).ok().expect("File create failed");

    file.write_all(blob.as_bytes()).ok().expect("File write failed");
    Command::new("sh")
        .arg("-c")
        .arg(format!("cd {} && git hash-object -t commit -w --stdin < {} && git reset --hard {}",
                     args[1], tmpfile, hash))
        .output().ok().expect("Failed to generate commit");

    println!("All done! Enjoy your new commit.")
}


fn get_repo_info(repo: &mut Repository) -> (String, String) {
    let head = repo.revparse_single("HEAD").ok().expect("can't parse HEAD");
    let mut index = repo.index().ok().expect("can't get index");
    let tree = index.write_tree().ok().expect("can't write tree");


    let head_s = format!("{}", head.id());
    let tree_s = format!("{}", tree);

    (tree_s, head_s)
}
