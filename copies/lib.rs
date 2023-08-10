#![feature(core_intrinsics)]
pub mod scraper {
    use std::any::Any;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;
    use csv::StringRecord;
    use html_parser::Dom;
    use security_framework::secure_transport::ClientBuilder;
    use std::sync::Arc;
    use std::error::Error;
    use std::net::TcpStream;
    use std::io::Read;
    use std::io::Write;
    use html_parser::Node;
    use regex::Regex;

    pub struct Scraper;

    impl Scraper {
        fn get(query: String) -> String {
            let mut page = vec![];
            loop{
                let query = format!("search?q={}", query);
                let stream = TcpStream::connect("google.com:443").unwrap();
                let mut stream = ClientBuilder::new().handshake("google.com", stream).unwrap();
                
                stream.write_fmt(format_args!("GET /{} HTTP/1.0\r\n\r\n", query)).unwrap();
                stream.read_to_end(&mut page).unwrap();
                if String::from_utf8_lossy(&page).to_string().contains("302") {
                    sleep(Duration::new(360,0)); 
                } else {
                    sleep(Duration::new(6,0)); 
                    break;
                }
            }
            String::from_utf8_lossy(&page).to_string()
        }

        fn preprocess_query(query: String) -> String {
            //println!("{}", query.replace(" ", "+").to_lowercase());
            query.replace(" ", "+").to_lowercase()
        }

        fn parse_html(page: String) -> Result<std::string::String, html_parser::Error> {
            // Preproc step
            if page.split_once("Accept-Encoding\r\n\r\n").is_none() {
                return Ok("".to_string())
            }
            let html = page.split_once("Accept-Encoding\r\n\r\n").unwrap().1;

            let mut idx: usize = 0;
            let binding: Dom = Dom::parse(html).unwrap();
            let mut temp: &Node = binding.children.get(idx).expect("Empty html document, check value submitted.");
            let mut stack: Vec<(&Node, usize)> = vec![];
            let mut record: bool = false;
            let mut ret: Vec<String> = vec![];
            // Essentially DFS algorithm
            loop {
                // 1. Check base case
                if temp.element().is_some() && temp.element().unwrap().classes.contains(&"BNeawe".to_string()) && !record {
                    // Assume this is the first featured snippet and then
                    // extract Text from this element.
                    // Clear stack and continue running loop with record on
                    stack.clear();
                    stack.push((&temp, idx+1));
                    record = true;
                }
                // Check if component is an element with children
                if temp.element().is_some() && temp.element().unwrap().children.len() > idx {
                    // if so then iterate downwards towards children
                    stack.push((&temp, idx+1));
                    temp = temp.element().unwrap().children.get(idx).expect("Reached a dead end of tree.");
                    idx = 0;
                } else {
                    if record && stack.len() == 0 {
                        break;
                    } else if record && temp.text().is_some() {
                        ret.push(temp.text().unwrap().to_string());
                    }
                    (temp, idx) = stack.pop().unwrap();
                }
            }
            println!("Return value: {:?}", ret.concat());
            Ok(ret.concat())
        }

        fn extract_npk(input: String) -> String {
            // Currently only finds strings with some format num(-::)num(-::)num. Many other
            // formats have yet to be considered.
            let re = Regex::new(r"[0-9]*(?:\s*-\s*|:)[0-9]*(?:\s*-\s*|:)[0-9]*").unwrap();
            if let Some(m) = re.find(input.as_str()) {
                println!("{}", m.as_str());
                m.as_str().to_string()
            } else {
                "".to_string()
            }
        }

        fn extract_temp(input: String) -> String {
            // Currently only finds strings with some format num° or num degrees or num°F. Many other
            // formats have yet to be considered.
            let re = Regex::new(r"[0-9]+(?:[\s]*\S[F]|°|[\s]*degree[s]*)").unwrap();
            if let Some(m) = re.find(input.as_str()) {
                println!("{}", m.as_str());
                m.as_str().to_string()
            } else {
                "".to_string()
            }
        }

