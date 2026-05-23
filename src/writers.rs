use noodles::cram;
use noodles::fasta;
use noodles::sam::{self, alignment::io::Write};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;

// Import from main module
use crate::misc::push_ext;

type CW = cram::io::Writer<BufWriter<File>>;
pub struct CramWriter {
    writer: CW,
    header: sam::Header,
}

impl CramWriter {
    pub fn new(
        output_path: &PathBuf,
        repository: &fasta::Repository,
        header: &sam::Header,
    ) -> io::Result<Self> {
        let mut writer: CW = Self::setup_writer(output_path, repository)?;
        writer.write_header(&header)?;
        Ok(Self {
            writer: writer,
            header: header.clone(),
        })
    }

    fn setup_writer(output_path: &PathBuf, repository: &fasta::Repository) -> io::Result<CW> {
        let buf_writer = File::create(&output_path).map(BufWriter::new)?;
        Ok(cram::io::writer::Builder::default()
            .set_reference_sequence_repository(repository.clone())
            .build_from_writer(buf_writer))
    }

    pub fn write<R>(&mut self, record: &R) -> io::Result<()>
    where
        R: sam::alignment::Record,
    {
        self.writer.write_alignment_record(&self.header, record)
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.writer.try_finish(&self.header)
    }

    pub fn index(s: &PathBuf) -> Result<(), std::io::Error> {
        let index_filename = push_ext(&s, ".crai");
        let mut idx_writer = File::create(&index_filename)
            .map(BufWriter::new)
            .map(cram::crai::io::Writer::new)?;
        let cram_index = cram::fs::index(&s)?;
        idx_writer.write_index(&cram_index)
    }
}
// TODO implement reader as a struct
// https://github.com/abdenlab/oxbow/blob/a3a1ecd96ac6dd834e8b0b4a4579efdd553757ef/oxbow/src/alignment/format/cram.rs#L278
