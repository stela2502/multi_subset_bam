use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use bam::{BamWriter, Header, Record, RecordWriter };
use bam::record::tags::TagValue;

pub struct Subsetter {
    tags: BTreeMap<String, usize>,
    ofile_writers: Vec<BamWriter<BufWriter<File>>>,
    ofile_records: Vec<Vec<Record>>, // Buffer for accumulated records
    ofile_names: Vec<String>,
    active: usize,
}

impl Subsetter {
    pub fn new() -> Self {
        Self {
            tags: BTreeMap::new(),
            ofile_writers: Vec::with_capacity(100),
            ofile_records: Vec::with_capacity(100),
            ofile_names: Vec::with_capacity(100),
            active: 0,
        }
    }

    pub fn read_simple_list(&mut self, bc_file: &str, prefix: &str, header: Header) {
        let file = File::open(bc_file).expect("Failed to open barcode file");
        let reader = BufReader::new(file);
        
        for (idx, line) in reader.lines().enumerate() {
            let tag_value = line.expect("Failed to read line");
            if self.tags.len() <= idx {
                self.tags.insert(tag_value.clone(), self.active);
                let ofile_name = format!("{}{}.bam", prefix, tag_value);
                let ofile_path = Path::new(&ofile_name);
                let f = File::create(ofile_path).expect("Failed to create output file");
                let output = BufWriter::new(f);
                let writer = BamWriter::build()
                    .write_header(true)
                    .from_stream(output, header.clone()).expect("Failed to build BAM writer");
                self.ofile_writers.push(writer);
                self.ofile_records.push(Vec::new());
                self.ofile_names.push(ofile_name);
                self.active += 1;
            }
        }
    }

    // Function to retrieve the value of a specific tag
    fn get_tag_value(record: &Record, tag: &[u8; 2]) -> Option<String> {
        // Find the tag in the record
        record.tags().iter().find_map(|(tag_id, tag_value)| {
            if tag_id == *tag {
                // Extract and return the tag value if it's a string
                if let TagValue::String(value, _) = tag_value {
                    std::str::from_utf8(value).ok().map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn process_records_parallel(&mut self, records: Vec<Record>, tag: &[u8; 2]) {
        let num_files = self.ofile_writers.len();
        
        let record_buffer_buffer: Vec<Vec<Vec<Record>>> = records.par_chunks(1000).for_each(|chunk| {
            
            let mut local_buffers: Vec<Vec<Record>> = vec![Vec::new(); num_files];

            for record in chunk {
                record = record.clone();
                if let Some(val) = Self::get_tag_value(&record, tag) {
                    if let Some(id) = self.tags.get(&val) {
                        local_buffers[*id].push(record.clone()); // Or use Arc if preferred
                    }
                }
            }

            local_buffers
        });

        // Accumulate results
        for record_buffers in &record_buffer_buffer{
            for (i, buffer) in record_buffers.into_iter().enumerate() {
                self.ofile_records[i].extend(buffer);
            }
        }
    }

    pub fn write_records(&mut self) {
        for (i, writer) in self.ofile_writers.iter_mut().enumerate() {
            for record in &self.ofile_records[i] {
                writer.write(record).expect("Failed to write record");
            }
        }
    }

    pub fn print(&self) {
        for (i, count) in self.ofile_records.iter().map(|v| v.len()).enumerate() {
            println!("{} reads are stored in {}", count, self.ofile_names[i]);
        }
        println!();
    }
}
