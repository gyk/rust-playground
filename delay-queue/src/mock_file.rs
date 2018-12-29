#![cfg(test)]

use crate::delay_task::Task;

// A mock file struct
pub struct File {
    pub content: Vec<u8>,
    pub path: String,
}

fn remove_file(file_path: &str) {
    println!("Shredding '{}'...", file_path);
}

impl Task for File {
    fn execute(self) {
        remove_file(&self.path);
    }
}
