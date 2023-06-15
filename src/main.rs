use clap::Parser;
//use this::cellids::CellIds;

extern crate bam;
use bam::RecordReader;

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
    /// the values of the bam tags file(s) (comma separated filenames) to selet for (a file with one value per line)
    #[clap(short, long)]
    values: String,
    /// the filename for the bam file subset
    #[clap(short, long)]
    ofile: String,
}


fn main() {
    let now = SystemTime::now();

    let opts: Opts = Opts::parse();

    let mut reader = bam::BamReader::from_path( &opts.bam , 1).unwrap();

    let mut subsetter = Subsetter::new();

    for fname in opts.values.split(','){
        subsetter.read_simple_list( fname.to_string(), opts.ofile.to_string(), reader.header().clone() );
    }
    

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

        reads += subsetter.process_record( record.clone(), tag );
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

            println!("\nI have selected {reads} reads from the bam file in {milli}h {min}min {sec} sec {mil}milli sec\n");
        },
        Err(e) => {println!("Error: {e:?}");}
    }

    subsetter.print();

    
}
