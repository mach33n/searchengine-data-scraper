#![feature(once_cell)]
extern crate argparse;

use std::sync::{Arc, RwLock, OnceLock};

use csv::{self, StringRecord};
use argparse::{ArgumentParser, StoreTrue, Store};

mod threadlib;

struct Arguments {
    names: String,
    reg: String,
    out: String
}

struct Options {
    verbose: bool,
    debug: bool
}

fn main() {
    // Argument Parsing 
    let mut args:Arguments = Arguments {names: "".to_string(), reg: "".to_string(), out: "".to_string()};
    let mut options: Options = Options {verbose: false, debug: false};
    let mut temp: String = Default::default();
    {
        let mut argparse = ArgumentParser::new();
        argparse.set_description("Cameron's Simple Google Data Scraper");
        argparse.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Include helpful outputs for interpretive processing.");
        argparse.refer(&mut options.debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Include helpful outputs for interpretive processing.");
        argparse.refer(&mut temp)
            .add_argument("blank", Store, "");
        argparse.refer(&mut args.names)
            .add_argument("names", Store, "Names of desired ids for dataset");
        argparse.refer(&mut args.reg)
            .add_argument("regex", Store, "This should be a csv file containing the names of features along with the regex associated for scraping them.");
        argparse.refer(&mut args.out)
            .add_argument("output", Store, "Desired output file name");

        // Handle errors for parsing arguments
        match argparse.parse_args() {
            Ok(()) => {},
            Err(x) => {
                std::process::exit(x);
            }
        }
    }

    // TODO: Remove before pushing to github
    if options.debug && args.names.is_empty() {
        args.names = String::from("./sampleinputs/plant_names.csv");
        args.reg = String::from("./sampleregex/plant_features_regex.csv");
        args.out = String::from("crop_rec.csv");
    }
    
    if options.verbose {
        println!("Verbose flag {}", options.verbose);
        println!("Input FileName: {}", args.names);
        println!("Output FileName: {}", args.out);
    }

    let args: Arc<Arguments> = Arc::new(Arguments {names: args.names, reg: args.reg, out: args.out});

    // Validate File structures
    {
        let dup_inp: csv::StringRecordsIntoIter<std::fs::File> = csv::Reader::from_path(args.names.clone()).expect("CSV is empty or null, please check file named.").into_records();
        let dup_reg: csv::StringRecordsIntoIter<std::fs::File>  = csv::Reader::from_path(args.reg.clone()).expect("CSV is empty or null, please check file named.").into_records();
        let len_inp: usize = dup_inp.count();
        let len_regfile: usize = dup_reg.count();
        if len_inp <= 0 || len_regfile <= 0 {
            println!("Check file lengths to ensure they are nonzero.");
            panic!("Unable to operate on empty files.");
        } 
    }

    // Add features to output file    
    let mut pool: threadlib::threadlib::ThreadPool = threadlib::threadlib::ThreadPool::new(1);

    let regfile: csv::Reader<std::fs::File> = csv::Reader::from_path(args.reg.clone()).expect("CSV is empty or null, please check file named.");
    let out: csv::Writer<std::fs::File> = csv::Writer::from_path(args.out.clone()).expect("Unable to instantiate csv writer.");
    let out: Arc<RwLock<csv::Writer<std::fs::File>>> = Arc::new(RwLock::new(out));
    let features: Vec<String> = regfile.into_records().map(|x| x.unwrap().get(0).unwrap().to_string()).collect::<Vec<_>>().clone();

    //TODO: Integrate citation option
    //features.push("references".to_string());
    out.write().unwrap().write_record(features).expect("Unable to write to output.");
    out.write().unwrap().flush().unwrap();

    // Iterate over ids
    static CELL: OnceLock<RwLock<Arc<Arguments>>> = OnceLock::new();
    let mut inp: csv::Reader<std::fs::File> = csv::Reader::from_path(args.names.clone()).expect("CSV is empty or null, please check file named.");
    CELL.get_or_init(|| {
        RwLock::new(args)
    });
    inp.records().for_each(|x| pool.execute(|| scrape_vals(x.unwrap(), CELL.get().unwrap())));
}

fn scrape_vals(record: StringRecord, args: &RwLock<Arc<Arguments>>) {
    let mut regfile: csv::Reader<std::fs::File> = csv::Reader::from_path(args.read().unwrap().reg.clone()).expect("CSV is empty or null, please check file named.");
    let mut output: Vec<String> = vec![];
    for entry in regfile.records() {
        // Preprocess query into searchable text
        let query: String= GDS::scraper::preprocess(record.get(1).unwrap().to_string(), entry.as_ref().unwrap().get(0).unwrap().to_string());
        println!("Query: {}\n", query);
        // Make request to google
        let html: String = GDS::scraper::get(query);
        // Scan html for featured snippet
        if let Ok(val) = GDS::scraper::scrape_featured(html.clone(), entry.as_ref().unwrap().get(1).unwrap().to_string()) {
            // If available pull data from featured snippet
            output.push(val);
        } else {
            // If unavailable enter crawler routine
            match GDS::scraper::crawler(html, entry.unwrap().get(1).unwrap().to_string()) {
                Ok(val) => {

                },
                Err(val) => {

                }
            };
        }
    }
    // Output to output csv file
    let mut outfile: csv::Writer<std::fs::File> = csv::Writer::from_path(args.write().unwrap().out.clone()).expect("Unable to write to out file.");
    outfile.write_record(output).unwrap();
}
