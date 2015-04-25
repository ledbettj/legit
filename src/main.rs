extern crate argparse;
extern crate crypto;
extern crate git2;
extern crate rand;
extern crate time;

use std::fs::File;
use std::io::*;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use argparse::{ArgumentParser,Store};
use git2::Repository;

use worker::Worker;

mod worker;

struct Options {
    threads: u32,
    target:  String,
    message: String,
    repo:    String
}

fn main() {
    let mut opts = Options{
        threads: 8,
        target:  "000000".to_string(),
        message: "default commit message".to_string(),
        repo:    ".".to_string()
    };

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Generate git commit sha with a custom prefix");

        ap.refer(&mut opts.repo)
            .add_argument("repository-path", Store, "Path to your git repository (required)")
            .required();

        ap.refer(&mut opts.target)
            .add_option(&["-p", "--prefix"], Store, "Desired commit prefix (required)")
            .required();

        ap.refer(&mut opts.threads)
            .add_option(&["-t", "--threads"], Store, "Number of worker threads to use (default 8)");

        ap.refer(&mut opts.message)
            .add_option(&["-m", "--message"], Store, "Commit message to use (required)")
            .required();

        ap.parse_args_or_exit();
    }

    let (tx, rx) = channel();
    let start = time::get_time();
    let mut repo = match Repository::open(&opts.repo) {
        Ok(r) => r,
        Err(e) => panic!("failed to open {}: {}", &opts.repo, e)
    };
    let (tree, parent) = get_repo_info(&mut repo);
    let author         = get_author(&repo);

    for i in 0..opts.threads {
        let thread_tx     = tx.clone();
        let thread_target = opts.target.clone();
        let (t, p, a, m) = (tree.clone(), parent.clone(), author.clone(), opts.message.clone());

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

    apply_commit(&opts.repo, &hash, &blob);

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
