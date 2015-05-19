use std::sync::mpsc;
use crypto::digest::Digest;
use crypto::sha1;
use time;

pub struct Worker {
    id:      u32,
    digest:  sha1::Sha1,
    tx:      mpsc::Sender<(u32, String, String)>,
    target:  String,
    tree:    String,
    parent:  String,
    author:  String,
    message: String,
    timestamp: time::Tm
}

impl Worker {
    pub fn new(id:        u32,
               target:    String,
               tree:      String,
               parent:    String,
               author:    String,
               message:   String,
               timestamp: time::Tm,
               tx:        mpsc::Sender<(u32, String, String)>) -> Worker {
        Worker {
            id:        id,
            digest:    sha1::Sha1::new(),
            tx:        tx,
            target:    target,
            tree:      tree,
            parent:    parent,
            author:    author,
            message:   message,
            timestamp: timestamp
        }
    }

    pub fn work(&mut self) {
        let tstamp = format!("{}", self.timestamp.strftime("%s %z").unwrap());

        let mut value  = 0u32;
        loop {
            let (raw, blob) = self.generate_blob(value, &tstamp);
            let result = self.calculate(&blob);

            if result.starts_with(&self.target) {
                self.tx.send((self.id, raw, result)).unwrap();
                break;
            }

            value += 1;
        }
    }

    fn generate_blob(&mut self, value: u32, tstamp: &str) -> (String, String) {
        let raw = format!("tree {}\n\
                           parent {}\n\
                           author {} {}\n\
                           committer {} {}\n\n\
                           {}\n{:02}-{:08x}",
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
