use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use bam::Record;
use bam::record::tags::TagValue;


pub struct Subsetter {
    tags: BTreeMap<String, usize>, // Storage for the keys to match
    ofile_writers: usize,// Vec<BamWriter<BufWriter<File>>>, // A vector of outfiles
    pub ofile_names: Vec<String>, // The outfile names
}

impl Subsetter {
    pub fn new() -> Self {
        Self {
            tags: BTreeMap::new(),
            ofile_writers: 0 ,//Vec::with_capacity(100),
            ofile_names: Vec::with_capacity(100),
        }
    }

    /// read a simple list of cell ids
    pub fn read_simple_list (&mut self, bc_file:String, prefix:String ) {
        let file = File::open(bc_file.to_string()).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(tag_value) = line {
                /*let clean_tag = tag_value.strip_suffix("-1").unwrap_or(&tag_value);
                self.tags.insert(clean_tag.to_string(), self.ofile_writers); */
                self.tags.insert(tag_value.to_string(), self.ofile_writers);
            }
        }
        self.ofile_writers +=1;
        let ofile = Path::new(&bc_file).file_stem().unwrap().to_str().unwrap();
        self.ofile_names.push( format!("{}{}.bam", prefix, ofile) );
    }

    pub fn process_records(&self, records: &[Record], tag: &[u8; 2]) -> Vec<Vec<usize>> {
        // Initialize result buffers for each output file
        let mut result: Vec<Vec<usize>> = vec![Vec::with_capacity(1_000_000); self.ofile_writers];

        // Process each record sequentially
        for (index, record) in records.iter().enumerate() {
            // Get the tag value for the current record
            
            if let Some(tag_value) = get_tag_value(record, tag) {
                #[cfg(debug_assertions)]
                println!("I found tag {}", tag_value );
                // If the tag_value exists, find the corresponding output file
                if let Some(id) = self.tags.get(&tag_value) {
                    result[*id].push(index); // Store the index of this record
                }
            }
        }

        result
    }

    /// the main data worker:
    /// It collects the positions and tags of the records and matches them
    /// using the par_chunks() functionality.
    pub fn process_records_parallel(&self, records: &[Record], tag: &[u8; 2], chunk_size: usize ) -> Vec<Vec<usize>> {

        // Initialize result buffers for each output file
        let mut result: Vec<Vec<usize>> = vec![Vec::with_capacity(1_000_000); self.ofile_writers];

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
            let mut chunk_buffers: Vec<Vec<usize>> = vec![Vec::with_capacity(chunk.len()); self.ofile_writers];
            #[cfg(debug_assertions)]
            println!("Processing a chunk of size {}",chunk.len());
            // Iterate over each record in the chunk
            for (index, tag_value ) in chunk.iter() {
                #[cfg(debug_assertions)]
                println!("I found tag {}", tag_value );
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
    pub fn process_record(&self, record: &Record, tag:&[u8;2] ) -> Option<&usize> {
        if let Some(tag_value) = get_tag_value(record, tag) {
            #[cfg(debug_assertions)]
            println!("I found tag {}", tag_value );
            // If the tag_value exists, find the corresponding output file
            self.tags.get(&tag_value)
        }else {
            None
        }
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
