// takes cram_reader, regions
use crate::CramReader;
use crate::CramWriter;
use crate::Region;
use std::collections::HashMap;
use std::io;

pub fn split_reads(cram_reader: &mut CramReader, regions: &Vec<Region>) -> io::Result<()> {
    // Create writers for each region
    let mut writers: Vec<CramWriter> = regions
        .iter()
        .map(|x| CramWriter::new(&x.filename, &cram_reader.repo, &cram_reader.header))
        .collect::<Result<Vec<_>, _>>()?;
    println!("Created {} writers", writers.len());

    // Process all records
    let mut total_records = 0;
    let mut written_records = 0;

    for record_result in cram_reader.reader.records(&cram_reader.header) {
        let record = record_result?;
        total_records += 1;

        // Check which regions this record overlaps
        match (
            record.reference_sequence_id(),
            record.alignment_start(),
            record.alignment_end(),
        ) {
            (Some(ref_seq_id), Some(start), Some(end)) => {
                let start_pos = start.get();
                let end_pos = end.get();
                regions
                    .iter()
                    .zip(writers.iter_mut())
                    .filter(|(region, _writer)| region.overlaps(ref_seq_id, start_pos, end_pos))
                    .try_for_each(|(_region, writer)| {
                        writer.write(&record).map(|_| written_records += 1)
                    })?;
            }
            _ => {}
        }
        if total_records % 1000000 == 0 {
            println!(
                "Processed {} records, written {} times",
                total_records, written_records
            );
        }
        // if total_records == 5000 {
        //     break;
        // }
    }

    // Finish all writers
    writers.iter_mut().try_for_each(|writer| writer.close())?;
    println!(
        "Finished! Processed {} total records, written {} times across regions",
        total_records, written_records
    );

    Ok(())
}

fn _split_reads_dict(
    cram_reader: &mut CramReader,
    regions: &HashMap<usize, Vec<Region>>,
) -> io::Result<()> {
    // Create writers for each region across chromosomes
    let mut writers: HashMap<usize, Vec<CramWriter>> = regions
        .iter()
        .map(|(tid, region_vec)| {
            region_vec
                .iter()
                .map(|region| {
                    CramWriter::new(&region.filename, &cram_reader.repo, &cram_reader.header)
                })
                // magically pulls the Result out
                .collect::<Result<Vec<CramWriter>, io::Error>>()
                .map(|writers| (*tid, writers))
        })
        // magically pulls the Result out
        .collect::<Result<HashMap<usize, Vec<CramWriter>>, io::Error>>()?;

    println!(
        "Created {} writers",
        writers.values().map(|w| w.len()).sum::<usize>()
    );

    // Process all records
    let mut total_records = 0;
    let mut written_records = 0;

    for record_result in cram_reader.reader.records(&cram_reader.header) {
        let record = record_result?;
        total_records += 1;

        // Check which regions this record overlaps
        if let Some(ref_seq_id) = record.reference_sequence_id() {
            if let Some(writer_chr) = writers.get_mut(&ref_seq_id) {
                if let Some(region_chr) = regions.get(&ref_seq_id) {
                    if let (Some(start), Some(end)) =
                        (record.alignment_start(), record.alignment_end())
                    {
                        // if let (Some(start), Some(end)) = (record.alignment_start(), record.alignment_start()) {
                        let start_pos = start.get();
                        let end_pos = end.get();

                        for (region_idx, region) in region_chr.iter().enumerate() {
                            if region.overlaps(ref_seq_id, start_pos, end_pos) {
                                writer_chr[region_idx].write(&record)?;
                                written_records += 1;
                            }
                        }
                    }
                }
            }
        }

        if total_records % 1000000 == 0 {
            println!(
                "Processed {} records, written {} times",
                total_records, written_records
            );
        }
        if total_records == 5000000 {
            break;
        }
    }

    // Finish all writers
    writers
        .iter_mut()
        .try_for_each(|(_, writer_vec)| writer_vec.iter_mut().try_for_each(|w| w.close()))?;
    println!(
        "Finished! Processed {} total records, written {} times across regions",
        total_records, written_records
    );

    Ok(())
}
