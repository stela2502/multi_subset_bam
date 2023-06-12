# SubsetBam

A Rust way to subset a bam based on an bam internal key.


# Install

You need to install Rust on your computer, then clone this repo and
finally compile the tools for your computer:
```
cargo build -r
```

The executable will be ``target/release/subset_bam``.

# Testing

```
./target/release/subset_bam -i testData/test.bam -b testData/barcodes.txt -o testData/outpath/subset.bam
samtools view testData/outpath/subset.bam | wc -l
```

This should tell you that there are 11 reads for the 4 barcodes.

By defualt this tool will query the CB:Z tag of the bam file and only write the reads matching the entry in the 'values' barcodes table into the oputfile.


