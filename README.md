# SubsetBam

A Rust way to subset a bam based on an bam internal key.


# Install

You need to install Rust on your computer, then clone this repo and
finally compile the tool for your computer:
```
cargo build -r
```

The executable will be ``target/release/subset_bam``.

# Testing

On a Linux system:

```
./target/release/subset_bam -b testData/test.bam -b testData/barcodes.txt -o testData/outpath/subset.bam
samtools view testData/outpath/subset.bam | wc -l
```

This should tell you that there are 11 reads for the 4 barcodes.

By default this tool will query the CB:Z tag (e.g. 'GAGCAGACAGGCAGTA') of the bam file and only write the reads matching to the entries in the 'values' barcodes table into the outfile. If you have cell ids like 'GAGCAGACAGGCAGTA-1' you want to use the option ``-t "CR"``.


