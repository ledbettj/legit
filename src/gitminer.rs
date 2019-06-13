use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use super::worker::Worker;

pub struct Options {
    pub threads:   u32,
    pub target:    String,
    pub message:   String,
    pub repo:      String,
    pub timestamp: time::Tm,
}

pub struct Gitminer {
    opts:   Options,
    repo:   git2::Repository,
    author: String
}


impl Gitminer {
    pub fn new(opts: Options) -> Result<Gitminer, &'static str> {

        let repo = match git2::Repository::open(&opts.repo) {
            Ok(r)  => r,
            Err(_) => { return Err("Failed to open repository"); }
        };

        let author = Gitminer::load_author(&repo)?;

        Ok(Gitminer {
            opts:   opts,
            repo:   repo,
            author: author
        })
    }

    pub fn mine(&mut self) -> Result<String, &'static str> {
        let (tree, parent) = match Gitminer::prepare_tree(&mut self.repo) {
            Ok((t, p)) => (t, p),
            Err(e)   => { return Err(e); }
        };

        let (tx, rx) = channel();

        for i in 0..self.opts.threads {
            let target = self.opts.target.clone();
            let author = self.author.clone();
            let msg    = self.opts.message.clone();
            let wtx    = tx.clone();
            let ts     = self.opts.timestamp.clone();
            let (wtree, wparent) = (tree.clone(), parent.clone());

            thread::spawn(move || {
                Worker::new(i, target, wtree, wparent, author, msg, ts, wtx).work();
            });
        }

        let (_, blob, hash) = rx.recv().unwrap();

        match self.write_commit(&hash, &blob) {
            Ok(_)  => Ok(hash),
            Err(e) => Err(e)
        }
    }

    fn write_commit(&self, hash: &String, blob: &String) -> Result<(), &'static str> {
        /* repo.blob() generates a blob, not a commit.
         * don't know if there's a way to do this with libgit2. */
        let tmpfile  = format!("/tmp/{}.tmp", hash);
        let mut file = File::create(&Path::new(&tmpfile))
            .ok()
            .expect(&format!("Failed to create temporary file {}", &tmpfile));

        file.write_all(blob.as_bytes())
            .ok()
            .expect(&format!("Failed to write temporary file {}", &tmpfile));

        Command::new("sh")
            .arg("-c")
            .arg(format!("cd {} && git hash-object -t commit -w --stdin < {} && git reset --hard {}", self.opts.repo, tmpfile, hash))
            .output()
            .ok()
            .expect("Failed to generate commit");

        Ok(())
    }


    fn load_author(repo: &git2::Repository) -> Result<String, &'static str> {
        let cfg = match repo.config() {
            Ok(c)  => c,
            Err(_) => { return Err("Failed to load git config"); }
        };

        let name  = match cfg.get_string("user.name") {
            Ok(s)  => s,
            Err(_) => { return Err("Failed to find git user name"); }
        };

        let email = match cfg.get_string("user.email") {
            Ok(s)  => s,
            Err(_) => { return Err("Failed to find git email address"); }
        };

        Ok(format!("{} <{}>", name, email))
    }

    fn prepare_tree(repo: &mut git2::Repository) -> Result<(String, String), &'static str> {
        Gitminer::ensure_no_unstaged_changes(repo)?;

        let head      = repo.revparse_single("HEAD").unwrap();
        let mut index = repo.index().unwrap();
        let tree      = index.write_tree().unwrap();

        let head_s = format!("{}", head.id());
        let tree_s = format!("{}", tree);

        Ok((tree_s, head_s))
    }

    fn ensure_no_unstaged_changes(repo: &mut git2::Repository) -> Result<(), &'static str> {
        let mut opts = git2::StatusOptions::new();
        let mut m    = git2::Status::empty();
        let statuses = repo.statuses(Some(&mut opts)).unwrap();

        m.insert(git2::Status::WT_NEW);
        m.insert(git2::Status::WT_MODIFIED);
        m.insert(git2::Status::WT_DELETED);
        m.insert(git2::Status::WT_RENAMED);
        m.insert(git2::Status::WT_TYPECHANGE);

        for i in 0..statuses.len() {
            let status_entry = statuses.get(i).unwrap();
            if status_entry.status().intersects(m) {
                return Err("Please stash all unstaged changes before running.");
            }
        }

        Ok(())
    }

}
