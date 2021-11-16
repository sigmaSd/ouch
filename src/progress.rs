//! Module that provides functions to display progress bars for compressing and decompressing files.
use std::{
    io::Cursor,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};
/// Draw a ProgressBar using an io::Cursor to check periodically for the progress (for zip archives)
pub struct ProgressByCursor {
    draw_stop: Sender<()>,
}
impl ProgressByCursor {
    /// Create a ProgressBar using an io::Cursor to check periodically for the progress (for zip archives)
    /// # Safety
    /// The pointer to the cursor must be valid and remain valid until the ProgressBar is dropped.
    pub unsafe fn new(total_input_size: u64, cursor: *const Cursor<Vec<u8>>) -> Self {
        let cursor = {
            struct SendPtr(*const Cursor<Vec<u8>>);
            unsafe impl Send for SendPtr {}
            SendPtr(cursor)
        };
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let cursor = {
                let c = cursor;
                c.0
            };
            let pb = ProgressBar::new(total_input_size);
            pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
            while rx.try_recv().is_err() {
                thread::sleep(Duration::from_millis(100));
                // Safety:
                // - The pointer validity is guaranteed by the contract of the `new` function.
                // - We don't care if the value is written underneath us (its just an approximation anyway)
                pb.set_position(unsafe { &*cursor }.position() as u64);
            }
            pb.finish();
        });
        ProgressByCursor { draw_stop: tx }
    }
}
impl Drop for ProgressByCursor {
    fn drop(&mut self) {
        let _ = self.draw_stop.send(());
    }
}

/// Draw a ProgressBar using a path to an output file to check periodically for the progress
pub struct ProgressByPath {
    draw_stop: Sender<()>,
}
impl ProgressByPath {
    /// Create a ProgressBar using a path to an output file to check periodically for the progress
    pub fn new(total_input_size: u64, output_file_path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let pb = ProgressBar::new(total_input_size);
            pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
            while rx.try_recv().is_err() {
                thread::sleep(Duration::from_millis(100));
                pb.set_position(output_file_path.metadata().unwrap().len());
            }
            pb.finish();
        });
        ProgressByPath { draw_stop: tx }
    }
}
impl Drop for ProgressByPath {
    fn drop(&mut self) {
        let _ = self.draw_stop.send(());
    }
}
