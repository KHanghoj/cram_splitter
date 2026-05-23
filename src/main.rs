mod misc;
mod readers;
mod split;
mod writers;

use clap::Parser;
use readers::{CramReader, Region};
use split::split_reads;
use std::path::PathBuf;
use writers::CramWriter;
// pub const BUFFER_SIZE_BYTES: usize = const { 1024 * 1024 };

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input CRAM file path
    #[arg(long)]
    cram: PathBuf,

    /// Regions file path (format: chromosome,start,end,filename)
    #[arg(long)]
    regions: PathBuf,

    // Reference genome file (required for CRAM)
    #[arg(long)]
    reference: PathBuf,

    // bp padding
    #[arg(long, default_value_t = 5000)]
    padding: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("{:?}", args);
    let mut cram_reader = CramReader::new(&args.cram, &args.reference)?;
    let regions: Vec<Region> =
        Region::parse_regions(&args.regions, &cram_reader.header, args.padding)?;
    split_reads(&mut cram_reader, &regions)?;
    regions
        .iter()
        .try_for_each(|x| CramWriter::index(&x.filename))?;

    // this is the same code but using dict. will test if faster later
    // let regions: HashMap<usize, Vec<Region>> =
    //     Region::parse_regions_dict(&args.regions, &cram_reader.header, args.padding)?;
    // println!("{:?}", regions);
    // split_reads_dict(&mut cram_reader, &regions)?;
    // regions.iter().try_for_each(|(_, regions)| {
    //     regions.iter().try_for_each(|reg| index_cram(&reg.filename))
    // })?;

    // TODO implement it with containers as it seems way way faster
    // https://github.com/abdenlab/oxbow/blob/a3a1ecd96ac6dd834e8b0b4a4579efdd553757ef/oxbow/src/alignment/format/cram.rs#L536
    Ok(())
}
