
// Import necessary modules
use std::process::Command;
use std::fs::File;
use std::fs;
use std::path::Path;


#[test]
fn test_multi_subset_bam() {
    let is_release_mode = !cfg!(debug_assertions);

    let command = if is_release_mode {
        "./target/release/multi_subset_bam"
    } else {
        "./target/debug/multi_subset_bam"
    };

    let path = Path::new("testData/output");
    if path.exists() {
        fs::remove_dir_all(path).expect("Failed to remove directory");
    }


    let args = &[
        "-b", "testData/test.bam",
        "-v", "testData/barcodes.txt testData/barcodes2.txt",
        "-o", "testData/output/two_clusters_",
    ];

    // Execute the command with the provided arguments
    let output = Command::new( command ).args( args )
        .output()
        .map_err(|e| {
            eprintln!("Failed to execute command: {}", e);
            e
        }).unwrap();

    let cmd = format!("{} {}", command, args.join(" "));
    if !output.status.success() {
        eprintln!("Command failed: {}", cmd);
        // Handle failure accordingly
    }else {
        println!("{}", cmd );
    }

    // Check if the command was successful (exit code 0)
    assert!(output.status.success());

    let expect = vec!["two_clusters_barcodes.bam","two_clusters_barcodes2.bam"];

    for file in expect{
        let ofile= "testData/output/".to_string() +file;
        assert!(Path::new( &ofile ).exists(), "Expected outfile {} does not exist!", ofile);
    }

}