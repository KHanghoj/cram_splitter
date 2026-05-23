# CRAM Splitter

A Rust tool for splitting large CRAM files into smaller region-specific files based on genomic coordinates.

## Features

- Split CRAM files by genomic regions specified in CSV format
- Configurable padding around target regions
- Efficient processing using the noodles bioinformatics library
- Automatic indexing of output files

## Usage

```bash
cram_splitter \
  --cram input.cram \
  --regions regions.csv \
  --reference reference.fa \
  --padding 5000
```

## Input Format

The regions CSV file should have the format: `chromosome,start,end,filename`

Example:
```
chromosome,start,end,filename
chr1,1000000,2000000,split_output/region1.cram
chr1,3000000,4000000,split_output/region2.cram
chr2,500000,1500000,split_output/region3.cram
```

## Command Line Options

- `--cram`: Input CRAM file path
- `--regions`: Regions CSV file path
- `--reference`: Reference genome file (required for CRAM)
- `--padding`: Base pair padding around regions (default: 5000)

---

## Setup

Setup input data for splitting

```bash
awk -v sample="test" -F, 'BEGIN{OFS=","}FNR==1{print $0,"filename";next} {print $0,sprintf("split_output/%s_%s_%s_%s.cram", sample, $1,$2,$3)}' data/intervals_15mb.csv > data/intervals_15mb_w_names.csv
```
