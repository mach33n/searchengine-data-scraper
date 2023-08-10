pub mod scraper {
    use std::net::{TcpStream};
    use std::io::{Read, Write};
    use html_parser::{Dom, Node};
    use native_tls::TlsConnector;

    pub fn preprocess(id: String, entry: String) -> String {
        let query = format!("What is the '{}' of '{}'?", entry, id);
        query.replace(" ", "+")
    }

    pub fn get(query: String) -> String {
        let connector = TlsConnector::new().unwrap();

        let stream = TcpStream::connect("google.com:443").unwrap();
        let mut stream = connector.connect("google.com", stream).unwrap();

        stream.write_all(format!("GET /search?q={} HTTP/1.0\r\n\r\n", query).as_bytes()).unwrap();
        let mut res = vec![];
        stream.read_to_end(&mut res).unwrap();
        String::from_utf8_lossy(&res).to_string()
    } 

    pub fn scrape_featured(html: String, regex: String) -> Result<String, String> {
        //println!("{}", html.split_once("Accept-Encoding\r\n\r\n").is_none());
        //println!("{}", !html.contains("About Featured Snippets"));
        // Check for featured snippet text
        if !html.contains("About Featured Snippets") {
            // If not present return err
            Err("No featured snippets available".to_string())
        } else {
            // If present scrape
            let html = html.split_once("Accept-Encoding\r\n\r\n").unwrap().1;
            let dom = Dom::parse(html).expect("Unable to parse html");
            let featured = extract_featured_block_html(dom).expect("Unable to identify featured block.");
            // Establish type of featured block
            // Currently only works for paragraph snippets and rich snippets
            let text = extract_text(featured, regex);
            println!("Response: {}\n", text.unwrap());
            // Execute appropriate scraper depending on type
            //let dom_string = Dom::parse(html).expect("Unable to parse html").to_json_pretty().unwrap();
            //println!("{}", dom_string);
            Ok("39".to_string())
        }
    }

    pub fn crawler(html: String, regex: String) -> Result<String, String> {
        Ok("Sample".to_string())
    }

    fn extract_featured_block_html(page: Dom) -> Result<Node, String> {
        let mut idx: usize = 0;
        let mut stack: Vec<(&Node, usize)> = vec![];
        let mut temp: &Node = page.children.get(idx).expect("Empty html document, check value submitted.");
        loop {
            // Base Case: Element containing "V3FYCf" is the lowest level element still containing all of featured 
            // snippet information.
            if temp.element().is_some() && temp.element().unwrap().classes.eq(&vec!["Gx5Zad", "xpd", "EtOod", "pkphOe"]) {
                return Ok(temp.clone());
            } else if temp.element().is_some() && temp.element().unwrap().children.len() > idx {
               // Checks if element has chidren and iterates into children 
               stack.push((&temp, idx + 1));
               temp = temp.element().unwrap().children.get(idx).unwrap();
               idx = 0;
            } else {
                if stack.len() <= 0 {
                    return Err("No Featured Snippet HTML Found".to_string());
                }
                (temp, idx) = stack.pop().unwrap();
            }
        }
    }

    // Essentially runs DFS on a given center node and returns the concat text.
    fn extract_text(page: Node, regex: String) -> Option<String> {
        let mut idx: usize = 0;
        let mut stack: Vec<(&Node, usize)> = vec![];
        let mut temp: &Node = &page.clone();
        let mut ret: Vec<String> = vec![];
        loop {
            if temp.element().is_some() && temp.element().unwrap().children.len() > idx {
               stack.push((&temp, idx + 1));
               temp = temp.element().unwrap().children.get(idx).unwrap();
               idx = 0;
            } else {
                if stack.len() <= 0 {
                    break;
                } else if temp.text().is_some() {
                    ret.push(temp.text().unwrap().to_string());
                }
                (temp, idx) = stack.pop().unwrap();
            }
        }
        if ret.len() <= 0 {
            return None
        }
        return Some(ret.join("\n"));
    }
}
