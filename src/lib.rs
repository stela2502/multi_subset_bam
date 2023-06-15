/// subsetter is a class that is used to subset sam/bam files.
/// It requires a bam file at startup and a gile with markers to look for an the position in the bam file you want to look for.
/// Its main usage is to subset a 10X bam file for a subset of cells.

use std::collections::BTreeMap;
extern crate bam;
use bam::RecordWriter;

use std::io;
use std::path::PathBuf;
use std::fs::File;
use bam::record::tags::TagValue;

use std::io::BufReader;
use std::io::BufRead;

use std::io::BufWriter;
use std::path::Path;

pub struct Subsetter{
	tags: BTreeMap< String, usize>, /// the storage for the keys to match 
	ofiles: Vec<bam::BamWriter<BufWriter<File>>>, /// a vector of outfiles
	ofile_counts: Vec<usize>, /// record the bam_entry counts written to the outfiles
	ofile_names: Vec<String>, /// the outfile names
	active: usize, //a marker to the currently active ofile during creation
}

impl Subsetter{
	/// create the Subsetter with the opened bam::BamReader
	pub fn new() -> Self{
		let tags = BTreeMap::< String, usize>::new();
		let ofiles = Vec::<bam::BamWriter<BufWriter<File>>>::with_capacity(100);
		let ofile_counts = Vec::<usize>::with_capacity(100);
		let ofile_names = Vec::<String>::with_capacity(100);
		let active = 0;
		Self{
			tags,
			ofiles,
			ofile_counts,
			ofile_names,
			active,
		}
	}
	/// read a simple list of cell ids
	pub fn read_simple_list (&mut self, bc_file:String, prefix:String, header:bam::Header ) {
		let file = File::open(bc_file.to_string()).unwrap();
	    let reader = BufReader::new(file);
	    for line in reader.lines() {
	        if let Ok(tag_value) = line {
	            self.tags.insert(tag_value, self.active);
	        }
	    }
	    let ofile = Path::new(&bc_file).file_stem().unwrap().to_str().unwrap();
	    self.ofile_names.push( format!("{}{}.bam", prefix, ofile) );
	    let o1 = PathBuf::from( self.ofile_names[self.active].to_string() );
	    let f1 = match File::create(o1){
	        Ok(file) => file,
	        Err(err) => panic!("The file {} cound not be created: {err}", ofile )
	    };
	    let output = io::BufWriter::new( f1 );
	    let writer = bam::BamWriter::build()
	        .write_header(true)
	        .from_stream(output, header ).unwrap();
	    self.ofiles.push( writer );
	    self.ofile_counts.push( 0 );
	    self.active +=1;
	}
	/// process a single bam::Record
	pub fn process_record( &mut self, record:bam::Record, tag:[u8; 2] ) -> usize{
		let mut ret = 0;
        for (tag_id, tag_value) in record.tags().iter() {
            if tag_id == tag {
                if let TagValue::String(tag_value_str, _) = tag_value {
                    match std::str::from_utf8(tag_value_str){
                        Ok(val) => {
                            if let Some(id) = self.tags.get( val) {
                                //println!("{}:Z entry: {:?}", &opts.tag, val );
                                self.ofiles[*id].write(&record).unwrap();
                                self.ofile_counts[*id] += 1;
                                ret +=1;
                            }
                        },
                        Err(e) => { 
                            panic!("I got an error: {e:?}")
                        },
                    };
                }
            }
        }
        ret
	}
	/// print the result of the matching process
	pub fn print(&self ) {
		for i in 0..self.ofiles.len() {
			println!("{} reads are stored in {}", self.ofile_counts[i], self.ofile_names[i] );
		}
		println!("\n");
	}


}
