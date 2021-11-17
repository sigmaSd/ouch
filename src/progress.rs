//! Module that provides functions to display progress bars for compressing and decompressing files.
use std::{
    io,
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

/// Draw a ProgressBar using an function that checks periodically for the progress
pub struct Progress {
    draw_stop: Sender<()>,
    display_handle: DisplayHandle,
}
impl Progress {
    /// Create a ProgressBar using a function that checks periodically for the progress
    /// If precise is true, the total_input_size will be displayed as the total_bytes size
    pub fn new(total_input_size: u64, precise: bool, current_position_fn: Box<dyn Fn() -> u64 + Send>) -> Self {
        let (draw_tx, draw_rx) = mpsc::channel();
        let (msg_tx, msg_rx) = mpsc::channel();

        thread::spawn(move || {
            let template = if precise {
                "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            } else {
                "{spinner:.green} {prefix} [{elapsed}] [{wide_bar:.cyan/blue}] {bytes}/?? ({bytes_per_sec}, {eta}) {path}"
            };
            let pb = ProgressBar::new(total_input_size);
            pb.set_style(ProgressStyle::default_bar().template(template).progress_chars("#>-"));

            while draw_rx.try_recv().is_err() {
                pb.set_position(current_position_fn());
                if let Ok(msg) = msg_rx.try_recv() {
                    pb.set_prefix(msg);
                }
                thread::sleep(Duration::from_millis(100));
            }
            pb.finish();
        });
        Progress { draw_stop: draw_tx, display_handle: DisplayHandle { buf: Vec::new(), sender: msg_tx } }
    }

    pub(crate) fn display_handle(&mut self) -> &mut impl io::Write {
        &mut self.display_handle
    }
}
impl Drop for Progress {
    fn drop(&mut self) {
        let _ = self.draw_stop.send(());
    }
}
