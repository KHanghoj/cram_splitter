use csv;
use noodles::cram;
use noodles::fasta;
use noodles::sam;
use serde;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;

use crate::misc::push_ext;

#[derive(Debug, serde::Deserialize)]
pub struct CSVRow {
    pub chromosome: String,
    pub start: usize,
    pub end: usize,
    pub filename: PathBuf,
}

pub struct Region {
    chromosome_tid: usize,
    start: usize,
    end: usize,
    pub filename: PathBuf,
}

impl Region {
    pub fn overlaps(&self, tid: usize, pos: usize, end_pos: usize) -> bool {
        self.chromosome_tid == tid && self.start <= end_pos && self.end >= pos
    }

    fn _get_tid_map(header: &sam::Header) -> Result<HashMap<String, usize>, io::Error> {
        header
            .reference_sequences()
            .keys()
            .map(
                |chrom| match header.reference_sequences().get_index_of(chrom) {
                    Some(tid) => Ok((chrom.to_string(), tid)),
                    None => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Chromosome not found",
                    )),
                },
            )
            .collect::<Result<HashMap<String, usize>, _>>()
    }

    fn process_row(row: CSVRow, header: &sam::Header, padding: usize) -> Result<Self, io::Error> {
        let chr_tid = header
            .reference_sequences()
            .get_index_of(row.chromosome.as_bytes())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Chromosome '{}' not in header of cram", row.chromosome,),
                )
            })?;

        let padded_start = match padding >= row.start {
            true => 1,
            false => row.start - padding,
        };
        let padded_end = row.end + padding;

        Ok(Self {
            chromosome_tid: chr_tid,
            start: padded_start,
            end: padded_end,
            filename: row.filename,
        })
    }

    pub fn parse_regions(
        regions_file: &PathBuf,
        header: &sam::Header,
        padding: usize,
    ) -> Result<Vec<Self>, io::Error> {
        let mut reader = csv::Reader::from_path(&regions_file)?;

        let regions: Vec<Self> = reader
            .deserialize()
            .map(|record| {
                match record {
                    Ok(row) => Self::process_row(row, header, padding),
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                }
                // Same as above.
                // Possible to use ? when the return value is set
                // let row: CSVRow = record?;
                // Self::process_row(row, &tid_map, padding)
            })
            .collect::<Result<Vec<Self>, io::Error>>()?;

        println!("Loaded {} regions", regions.len());
        Ok(regions)
    }

    pub fn _parse_regions_dict(
        regions_file: &PathBuf,
        header: &sam::Header,
        padding: usize,
    ) -> Result<HashMap<usize, Vec<Region>>, io::Error> {
        let mut reader = csv::Reader::from_path(&regions_file)?;

        let regions: HashMap<usize, Vec<Self>> = reader
            .deserialize()
            .map(|record| match record {
                Ok(row) => Self::process_row(row, &header, padding),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            })
            .collect::<Result<Vec<Self>, io::Error>>()?
            .into_iter()
            .fold(HashMap::new(), |mut d, reg| {
                d.entry(reg.chromosome_tid)
                    .or_insert_with(Vec::new)
                    .push(reg);
                d
            });

        println!(
            "Loaded {} regions across {} chromosomes",
            regions.values().map(|v| v.len()).sum::<usize>(),
            regions.len()
        );
        Ok(regions)
    }
}

pub struct CramReader {
    pub reader: cram::io::Reader<File>,
    pub repo: fasta::Repository,
    pub header: sam::Header,
}

impl CramReader {
    pub fn new(cram: &PathBuf, fasta: &PathBuf) -> io::Result<Self> {
        let repo = Self::setup_repository(fasta)?;

        let mut reader = cram::io::reader::Builder::default()
            .set_reference_sequence_repository(repo.clone())
            .build_from_path(cram)?;
        let header = Self::get_header(&mut reader)?;

        Ok(Self {
            reader: reader,
            repo: repo,
            header: header,
        })
    }

    fn setup_repository(f: &PathBuf) -> io::Result<fasta::Repository> {
        let fai_path = push_ext(f, ".fai");
        let fai = if fai_path.exists() {
            println!("loads existing fai: {:?}", fai_path);
            fasta::fai::fs::read(fai_path)?
        } else {
            println!("generates fai from {:?}", f);
            fasta::fs::index(f)?
        };
        std::fs::File::open(f)
            .map(BufReader::new)
            .map(|f| fasta::io::IndexedReader::new(f, fai))
            .map(fasta::repository::adapters::IndexedReader::new)
            .map(fasta::Repository::new)
    }

    fn get_header(reader: &mut cram::io::Reader<File>) -> io::Result<sam::Header> {
        reader.read_header()
    }
}
