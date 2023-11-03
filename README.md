[![Rust](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml/badge.svg)](https://github.com/stela2502/multi_subset_bam/actions/workflows/rust.yml)

# MultiSubsetBam

A Rust way to subset a bam based on an bam internal key.

In comparison to the Illumina subset_bam tool this one is able to create up to 1000 subsets in one run of the program.
As reading and writing the BAM file is the most time consuming step in the split procedure this program should (in theory) speed the splitting of multiple subsets up by the amount of supbsets you want to split - e.g 10x for 10 clusters of cells to split.


# Install

You need to install Rust on your computer, then clone this repo and
finally compile the tool for your computer:
```
cargo build -r
```

The executable will be ``target/release/multi_subset_bam``.

# Testing

On a Linux system:

```
./target/release/multi_subset_bam -b testData/test.bam -v testData/barcodes.txt,testData/barcodes2.txt -o testData/outpath/subset_

samtools view testData/outpath/subset_barcodes.bam | wc -l
samtools view testData/outpath/subset_barcodes2.bam | wc -l
```

This should tell you that there are 11 reads for the 4 barcodes.

By default this tool will query the CB:Z tag (e.g. 'GAGCAGACAGGCAGTA') of the bam file and only write the reads matching to the entries in the 'values' barcodes table into the outfile. If you have cell ids like 'GAGCAGACAGGCAGTA-1' you want to use the option ``-t "CR"``.


