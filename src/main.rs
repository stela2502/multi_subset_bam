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

use indicatif::{ProgressBar, MultiProgress, ProgressStyle};

use subset_bam::Subsetter;

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
    values: Option<String>,
    /// more groups of cell ids to export ()
    #[clap(short, long)]
    values: Option<String>,
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

    let mut subsetter = Subsetter::new():

    subsetter.read_simple_list( opts.values, opts.ofile.to_string() );

    let mut record = bam::Record::new();
    if opts.tag.len() != 2 {
        panic!("The tag needs to be exactly two chars long - not {}", &opts.tag);
    }
    let tag: [u8; 2]  = opts.tag.as_bytes().try_into().unwrap();
    
    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new(5000));

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    pb.set_style(spinner_style);
    pb.set_message( "" );

    let bc = read_bc( opts.values );
    let mut reads = 0;
    let mut lines:u64 = 0;
    let split = 1_000_000_u64;

    loop {
        match reader.read_into(&mut record) {
            Ok(true) => {},
            Ok(false) => break,
            Err(e) => panic!("{}", e),
        }
        if lines % split == 0{
            //println!("A log should be printed?");
            pb.set_message( format!("{} mio reads processed", lines / split) );
            pb.inc(1);
        }
        lines +=1;

        reads += subsetter.process_record( record, tag );
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

    subsetter.print();
    
}
