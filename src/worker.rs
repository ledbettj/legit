use std::sync::mpsc;
use crypto::digest::Digest;
use crypto::sha1;
use rand;
use rand::Rng;
use time;

pub struct Worker {
    id:      u32,
    digest:  sha1::Sha1,
    rng:     rand::ThreadRng,
    tx:      mpsc::Sender<(u32, String, String)>,
    target:  String,
    tree:    String,
    parent:  String,
    author:  String,
    message: String
}

impl Worker {
    pub fn new(id:      u32,
               target:  String,
               tree:    String,
               parent:  String,
               author:  String,
               message: String,
               tx:      mpsc::Sender<(u32, String, String)>) -> Worker {
        Worker {
            id:      id,
            digest:  sha1::Sha1::new(),
            rng:     rand::thread_rng(),
            tx:      tx,
            target:  target,
            tree:    tree,
            parent:  parent,
            author:  author,
            message: message
        }
    }

    pub fn work(&mut self) {

        loop {
            let value = self.rng.next_u32();
            let (raw, blob) = self.generate_blob(value);
            let result = self.calculate(&blob);

            if result.starts_with(&self.target) {
                self.tx.send((self.id, raw, result));
                break;
            }
        }
    }

    fn generate_blob(&mut self, value: u32) -> (String, String) {
        let tstamp = time::now_utc().to_timespec().sec;
        let raw = format!("tree {}\n\
                           parent {}\n\
                           author {} {} +0000\n\
                           committer {} {} +0000\n\n\
                           {} ({:02}-{:08x})",
                          self.tree,
                          self.parent,
                          self.author, tstamp,
                          self.author, tstamp,
                          self.message,
                          self.id,
                          value);
        let blob = format!("commit {}\0{}", raw.len(), raw);

        (raw, blob)
    }

    fn calculate(&mut self, blob: &str) -> String {
        self.digest.reset();
        self.digest.input_str(blob);

        self.digest.result_str()
    }
}
