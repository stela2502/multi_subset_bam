[![Rust](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml/badge.svg)](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml)

# MultiSubsetBam

A Rust way to subset a BAM based on an internal BAM key.

In comparison to the Illumina `subset_bam` tool, this one is able to create up to 1000 subsets in a single run of the program. Since reading and writing the BAM file is the most time-consuming step in the split procedure, this program (in theory) should speed up the process of splitting multiple subsets by the number of subsets you want to split—for example, a 10x speedup for splitting 10 clusters of cells.

By default, this tool will query the `CB:Z` tag (e.g. `'GAGCAGACAGGCAGTA'`) of the BAM file and only write the reads matching the entries in the 'values' barcodes table into the outfile. I recommend using only the sequence part of your cell names and not including e.g the `-1`.

### New in Version: Multiprocessor Support

The latest version of `MultiSubsetBam` includes support for multiprocessor execution. This feature allows you to utilize multiple CPU cores to process large BAM files more efficiently. The program will automatically distribute the workload across available processors, speeding up the process of generating multiple subsets.

## Usage

```
multi_subset_bam -h

multi_subset_bam 0.1.0
Stefan L. <stefan.lang@med.lu.se>

USAGE:
    multi_subset_bam [OPTIONS] --bam <BAM> --values <VALUES> --ofile <OFILE>

OPTIONS:
    -b, --bam <BAM>                  the bam file you want to subset
    -h, --help                       Print help information
    -o, --ofile <OFILE>              the filename for the bam file subset
    -p, --processors <PROCESSORS>    the numer of processors to use (default all)
    -t, --tag <TAG>                  the bam tag you want to look for [default: CR]
    -v, --values <VALUES>            the values of the bam tags file(s) (comma separated filenames)
                                     to selet for (a file with one value per line)
    -V, --version                    Print version information
```

This github repo contains an example of how the input files should look like.

This is how you can run the test from the repo's main path:
```
multi_subset_bam -b testData/test.bam -v testData/barcodes.txt,testData/barcodes2.txt -o testData/outpath/subset_
```

## Install 

First of all you need to install Rust on your computer (https://www.rust-lang.org/tools/install).

Then it is easiest to install the tool with:
```
install --git https://github.com/stela2502/multi_subset_bam
```

## Install from source 

You need to install Rust on your computer, then clone this repo and compile the tool for your machine:

```
cargo build -r
```

The executable will be `target/release/multi_subset_bam`.

## Testing

On a Linux system:

```
./target/release/multi_subset_bam -b testData/test.bam -v testData/barcodes.txt,testData/barcodes2.txt -o testData/outpath/subset_

samtools view testData/outpath/subset_barcodes.bam | wc -l
samtools view testData/outpath/subset_barcodes2.bam | wc -l
```

This should show that there are 11 reads for the 4 barcodes.

By default, this tool will query the `CR:Z` tag (e.g. `'GAGCAGACAGGCAGTA'`) of the BAM file and only write the reads matching the entries in the 'values' barcodes table into the outfile. If your split 10x bam files the CB tag contains corrected cell ids. You should query them instead (-t CB), but you also need provide cell ids with a "-1" at the end.

## Multiprocessing Example

If you'd like to take advantage of the multiprocessor feature, you can set the number of threads using the `-p` option (e.g., to use 4 threads):

```
./target/release/multi_subset_bam -b testData/test.bam -v testData/barcodes.txt testData/barcodes2.txt -o testData/outpath/subset_ -p 4
```

This will split the BAM file into multiple subsets while leveraging four processor threads to speed up the process.
