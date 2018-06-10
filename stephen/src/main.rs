#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate csv;
extern crate clap;

use std::fs::File;
use std::io::Write;
use clap::{App, Arg};

mod row;
mod tree;
mod titanic;
 
use row::DataRow;

fn main() {
    // Setup args
    let matches = App::new("decisions")
        .version("1.0")
        .about("Nice decision tree")
        .author("Wakka")
        .arg(Arg::with_name("training")
             .short("t")
             .long("train")
             .help("Training file")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("test")
             .short("s")
             .long("test")
             .help("Testing file")
             .takes_value(true))
        .arg(Arg::with_name("check")
             .short("c")
             .long("check")
             .help("File to check the test")
             .takes_value(true))
        .arg(Arg::with_name("dot")
             .short("d")
             .long("dot")
             .help("Output dot file")
             .takes_value(true))
        .get_matches();
    
    let train = titanic::load_titanic(matches.value_of("training").unwrap()).unwrap();
    let train = train.iter().collect();
    let tree = tree::build_tree(train);
    
    if let Some(dotfile) = matches.value_of("dot") {
        let mut f = File::create(dotfile).unwrap();
        f.write_all(tree.to_dot().as_bytes()).unwrap();
    }
    
    if let Some(testfile) = matches.value_of("test") {
        if let Some(checkfile) = matches.value_of("check") {
            test(&tree, testfile, checkfile);
       }
    }
}

/// Test the given decision tree against the test file,
/// using checkfile to check the results.
fn test(tree: &tree::Node, testfile: &str, checkfile: &str) {
    let test = titanic::load_titanic(testfile).unwrap();
    let check = titanic::load_check(checkfile).unwrap();

    // Partition into correct guesses and incorrect guesses
    let correct = test.iter().filter(|t| {
        let actual = check.get(&t.id()).unwrap();
        actual == &tree::classify(&**t, &tree) 
    }).count();
    
    print!("{}/{} = {}%", 
           correct, 
           test.len(),
           correct as f64 / test.len() as f64 * 100.0);
}

