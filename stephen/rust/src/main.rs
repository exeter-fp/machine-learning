#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate csv;
extern crate clap;
extern crate rayon;

use std::fs::File;
use std::io::Write;
use clap::{App, Arg};
use rayon::prelude::*;

mod row;
mod tree;
mod titanic;
mod cross_validation;

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
        .arg(Arg::with_name("depth")
             .long("depth")
             .help("Cross validate to get the depth")
             .takes_value(false))
        .get_matches();
    
    let train = titanic::load_titanic(matches.value_of("training").unwrap()).unwrap();
    let train = train.iter().collect();
    let depth = if matches.is_present("depth") {
        Some(get_best_depth(&train))
    } else {
        None
    };
        
    let tree = tree::build_tree(train, depth);
    
    if let Some(dotfile) = matches.value_of("dot") {
        let mut f = File::create(dotfile).unwrap();
        f.write_all(tree.to_dot().as_bytes()).unwrap();
    }

    if let Some(testfile) = matches.value_of("test") {
        if let Some(checkfile) = matches.value_of("check") {
            test(&tree, testfile, checkfile);
        } else {
            predict(&tree, testfile);
        }
    }   
}

/// Predict the given file and output results
/// as a csv to stdout.
fn predict(tree: &tree::Node, testfile: &str) {
    let test = titanic::load_titanic(testfile).unwrap();
    println!("PassengerId,Survived");
    for t in test {
        // Kaggle wants 1 for lived, 0 for died.
        println!("{},{}", t.id(), if &tree::classify(&t, &tree) == "Lived" {
            1
        } else {
            0
        });
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

/// Using cross validation see which depth gives us the best results.
fn get_best_depth<T>(train: &Vec<&T>) -> u64 
where T: DataRow + Sync {
    let (best_depth, _) = (1_u64..10_u64)
    // Parallelise the validation. This actually slows it down from 3 -> 6 seconds..
        .into_par_iter() 
        .map(|depth| {
            (depth, cross_validation::validate(&train, 10, |t| tree::build_tree(t, Some(depth))))
        })
        .max_by(|(_, ascore),(_, bscore)| ascore.partial_cmp(bscore).unwrap())
        .unwrap();

    best_depth
}
