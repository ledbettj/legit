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

use argparse::{ArgumentParser,Store};
use git2::{Repository,Config};

use worker::Worker;

mod worker;

static DEFAULT_THREAD_COUNT : u32 = 8;

fn main() {
    let mut target       = "000000".to_string();
    let mut thread_count = DEFAULT_THREAD_COUNT;
    let mut message      = "default commit message".to_string();
    let mut repo_path    = ".".to_string();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Generate a custom git commit sha");

        ap.refer(&mut repo_path)
            .add_argument("path-to-repo", Store, "Path to your git repo")
            .required();

        ap.refer(&mut target)
            .add_option(&["-p", "--prefix"], Store, "Desired commit prefix")
            .required();

        ap.refer(&mut thread_count)
            .add_option(&["-t", "--threads"], Store, "Number of worker threads to use");

        ap.refer(&mut message)
            .add_option(&["-m", "--message"], Store, "Commit message to use")
            .required();

        ap.parse_args_or_exit();
    }

    let (tx, rx) = channel();
    let start = time::get_time();
    let mut repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => panic!("failed to open {}: {}", &repo_path, e)
    };
    let (tree, parent) = get_repo_info(&mut repo);
    let author         = get_author(&repo);

    for i in 0..thread_count {
        let thread_tx     = tx.clone();
        let thread_target = target.clone();
        let (t, p, a, m) = (tree.clone(), parent.clone(), author.clone(), message.clone());

        thread::spawn(move || {
            Worker::new(
                i,
                thread_target,
                t, p, a, m,
                thread_tx).work();
        });
    };

    let (id, blob, hash) = rx.recv().ok().expect("Recv failed");
    let duration = time::get_time() - start;

    println!("success! worker {:02} generated commit {} in {}s",
             id,
             hash,
             duration.num_seconds());

    apply_commit(&repo_path, &hash, &blob);

    println!("All done! Enjoy your new commit.");
}

fn apply_commit(git_root: &str, hash: &str, blob: &str) {
    let tmpfile  = format!("/tmp/{}.tmp", hash);
    let mut file = File::create(&Path::new(&tmpfile)).ok().expect("File create failed");

    file.write_all(blob.as_bytes()).ok().expect("File write failed");

    Command::new("sh")
        .arg("-c")
        .arg(format!("cd {} && git hash-object -t commit -w --stdin < {} && git reset --hard {}",
                     git_root, tmpfile, hash))
        .output().ok().expect("Failed to generate commit");
}


fn get_repo_info(repo: &mut Repository) -> (String, String) {
    let head = repo.revparse_single("HEAD").ok().expect("can't parse HEAD");
    let mut index = repo.index().ok().expect("can't get index");
    let tree = index.write_tree().ok().expect("can't write tree");


    let head_s = format!("{}", head.id());
    let tree_s = format!("{}", tree);

    (tree_s, head_s)
}

fn get_author(repo: &Repository) -> String {
    let cfg = match repo.config() {
        Ok(c) => c,
        Err(e) => panic!("Couldn't open git config: {}", e)
    };

    let name  = cfg.get_string("user.name").ok().expect("can't get git name");
    let email = cfg.get_string("user.email").ok().expect("can't get git email");

    format!("{} <{}>", name, email)
}
