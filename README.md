[![Rust](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml/badge.svg)](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml)

# MultiSubsetBam

A Rust way to subset a BAM based on an internal BAM key.

In comparison to the Illumina `subset_bam` tool, this one is able to create up to 1000 subsets in a single run of the program. Since reading and writing the BAM file is the most time-consuming step in the split procedure, this program (in theory) should speed up the process of splitting multiple subsets by the number of subsets you want to split—for example, a 10x speedup for splitting 10 clusters of cells.

By default, this tool will query the `CB:Z` tag (e.g. `'GAGCAGACAGGCAGTA'`) of the BAM file and only write the reads matching the entries in the 'values' barcodes table into the outfile. I recommend using only the sequence part of your cell names and not including e.g the `-1`.

### New in Version: Multiprocessor Support

The latest version of `MultiSubsetBam` includes support for multiprocessor execution. This feature allows you to utilize multiple CPU cores to process large BAM files more efficiently. The program will automatically distribute the workload across available processors, speeding up the process of generating multiple subsets.

## Install

You need to install Rust on your computer, then clone this repo and compile the tool for your machine:

<script_start>
cargo build -r
<script_end>

The executable will be `target/release/multi_subset_bam`.

## Testing

On a Linux system:

<script_start>
./target/release/multi_subset_bam -b testData/test.bam -v testData/barcodes.txt,testData/barcodes2.txt -o testData/outpath/subset_

samtools view testData/outpath/subset_barcodes.bam | wc -l
samtools view testData/outpath/subset_barcodes2.bam | wc -l
<script_end>

This should show that there are 11 reads for the 4 barcodes.

By default, this tool will query the `CB:Z` tag (e.g. `'GAGCAGACAGGCAGTA'`) of the BAM file and only write the reads matching the entries in the 'values' barcodes table into the outfile. If your cell IDs are like `'GAGCAGACAGGCAGTA-1'`, use the option `-t "CR"`.

## Multiprocessing Example

If you'd like to take advantage of the multiprocessor feature, you can set the number of threads using the `-p` option (e.g., to use 4 threads):

<script_start>
./target/release/multi_subset_bam -b testData/test.bam -v testData/barcodes.txt,testData/barcodes2.txt -o testData/outpath/subset_ -p 4
<script_end>

This will split the BAM file into multiple subsets while leveraging four processor threads to speed up the process.
