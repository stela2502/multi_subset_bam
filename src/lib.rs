use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rust_htslib::bam::record::Record;



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
    if let Some(value) = record.aux(tag).ok() {
        match value {
            rust_htslib::bam::record::Aux::String(s) => Some(s.to_string()),
            _ => None,
        }
    } else {
        None
    }
}


