use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufWriter};
use std::path::{Path, PathBuf};
use bam::{BamReader, BamWriter, Record, RecordWriter };
use bam::record::tags::TagValue;

pub struct Subsetter {
    tags: BTreeMap<String, usize>, // Storage for the keys to match
    ofile_writers: Vec<BamWriter<BufWriter<File>>>, // A vector of outfiles
    ofile_names: Vec<String>, // The outfile names
}

impl Subsetter {
    pub fn new() -> Self {
        Self {
            tags: BTreeMap::new(),
            ofile_writer_count: usize Vec::with_capacity(100),
            ofile_names: Vec::with_capacity(100),
        }
    }

    pub fn read_simple_list(&mut self, bc_file: &str, prefix: &str, header: bam::Header) {
        let file = File::open(bc_file).unwrap();
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(tag_value) = line {
                self.tags.insert(tag_value.clone(), self.tags.len());
                let ofile_name = format!("{}{}.bam", prefix, self.tags.len() - 1);
                self.ofile_names.push(ofile_name);
                let output = File::create(&self.ofile_names.last().unwrap()).unwrap();
                let writer = BamWriter::build().write_header(true).from_stream(BufWriter::new(output), header.clone()).unwrap();
                self.ofile_writers.push(writer);
            }
        }
    }

    pub fn process_records_parallel(&self, records: &[Record], tag: &[u8; 2], chunk_size: usize) -> Vec<Vec<usize>> {
        // Initialize result buffers for each output file
        let mut result: Vec<Vec<usize>> = vec![Vec::with_capacity(1_000_000); self.ofile_writers.len()];

        // Collect tag_values with indices, in parallel
        let tag_values_with_indices: Vec<(usize, String)> = records.iter()
            .enumerate()
            .filter_map(|(index, record)| {
                get_tag_value(&record, tag).map(|value| (index, value))
            })
            .collect();

        // Process records in parallel using par_chunks, and collect the chunk buffers
        let chunk_buffers: Vec<Vec<Vec<usize>>> = tag_values_with_indices.par_chunks(chunk_size).map(|chunk| {
            // Initialize temporary buffers for each output file in this chunk
            let mut chunk_buffers: Vec<Vec<usize>> = vec![Vec::with_capacity(chunk.len()); self.ofile_writers.len()];

            // Iterate over each record in the chunk
            for (index, tag_value ) in chunk.iter() {
                // If the tag_value exists, find the corresponding output file
                if let Some(id) = self.tags.get(tag_value) {
                    chunk_buffers[*id].push(*index); // Store the index of this record within the chunk
                }
            }
            chunk_buffers // Return the chunk's buffers
        }).collect(); // Collect all chunk_buffers from different threads

        // Now merge the chunk_buffers into the main result vector
        for chunk in chunk_buffers {
            for (id, cell_ids) in chunk.into_iter().enumerate() {
                result[id].extend(cell_ids); // Populate the final result with indices from all chunks
            }
        }

        result
    }
}

fn get_tag_value(record: &Record, tag: &[u8; 2]) -> Option<String> {
    record.tags().iter().find_map(|(tag_id, tag_value)| {
        if tag_id == *tag {
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
