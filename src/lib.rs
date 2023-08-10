pub mod scraper {
    use std::net::{TcpStream};
    use std::io::{Read, Write};
    use html_parser::{Dom, Node};
    use native_tls::TlsConnector;
    struct SnippetText {
        original_text: String,
        bold_text: String
    }

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
        // Check for featured snippet text
        if !html.contains("About Featured Snippets") {
            // If not present return err
            Err("No featured snippets available".to_string())
        } else {
            // If present scrape
            let html = html.split_once("Accept-Encoding\r\n\r\n").unwrap().1;
            let dom = Dom::parse(html).expect("Unable to parse html");
            // Extract featured snippet html
            let featured = extract_featured_block_html(dom).expect("Unable to identify featured block.");
            // Currently only works for paragraph snippets and rich snippets
            let text = extract_text(featured, regex, true).unwrap();
            println!("Original Response: {}\n", text.original_text);
            println!("Bold Text: {}\n", text.bold_text);
            if text.bold_text.len() > 0 {
                return Ok(text.bold_text);
            }
            return Ok(text.original_text);
        }
    }

    pub fn crawler(html: String, regex: String) -> Result<String, String> {
        let html = html.split_once("Accept-Encoding\r\n\r\n").expect("Unable to split HTML on Accept-Encoding").1;
        let dom = Dom::parse(html).expect("Unable to parse html");
        // Loop through snippets for first matching regex
        let snippets = extract_snippet_blocks(dom.clone()).unwrap();
        let out = extract_text(snippets.first().unwrap().clone(), regex, true).unwrap();
        println!("Original Response: {}\n", out.original_text);
        println!("Bold Text: {}\n", out.bold_text);
        return Ok(out.bold_text);
    }

    fn extract_snippet_blocks(page: Dom) -> Result<Vec<Node>, String> {
        let mut idx: usize = 0;
        let mut stack: Vec<(&Node, usize)> = vec![];
        let mut temp: &Node = page.children.get(idx).expect("Empty html document, check value submitted.");
        let mut ret: Vec<Node> = vec![];
        loop {
            // Base Case: Element containing specified class is the lowest level element still containing all of featured 
            // snippet information.
            if temp.element().is_some() && temp.element().unwrap().classes.eq(&vec!["Gx5Zad", "fP1Qef", "xpd", "EtOod", "pkphOe"]) {
                ret.push(temp.clone());
                break;
            } else if temp.element().is_some() && temp.element().unwrap().children.len() > idx {
               // Checks if element has chidren and iterates into children 
               stack.push((&temp, idx + 1));
               temp = temp.element().unwrap().children.get(idx).unwrap();
               idx = 0;
            } else {
                if stack.len() <= 0 {
                    break;
                }
                (temp, idx) = stack.pop().unwrap();
            }
        }
        if ret.len() <= 0 {
            return Err("No snippets found.".to_string());
        }
        return Ok(ret);
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
               temp = temp.element().unwrap().children.get(idx).expect("Cannot unwrap child");
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
    fn extract_text(page: Node, regex: String, seperate_bold: bool) -> Option<SnippetText> {
        let mut idx: usize = 0;
        let mut stack: Vec<(&Node, usize)> = vec![];
        let mut temp: &Node = &page.clone();
        let mut ret: Vec<String> = vec![];
        let mut bold_text: Vec<String> = vec![];
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
                if temp.element().unwrap().classes.eq(&vec!["FCUp0c", "rQMQod"]) {
                    bold_text.push(ret.last().clone().unwrap().to_string());
                }
            }
        }
        if ret.len() <= 0 {
            return None
        }
        return Some(SnippetText { original_text: ret.join("\n"), bold_text: bold_text.join(" ")});
    }
}
