extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, Store};

use std::{sync::{Arc, RwLock, OnceLock, Mutex}, fs::OpenOptions};
use std::str::FromStr;
use once_cell::sync::Lazy;

use csv::{self, StringRecord};

use GDS::regexbank::regexlib::{RegexType, RegBank};
use GDS::scraper;
use GDS::threadlib;

struct Arguments {
    names: String,
    reg: String,
    out: String
}

struct Options {
    verbose: bool,
    debug: bool,
    citation: bool
}
fn main() {
    // Argument Parsing 
    let mut args:Arguments = Arguments {names: "".to_string(), reg: "".to_string(), out: "".to_string()};
    let mut options: Options = Options {verbose: false, debug: false, citation: false};
    let mut temp: String = Default::default();
    {
        let mut argparse = ArgumentParser::new();
        argparse.set_description("Cameron's Simple Google Data Scraper");
        argparse.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Include helpful outputs for interpretive processing.");
        argparse.refer(&mut options.debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Include helpful outputs for interpretive processing.");
        argparse.refer(&mut options.citation)
            .add_option(&["-c", "--citation"], StoreTrue, "Include sources next to data points in table.");
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
        args.names = String::from("./sampleinputs/fruits.csv");
        args.reg = String::from("./sampleregex/fruit_regex.csv");
        args.out = String::from("fruit.csv");
    }
    
    if options.verbose {
        println!("Verbose flag {}", options.verbose);
        println!("Input FileName: {}", args.names);
        println!("Output FileName: {}", args.out);
    }

    let args: Arc<Arguments> = Arc::new(Arguments {names: args.names, reg: args.reg, out: args.out});
    let options: Arc<Options> = Arc::new(Options {verbose: options.verbose, debug: options.debug, citation: options.citation});
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
    let mut features: Vec<String> = regfile.into_records().map(|x| x.as_ref().unwrap().get(0).unwrap().to_string()).collect::<Vec<_>>().clone();

    //TODO: Integrate citation option
    if options.citation {
        for entry in 0..features.len() {
            features.insert(entry*2 + 1, "citation".to_string());
        }    
    }
    features.insert(0, "ID".to_string());
    out.write().unwrap().write_record(features).expect("Unable to write to output.");
    out.write().unwrap().flush().unwrap();

    // Iterate over ids
    static CELL: OnceLock<RwLock<Arc<Arguments>>> = OnceLock::new();
    let mut inp: csv::Reader<std::fs::File> = csv::Reader::from_path(args.names.clone()).expect("CSV is empty or null, please check file named.");
    CELL.get_or_init(|| {
        RwLock::new(args.clone())
    });
    static CELL2: OnceLock<RwLock<Arc<Options>>> = OnceLock::new();
    CELL2.get_or_init(|| {
        RwLock::new(options)
    });
    static count: Lazy<Arc<Mutex<usize>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
    inp.records().for_each(|x| pool.execute(|| scrape_vals(x.unwrap(), CELL.get().unwrap(), CELL2.get().unwrap(), count.clone())));
}

fn scrape_vals(record: StringRecord, args: &RwLock<Arc<Arguments>>, options: &RwLock<Arc<Options>>, count: Arc<Mutex<usize>>) {
    let mut regfile: csv::Reader<std::fs::File> = csv::Reader::from_path(args.read().unwrap().reg.clone()).expect("CSV is empty or null, please check file named.");
    let features: Vec<(String, RegBank)> = regfile.into_records().map(|x| (x.as_ref().unwrap().get(0).unwrap().to_string(), RegBank::new(RegexType::from_str(x.unwrap().get(1).unwrap()).unwrap()))).collect::<Vec<_>>().clone();
    let mut output: Vec<String> = vec![record.get(1).unwrap().to_string()];
    for entry in features {
        // Preprocess query into searchable text
        let query: String = scraper::preprocess(record.get(1).unwrap().to_string(), entry.0.clone());
        let readable = query.replace("+", " ");
        println!("Query: {}\n", readable);
        //println!("Query: {}\n", query);
        // Make request to google
        let html: String = scraper::get(query);
        // Scan html for featured snippet
        if let Ok(val) = scraper::scrape_featured(html.clone(), entry.1.clone(), options.read().unwrap().citation.clone()) {
            // If available pull data from featured snippet
            output.push(val.bold_text);
            if options.read().unwrap().citation.clone() {
                output.push(val.citation);
            }
        } else {
            // If unavailable enter crawler routine
            match GDS::scraper::crawler(html, entry.1.clone(), options.read().unwrap().citation.clone()) {
                Ok(val) => {
                    output.push(val.bold_text);
                    if options.read().unwrap().citation.clone() {
                        output.push(val.citation);
                    }
                },
                Err(val) => {
                    output.push("None".to_string());
                }
            };
        }
    }
    // Output to output csv file
    let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .append(true)
    .open(args.write().unwrap().out.clone())
    .unwrap();
    let mut outfile: csv::Writer<std::fs::File> = csv::Writer::from_writer(file);
    outfile.write_record(output.clone()).unwrap();

    let mut inp: csv::Reader<std::fs::File> = csv::Reader::from_path(args.read().unwrap().names.clone()).expect("CSV is empty or null, please check file named.");
    let mut regfile: csv::Reader<std::fs::File> = csv::Reader::from_path(args.read().unwrap().reg.clone()).expect("CSV is empty or null, please check file named.");
    let mut count = count.lock().unwrap();
    if options.read().unwrap().citation {
        *count += (output.len() - 1)/2 - output.iter().skip(1).step_by(2).filter(|x| x.len()<= 0).count();
    } else {
        *count += (output.len() - 1) - output.iter().filter(|x| x.len()<= 0).count();
    }
    println!("Entries covered: {}", count);
    println!("Entries total: {}", inp.records().count() * regfile.records().count());
}
