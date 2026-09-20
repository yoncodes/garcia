use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::Direction;

pub(crate) struct Capture {
    directory: PathBuf,
    index: File,
    sequence: u64,
}

impl Capture {
    pub(crate) fn start(base: &Path, conversation: u32) -> io::Result<Self> {
        let timestamp = common::time::ServerTime::now_millis();
        let directory = base.join(format!("{timestamp}-conv-{conversation}"));
        fs::create_dir_all(directory.join("c2g"))?;
        fs::create_dir_all(directory.join("g2c"))?;
        let index = OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("packets.jsonl"))?;
        tracing::info!(path = %directory.display(), "packet capture started");
        Ok(Self {
            directory,
            index,
            sequence: 0,
        })
    }

    pub(crate) fn write(
        &mut self,
        direction: Direction,
        command_id: u16,
        command_name: &str,
        packet: &[u8],
    ) -> io::Result<()> {
        self.sequence += 1;
        let file_name = format!(
            "{:06}_{}_cmd-{}_{}.bin",
            self.sequence,
            direction.label(),
            command_id,
            command_name
        );
        fs::write(
            self.directory.join(direction.label()).join(&file_name),
            packet,
        )?;
        writeln!(
            self.index,
            "{{\"sequence\":{},\"direction\":\"{}\",\"cmd_id\":{},\"cmd_name\":\"{}\",\"length\":{},\"file\":\"{}/{}\"}}",
            self.sequence,
            direction.label(),
            command_id,
            command_name,
            packet.len(),
            direction.label(),
            file_name
        )?;
        self.index.flush()
    }
}
