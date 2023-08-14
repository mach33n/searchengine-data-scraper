pub mod regexlib {
    use regex::{Regex};
    use std::str::FromStr;

    #[derive(Clone)]
    pub enum RegexType {
        Numeric,
        NumericOnly,
        Stringy,
        Rangey,
        Custom(String)
    }
    
    #[derive(Clone)]
    pub struct RegBank {
        pub banktype: RegexType,
        pub reg: Regex
    }

    impl FromStr for RegexType {
        type Err = ();

        fn from_str(input: &str) -> Result<RegexType, Self::Err> {
            match input.to_lowercase().as_str() {
                "numeric" => Ok(RegexType::Numeric),
                "numeric_only" => Ok(RegexType::NumericOnly),
                "string" => Ok(RegexType::Stringy),
                "range" => Ok(RegexType::Rangey),
                val => Ok(RegexType::Custom(val.to_string().clone())),
            }
        }
    }

    impl RegBank {
        pub fn new(btype: RegexType) -> RegBank {
            let binding = btype.clone();
            let rstring: &str = match binding {
                RegexType::Numeric => r"\d+\S*",
                RegexType::NumericOnly => r"\d+",
                RegexType::Stringy => r"\w+",
                RegexType::Rangey => r"",
                RegexType::Custom(ref val) => val.as_str(),
            };
            RegBank {
                banktype: btype,
                reg: Regex::new(&rstring.to_string()).unwrap() 
            }
        }
    }
}