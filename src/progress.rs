//! Module that provides functions to display progress bars for compressing and decompressing files.
use std::{
    io::{self, Cursor},
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};

struct DisplayHandle {
    buf: Vec<u8>,
    sender: Sender<String>,
}
impl io::Write for DisplayHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.sender.send(String::from_utf8(self.buf.drain(..).collect()).unwrap()).unwrap();
        Ok(())
    }
}

/// Draw a ProgressBar using an io::Cursor to check periodically for the progress (for zip archives)
pub struct ProgressByCursor {
    draw_stop: Sender<()>,
    display_handle: DisplayHandle,
}
impl ProgressByCursor {
    /// Create a ProgressBar using an io::Cursor to check periodically for the progress (for zip archives)
    /// If precise is true, the total_input_size will be displayed as the total_bytes size
    /// # Safety
    /// The pointer to the cursor must be valid and remain valid until the ProgressBar is dropped.
    pub unsafe fn new(total_input_size: u64, precise: bool, cursor: *const Cursor<Vec<u8>>) -> Self {
        let cursor = {
            struct SendPtr(*const Cursor<Vec<u8>>);
            unsafe impl Send for SendPtr {}
            SendPtr(cursor)
        };
        let template = if precise {
            "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
        } else {
            "{spinner:.green} {prefix} [{elapsed}] [{wide_bar:.cyan/blue}] {bytes}/?? ({bytes_per_sec}, {eta}) {path}"
        };
        let (draw_tx, draw_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        thread::spawn(move || {
            let cursor = {
                let c = cursor;
                c.0
            };
            let pb = ProgressBar::new(total_input_size);
            pb.set_style(ProgressStyle::default_bar().template(template).progress_chars("#>-"));
            while draw_rx.try_recv().is_err() {
                // Safety:
                // - The pointer validity is guaranteed by the contract of the `new` function.
                // - We don't care if the value is written underneath us (its just an approximation anyway)
                pb.set_position(unsafe { &*cursor }.position() as u64);
                if let Ok(msg) = msg_rx.try_recv() {
                    pb.set_prefix(msg);
                }
                thread::sleep(Duration::from_millis(100));
            }
            pb.finish();
        });
        ProgressByCursor { draw_stop: draw_tx, display_handle: DisplayHandle { buf: Vec::new(), sender: msg_tx } }
    }

    pub(crate) fn display_handle(&mut self) -> &mut impl io::Write {
        &mut self.display_handle
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
    display_handle: DisplayHandle,
}
impl ProgressByPath {
    /// Create a ProgressBar using a path to an output file to check periodically for the progress
    /// If precise is true, the total_input_size will be displayed as the total_bytes size
    pub fn new(total_input_size: u64, precise: bool, output_file_path: PathBuf) -> Self {
        //NOTE: canonicalize is here to avoid a weird bug:
        //      > If output_file_path is a nested path and it exists and the user overwrite it
        //      >> output_file_path.exists() will always return false (somehow)
        //      - canonicalize seems to fix this
        let output_file_path = output_file_path.canonicalize().unwrap();

        let (draw_tx, draw_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        let template = if precise {
            "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
        } else {
            "{spinner:.green} {prefix} [{elapsed}] [{wide_bar:.cyan/blue}] {bytes}/?? ({bytes_per_sec}, {eta}) {path}"
        };
        thread::spawn(move || {
            let pb = ProgressBar::new(total_input_size);
            pb.set_style(ProgressStyle::default_bar().template(template).progress_chars("#>-"));
            while draw_rx.try_recv().is_err() {
                if let Ok(msg) = msg_rx.try_recv() {
                    pb.set_prefix(msg);
                }
                pb.set_position(output_file_path.metadata().unwrap().len());
                thread::sleep(Duration::from_millis(100));
            }
            pb.finish();
        });
        ProgressByPath { draw_stop: draw_tx, display_handle: DisplayHandle { buf: Vec::new(), sender: msg_tx } }
    }

    pub(crate) fn display_handle(&mut self) -> &mut impl io::Write {
        &mut self.display_handle
    }
}
impl Drop for ProgressByPath {
    fn drop(&mut self) {
        let _ = self.draw_stop.send(());
    }
}
