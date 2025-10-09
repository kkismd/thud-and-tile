use std::io::{self, Write};

use super::{CliRenderSettings, Frame};

pub struct TerminalDriver {
    last_frame: Option<Frame>,
}

impl TerminalDriver {
    pub fn new() -> Self {
        Self { last_frame: None }
    }

    pub fn present_full_frame(
        &mut self,
        frame: Frame,
        settings: &CliRenderSettings,
    ) -> io::Result<()> {
        if frame.requires_clear || settings.double_buffering {
            print!("\x1B[2J\x1B[H");
        } else {
            print!("\x1B[H");
        }

        for line in &frame.lines {
            println!("{}", line);
        }

        io::stdout().flush()?;
        self.last_frame = Some(frame);
        Ok(())
    }

    pub fn present_incremental_frame(
        &mut self,
        frame: Frame,
        settings: &CliRenderSettings,
    ) -> io::Result<()> {
        if self.last_frame.is_none() || frame.requires_clear || settings.double_buffering {
            return self.present_full_frame(frame, settings);
        }

        let previous = self.last_frame.as_ref().unwrap();
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let new_len = frame.lines.len();
        let old_len = previous.lines.len();

        for (idx, line) in frame.lines.iter().enumerate() {
            let old_line = previous.lines.get(idx);
            if old_line.map(|value| value != line).unwrap_or(true) {
                write!(handle, "\x1B[{};1H{}\x1B[0K", idx + 1, line)?;
            }
        }

        if new_len < old_len {
            for idx in new_len..old_len {
                write!(handle, "\x1B[{};1H\x1B[0K", idx + 1)?;
            }
        }

        handle.flush()?;
        self.last_frame = Some(frame);
        Ok(())
    }
}
