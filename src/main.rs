use clap::Parser;
//use this::cellids::CellIds;

use rust_htslib::{bam, bam::Read, bam::Writer};
use std::fs::{self,File};
use std::path::Path;
use std::io::BufWriter;
use std::time::SystemTime;

use multi_subset_bam::Subsetter;
use std::path::PathBuf;

/// Can split one BAM file into as many as 1000 sub bam files using and BAM tag value.
/// The values to group should be provided in one value per line text files.

#[derive(Parser)]
#[clap(disable_version_flag = true)]  // This prevents the automatic --version
#[clap(version = "0.2.0", author = "Stefan L. <stefan.lang@med.lu.se>")]
struct Opts {
    /// the bam file you want to subset
    #[clap(short, long)]
    bam: String,
    /// the bam tag you want to look for
    #[clap(default_value="CR", short, long)]
    tag: String,
    /// the values of the bam tag file(s); each file stands for one group with one cell_id per line
    #[clap(short, long, value_parser, num_args(1..), value_delimiter = ' ' )] 
    values: Vec<String>,
    /// the filename for the bam file subset
    #[clap(short, long)]
    ofile: String,
}


fn main() {
    let now = SystemTime::now();

    let opts: Opts = Opts::parse();

    let mut reader = bam::Reader::from_path( &opts.bam  ).unwrap();
    let header = bam::Header::from_template(reader.header());

    let mut subsetter = Subsetter::new();
    let size = opts.values.len();

    for fname in &opts.values {
        subsetter.read_simple_list( fname.to_string(), opts.ofile.to_string()  );
    }

    // check if the outfile can be written and create the out folder if it does not exist
    let outpath = match Path::new(&opts.ofile).parent(){
        Some(path) => path,
        None => panic!("Oops - I could not get the parent path of the outfile prefix {}", &opts.ofile),
    };
    if fs::metadata(&outpath).is_err() {
        if let Err(err) = fs::create_dir_all(&outpath) {
            eprintln!("Error creating directory {}: {}", outpath.display(), err);
        } else {
            println!("New output directory created successfully!");
        }
    }
    
    let mut ofiles = Vec::<_>::with_capacity( size );
    for fname in &subsetter.ofile_names {
        ofiles.push(  bam::Writer::from_path( fname, &header, bam::Format::Bam).unwrap() );
    }


    if opts.tag.len() != 2 {
        panic!("The tag needs to be exactly two chars long - not {}", &opts.tag);
    }
    let tag: [u8; 2]  = opts.tag.as_bytes().try_into().unwrap();

    let mut reads = 0;
    let mut lines = 0;

    //let chunk_size = 100_000;
    //let batch_size = chunk_size * num_cpus::get();

    for r in reader.records() {
        if let Ok(record) = r {
            lines+=1;
            match subsetter.process_record( &record, &tag ) {
                Some(ofile_id) => {
                    reads +=1;
                    ofiles[*ofile_id].write(&record).unwrap()
                },
                None => {}
            }
        }else {
            panic!("There was an error r4eading from the input file!");
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

            println!("\nI have selected {reads}/{lines} reads from the bam file in {milli}h {min}min {sec} sec {mil}milli sec\n");
        },
        Err(e) => {println!("Error: {e:?}");}
    }

    
}