        fn extract_hum(input: String) -> String {
            // Currently only finds strings with some format num(-::)num(-::)num. Many other
            // formats have yet to be considered.
            let re = Regex::new(r"\d*[0-9]+\s*(?:%|percent)").unwrap();
            if let Some(m) = re.find(input.as_str()) {
                println!("{}", m.as_str());
                m.as_str().to_string()
            } else {
                "".to_string()
            }
        }

        //TODO
        fn extract_light(input: String) -> String {
            // Currently only finds strings with some format num(-::)num(-::)num. Many other
            // formats have yet to be considered.
            let re = Regex::new(r"[0-9]*(?:[+]|\s*)\s*(?:hrs|hours|hr)").unwrap();
            if let Some(m) = re.find(input.as_str()) {
                println!("{}", m.as_str());
                m.as_str().to_string()
            } else {
                "".to_string()
            }
        }

        //TODO
        fn extract_moist(input: String) -> String {
            // Currently only finds strings with some format num(-::)num(-::)num. Many other
            // formats have yet to be considered.
            let re = Regex::new(r"\d*[0-9]+\s*(?:%|percent)").unwrap();
            if let Some(m) = re.find(input.as_str()) {
                println!("{}", m.as_str());
                m.as_str().to_string()
            } else {
                "".to_string()
            }
        }

        pub fn get_npk(record: Arc<StringRecord>) -> (String, String) {
            let example = format!("What is \"NPK\" for {}?", record.get(1).expect("Null Record").to_lowercase()).to_string();
            println!("Prompt: {}", example);
            let query = Scraper::preprocess_query(example);
            let mut page: String;
            loop {
                page = Scraper::get(query.clone());
                if page.len() <= 0 {
                    sleep(Duration::new(500,0));
                } else {
                    break;
                }
            } 
            let ret_string: String = Scraper::parse_html(page).expect("Problem with parsing html");
            let npk = Scraper::extract_npk(ret_string.clone());
            (npk, ret_string)
        }
        
        //TODO: Humidity seems to have fewer feature boxes, might want to extend search space
        // to enable acquisition of data from description boxes.
        pub fn get_temp_hum(record: Arc<StringRecord>) -> (String, String, String, String) {
            // temperature
            let example = format!("What is optimal \"temperature\" for growing {}?", record.get(1).expect("Null Record").to_lowercase()).to_string();
            println!("Prompt: {}", example);
            let query = Scraper::preprocess_query(example);
            let paget = Scraper::get(query);
            let temprefr: String = Scraper::parse_html(paget).expect("Problem with parsing html");
            let temp = Scraper::extract_temp(temprefr.clone());

            // humidity
            let example = format!("What is \"humidity\" percent for {}?", record.get(1).expect("Null Record").to_lowercase()).to_string();
            println!("Prompt: {}", example);
            let query = Scraper::preprocess_query(example);
            let pageh = Scraper::get(query);
            let humrefr: String = Scraper::parse_html(pageh).expect("Problem with parsing html");
            let hum = Scraper::extract_hum(humrefr.clone());
            (temp, temprefr, hum, humrefr)
        }

        //TODO
        pub fn get_light_moist(record: Arc<StringRecord>) -> (String, String, String, String) {
            // lighting
            let example = format!("How many hours of sun do {} need to grow?", record.get(1).expect("Null Record").to_lowercase()).to_string();
            println!("Prompt: {}", example);
            let query = Scraper::preprocess_query(example);
            let pagel = Scraper::get(query);
            let lightref: String = Scraper::parse_html(pagel).expect("Problem with parsing html");
            let light = Scraper::extract_light(lightref.clone());

            // moisture
            let example = format!("What is optimal moisture for {} as a percent?", record.get(1).expect("Null Record").to_lowercase()).to_string();
            println!("Prompt: {}", example);
            let query = Scraper::preprocess_query(example);
            let pagem = Scraper::get(query);
            let moistref: String = Scraper::parse_html(pagem).expect("Problem with parsing html");
            let moist = Scraper::extract_moist(moistref.clone());
            (light, lightref, moist, moistref)
        }
    }
}

