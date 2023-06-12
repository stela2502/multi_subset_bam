use clap::Parser;
//use this::cellids::CellIds;

extern crate bam;
use bam::RecordReader;
use bam::RecordWriter;

use std::io;
use std::path::PathBuf;
use std::fs::File;
use bam::record::tags::TagValue;

use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashSet;

use std::time::SystemTime;


#[derive(Parser)]
#[clap(version = "0.1.0", author = "Stefan L. <stefan.lang@med.lu.se>")]
struct Opts {
    /// the bam file you want to subset
    #[clap(short, long)]
    bam: String,
    /// the bam tag you want to look for
    #[clap(default_value="CR", short, long)]
    tag: String,
    /// the values of the bam tag to selet for (a file with one value per line)
    #[clap(short, long)]
    values: String,
    /// the filename for the bam file subset
    #[clap(short, long)]
    ofile: String,
}

fn read_bc( bc_file:String )-> HashSet<String> {
    let file = File::open(bc_file).unwrap();
    let reader = BufReader::new(file);
    let mut tag_set: HashSet<String> = HashSet::new();
    for line in reader.lines() {
        if let Ok(tag_value) = line {
            tag_set.insert(tag_value);
        }
    }
    return tag_set
}

fn main() {
    let now = SystemTime::now();

    let opts: Opts = Opts::parse();

    let mut reader = bam::BamReader::from_path( &opts.bam , 1).unwrap();

    let o1 = PathBuf::from( &opts.ofile );
    let f1 = match File::create(o1){
        Ok(file) => file,
        Err(err) => panic!("The file {} cound not be created: {err}", &opts.ofile )
    };
    let output = io::BufWriter::new( &f1 );
    let mut writer = bam::BamWriter::build()
        .write_header(true)
        .from_stream(output, reader.header().clone()).unwrap();

    let mut record = bam::Record::new();
    if opts.tag.len() != 2 {
        panic!("The tag needs to be exactly two chars long - not {}", &opts.tag);
    }
    let tag: [u8; 2]  = opts.tag.as_bytes().try_into().unwrap();
    
    let bc = read_bc( opts.values );
    let mut reads = 0;
    loop {
        match reader.read_into(&mut record) {
            Ok(true) => {},
            Ok(false) => break,
            Err(e) => panic!("{}", e),
        }

        let tags_data = record.tags().iter();
        for (tag_id, tag_value) in tags_data {
            if tag_id == tag {
                if let TagValue::String(tag_value_str, _) = tag_value {
                    match std::str::from_utf8(tag_value_str){
                        Ok(val) => {
                            if bc.contains(val) {
                                //println!("{}:Z entry: {:?}", &opts.tag, val );
                                writer.write(&record).unwrap();
                                reads += 1;
                            }
                        },
                        Err(e) => { 
                            panic!("I got an error: {e:?}")
                        },
                    };
                }
            }
        }
    }
    match now.elapsed() {
        Ok(elapsed) => {
            let mut milli = elapsed.as_millis();

            let mil = milli % 1000;
            milli= (milli - mil) /1000;

            let sec = milli % 60;
            milli= (milli -sec) /60;

            let min = milli % 60;
            milli= (milli -min) /60;

            println!("I have selected {reads} reads from the bam file in {milli}h {min}min {sec} sec {mil}milli sec\n");
        },
        Err(e) => {println!("Error: {e:?}");}
    }
    
}
