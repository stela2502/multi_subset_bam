use clap::Parser;
//use this::cellids::CellIds;

extern crate bam;
use bam::{Record, RecordReader, RecordWriter, BamWriter};
use std::fs::{self,File};
use std::path::Path;
use std::io::BufWriter;
use std::time::SystemTime;

use indicatif::{ProgressBar, MultiProgress, ProgressStyle};

use multi_subset_bam::Subsetter;
use std::path::PathBuf;
use num_cpus;

use rayon::ThreadPoolBuilder;


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
    /// the numer of processors to use (default all)
    #[clap(short, long)]
    processors: Option<usize>,
}


fn main() {
    let now = SystemTime::now();

    let opts: Opts = Opts::parse();

    let mut reader = bam::BamReader::from_path( &opts.bam , 1).unwrap();

    let cpus = match &opts.processors{
        Some(p) => num_cpus::get().min(*p),
        None => num_cpus::get()
    };

    // Set the number of threads using the calculated number
    ThreadPoolBuilder::new().num_threads(cpus).build_global().unwrap();

    let mut subsetter = Subsetter::new();

    for fname in opts.values {
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

    let mut ofiles: Vec<BamWriter<BufWriter<_>>> = subsetter.ofile_names.clone().into_iter().map( |ofile_name| {
        let o1 = PathBuf::from( ofile_name.to_string() );
        let f1 = match File::create(o1){
            Ok(file) => file,
            Err(err) => panic!("The file {} cound not be created: {err}", ofile_name )
        };
        let output = BufWriter::new( f1 );
        let writer = bam::BamWriter::build()
            .write_header(true)
            .from_stream(output, reader.header().clone() ).unwrap();
        writer}
    ).collect();

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
    let mut lines = 0;

    let chunk_size = 100_000;
    let batch_size = chunk_size * num_cpus::get();

    let mut records_tmp= Vec::<Record>::with_capacity( batch_size );

    loop {
        match reader.read_into(&mut record) {
            Ok(true) => {},
            Ok(false) => break,
            Err(e) => panic!("{}", e),
        }
        match subsetter.process_record( &record, &tag ) {
            Some(ofile_id) => {
                ofiles[*ofile_id].write(&record).unwrap()
            },
            None => {}
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

            println!("\nI have selected {reads} reads from the bam file in {milli}h {min}min {sec} sec {mil}milli sec\n");
        },
        Err(e) => {println!("Error: {e:?}");}
    }

    
}
