use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use chrono::{NaiveDate, DateTime, Date};
use chrono::prelude::*;
use std::collections::BTreeMap;
use curl::easy::Easy;

pub struct Filters {}

impl Filters {
    #[inline]
    pub fn male              (c: &Case) -> bool { c.Sex.eq("Male") }
    #[inline]
    pub fn female            (c: &Case) -> bool { return c.Sex.eq("Female") }
    #[inline]
    pub fn alive             (c: &Case) -> bool { return !c.Deceased.eq("Yes") }
    #[inline]
    pub fn dead              (c: &Case) -> bool { return c.Deceased.eq("Yes") }
    #[inline]
    pub fn age_group_0_9     (c: &Case) -> bool { return c.Agegroup.eq("0-9") } // only alive
    #[inline]
    pub fn age_group_10_19   (c: &Case) -> bool { return c.Agegroup.eq("10-19") } // only alive
    #[inline]
    pub fn age_group_20_29   (c: &Case) -> bool { return c.Agegroup.eq("20-29") } // only alive
    #[inline]
    pub fn age_group_30_39   (c: &Case) -> bool { return c.Agegroup.eq("30-39") } // only alive
    #[inline]
    pub fn age_group_40_49   (c: &Case) -> bool { return c.Agegroup.eq("40-49") } // only alive
    #[inline]
    pub fn age_group_min_50  (c: &Case) -> bool { return c.Agegroup.eq("<50") }  // only dead
    #[inline]
    pub fn age_group_50_59   (c: &Case) -> bool { return c.Agegroup.eq("50-59") }
    #[inline]
    pub fn age_group_60_69   (c: &Case) -> bool { return c.Agegroup.eq("60-69") }
    #[inline]
    pub fn age_group_70_79   (c: &Case) -> bool { return c.Agegroup.eq("70-79") }
    #[inline]
    pub fn age_group_80_89   (c: &Case) -> bool { return c.Agegroup.eq("80-89") }
    #[inline]
    pub fn age_group_90_plus (c: &Case) -> bool { return c.Agegroup.eq("90+") }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Case {
    #[serde(with = "my_datetime_format")]    
    pub Date_file: DateTime<Utc>,
    #[serde(with = "my_date_format")]    
    pub Date_statistics: NaiveDate,
    pub Date_statistics_type: String,
    pub Agegroup: String,
    pub Sex: String,
    pub Province: String,
    pub Hospital_admission: String,
    pub Deceased: String,
    pub Week_of_death: Option<String>,
    pub Municipal_health_service: String    
}

impl Case {
    pub fn name(&self) -> String {
        self.Date_statistics.format("%Y%m%d").to_string()
    }
}

pub fn windowed_average(values: &Vec<f32>, window: usize) -> Vec<f32> {
    values.iter().enumerate().skip(window).map(|(index, _)| {
        values[index-window+1..index+1].to_vec().iter().sum::<f32>() / (window as f32)
    }).collect::<Vec<f32>>()
}

pub fn active_cases(case_counts: &Vec<f32>, window: usize) -> Vec<f32> {
    case_counts.iter().enumerate().skip(window).map(|(index, _)| {
        case_counts[index-window..index+1].to_vec().iter().sum::<f32>()
    }).collect::<Vec<f32>>()
}

pub fn growth_factor(case_counts: &Vec<f32>) -> Vec<f32> {
    case_counts.iter().enumerate().skip(1).map(|(index, &v)| {
        v as f32 / case_counts[index-1] as f32 
    }).collect::<Vec<f32>>()
}

pub fn filter_cases(cases: &BTreeMap<String, Vec<Case>>, filters: &Vec<&dyn Fn(&Case) -> bool>) -> BTreeMap<String, Vec<Case>> {
    let mut res: BTreeMap<String, Vec<Case>> = BTreeMap::new();
    let labels = cases.iter().map(|(name,_)| name.clone() ).collect::<Vec<String>>();

    cases.iter().for_each(|(name, cs)| {
        res.insert(name.clone(), cs.iter().filter(|&c| filters.iter().fold(true, |acc, &func| acc & func(c) )).map(|c| c.clone()).collect::<Vec<Case>>());
    });

    labels.iter().for_each(|name| {
        if !res.contains_key(name) {
            res.insert(name.clone(), vec![]);
        }
    });

    res
}

pub fn get_cases(from: Option<Date<Utc>>) -> BTreeMap<String, Vec<Case>> {
    let mut res: BTreeMap<String, Vec<Case>> = BTreeMap::new();
    if let Some(cases) = get_data_from_file(from) {
        cases.iter().for_each(|case| {
            res.entry(case.name()).or_insert(vec![]).push(case.clone());
        });
    }
    res
}

pub fn get_tests(from: Option<Date<Utc>>) -> (BTreeMap<String, usize>, usize) {
    let mut res: BTreeMap<String, usize> = BTreeMap::new();
    
    let file = File::open("test-data/time_series_covid19_confirmed_global.csv").unwrap();
    let mut rdr = csv::Reader::from_reader(file);

    let header_record = rdr.headers().unwrap().clone();

    let mut total = 0;
    for result in rdr.records() {
        let record = result.unwrap();
        let resrec =  header_record.iter().zip(record.iter());
        let mut country = String::from("");
        let mut last = 0;
        for rr in resrec {
            match rr.0 {
                "Province/State" | "Country/Region" => { country.push_str(rr.1); },
                "Lat" | "Long" => {}
                _ => {
                    if country.eq("Netherlands") {
                        let date_parts = rr.0.split('/').map(|v| v.parse::<usize>().unwrap()).collect::<Vec<usize>>();
                        let date = Utc.ymd(date_parts[2] as i32 + 2000, date_parts[0] as u32, date_parts[1] as u32);
                        if (from != None && date > from.unwrap()) || from == None {
                            let date_str = format!("20{:02}{:02}{:02}", date_parts[2], date_parts[0], date_parts[1]);
                            let thisvalue = rr.1.parse::<usize>().unwrap();
                            res.insert(date_str.clone(), thisvalue - last);
                            total = thisvalue;
                            last = thisvalue;
                        }
                    }
                } 
            }
        }
    }

    (res, total)
}

pub fn get_data_from_file(from: Option<Date<Utc>>) -> Option<Vec<Case>> {
    // Open the file in read-only mode with buffer.

    if let Ok(file) = File::open("test-data/COVID-19_casus_landelijk.json") {
    // if let Ok(file) = File::open("test-data/test.json") {
        let mut reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`
        match serde_json::from_reader(reader) {
            Ok(cases) => {
                if from != None {
                    let all_cases: Vec<Case> = cases;
                    return Some(all_cases.iter().filter(|&case| {
                        return case.Date_statistics > from.unwrap().naive_utc() 
                    }).map(|case| case.clone() ).collect::<Vec<Case>>());
                }
                return Some(cases)
            },
            Err(e) => println!("Error: {:?}", e)
        }
    } else {
        println!("Error reading file");
    }
    None
}

mod my_datetime_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};
    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}


mod my_date_format {
    use chrono::{NaiveDate};
    use serde::{self, Deserialize, Serializer, Deserializer};
    const FORMAT: &'static str = "%Y-%m-%d";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub fn download_data() {
    let mut NL_datafile = File::create("test-data/COVID-19_casus_landelijk.json").unwrap();
    let mut handle = Easy::new();
    handle.url("https://data.rivm.nl/covid-19/COVID-19_casus_landelijk.json").unwrap();
    handle.write_function(move |data| {
        NL_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

    let mut WORLD_datafile = File::create("test-data/time_series_covid19_confirmed_global.csv").unwrap();
    let mut handle = Easy::new();
    handle.url("https://raw.githubusercontent.com/CSSEGISandData/COVID-19/master/csse_covid_19_data/csse_covid_19_time_series/time_series_covid19_confirmed_global.csv").unwrap();
    handle.write_function(move |data| {
        WORLD_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

}