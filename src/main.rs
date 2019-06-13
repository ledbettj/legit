use argparse::{ArgumentParser,Store};

use gitminer::Gitminer;

mod worker;
mod gitminer;

fn main() {
    let mut opts = gitminer::Options{
        threads: 8,
        target:  "000000".to_string(),
        message: "default commit message".to_string(),
        repo:    ".".to_string(),
        timestamp: time::now()
    };

    parse_args_or_exit(&mut opts);

    let start = time::get_time();
    let mut miner = match Gitminer::new(opts) {
        Ok(m)  => m,
        Err(e) => { panic!("Failed to start git miner: {}", e); }
    };

    let hash = match miner.mine() {
        Ok(s)  => s,
        Err(e) => { panic!("Failed to generate commit: {}", e); }
    };

    let duration = time::get_time() - start;
    println!("Success! Generated commit {} in {} seconds", hash, duration.num_seconds());
}

fn parse_args_or_exit(opts: &mut gitminer::Options) {
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

    //ap.refer(&mut opts.timestamp)
    //    .add_option(&["--timestamp"], Store, "Commit timestamp to use (default now)");

    ap.parse_args_or_exit();
}
