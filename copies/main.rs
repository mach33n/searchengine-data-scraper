use csv::{self};
use std::{sync::Arc};
use plant_proj::scraper;
use std::sync::{mpsc, Mutex};

mod threadlib;

fn main() {
    let inp: Result<csv::Reader<std::fs::File>, csv::Error> = csv::Reader::from_path("Plant_names_only.csv");
    let mut out: csv::Writer<std::fs::File> = csv::Writer::from_path("Crop_Rec.csv").unwrap();
    out.write_record(["plant_name","npk", "npktext", "temp", "temprefr", "hum", "humrefr", "light", "lightref", "moist", "moistref"]);
    let mut pool = threadlib::threadlib::ThreadPool::new(50);
    for mut res in inp.expect("Out is empty. Check CSV.").records() {
        let thing = res.as_ref().unwrap().clone();
        let outin = csv::Reader::from_path("Crop_Rec.csv").unwrap();
        if outin.into_records().filter(|x| x.as_ref().unwrap().get(1).unwrap().to_lowercase() == thing.get(1).unwrap().to_lowercase()).count() > 0 {
            continue;
        }
        let rec = res;
        let temp = Arc::new(rec.expect("Empty Record"));
        let (sender, reciever) = mpsc::channel();
        pool.execute(move || {
            let (npk, refr) = scraper::Scraper::get_npk(temp.clone());
            println!("Plant: {}, NPK Val: {}, Ref: {}", temp.get(1).unwrap(), npk, refr);
            let (tem, trefr, hum, humrefr) = scraper::Scraper::get_temp_hum(temp.clone());
            println!("Plant: {}, Temp Val: {}, Ref: {}", temp.get(1).unwrap(), tem, refr);
            println!("Plant: {}, Hum Val: {}, Ref: {}", temp.get(1).unwrap(), hum, humrefr);
            let (light, lightref, moist, moistref) = scraper::Scraper::get_light_moist(temp.clone());
            println!("Plant: {}, Light Val: {}, Ref: {}", temp.get(1).unwrap(), light, lightref);
            println!("Plant: {}, Moist Val: {}, Ref: {}", temp.get(1).unwrap(), moist, moistref);
            sender.send([temp.get(1).unwrap().to_string(), npk, refr, tem, trefr, hum, humrefr, light, lightref, moist, moistref]);
        });
        //println!("Value REcieved: {:?}", reciever.recv().expect("Unable to ret array"));
        let output = reciever.recv().expect("Unable to ret array");
        println!("Written Record: {}", out.write_record(&output).is_ok());
        out.flush();
    }
}
