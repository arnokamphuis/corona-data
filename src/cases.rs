use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use chrono::{NaiveDate, DateTime, Date};
use chrono::prelude::*;
use std::collections::BTreeMap;
use curl::easy::Easy;

pub struct Filters {}

#[allow(dead_code)]
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
pub struct PatientCount {
    #[serde(with = "my_date_format")]    
    pub date: NaiveDate,
    pub value: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hospitalization {
    #[serde(with = "my_date_format")]    
    pub Date_statistics: NaiveDate,
    pub ic_patients: usize,
    pub rc_patients: usize,
}

impl Hospitalization {
    pub fn name(&self) -> String {
        self.Date_statistics.format("%Y%m%d").to_string()
    }
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

pub fn get_hospitalizations(from: Option<Date<Utc>>) -> BTreeMap<String, Hospitalization> {
    let mut res: BTreeMap<String, Hospitalization> = BTreeMap::new();
    if let Some(hospitalizations) = get_hospitalizationdata_from_file(from) {
        hospitalizations.iter().for_each(|hospitalization| {
            res.insert( hospitalization.name(), hospitalization.clone());
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
        let reader = BufReader::new(file);

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

pub fn get_hospitalizationdata_from_file(from: Option<Date<Utc>>) -> Option<Vec<Hospitalization>> {
    let mut ic: Vec<PatientCount> = vec![]; // intake_count
    let mut rc: Vec<PatientCount> = vec![]; // zkh_intake_count

    if let Ok(file) = File::open("test-data/intake_count.json") {
        let reader = BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(patients) => {
                if from != None {
                    let all_partients: Vec<PatientCount> = patients;
                    ic = all_partients.iter().filter(|&patient| {
                        return patient.date > from.unwrap().naive_utc() 
                    }).map(|patient| patient.clone() ).collect::<Vec<PatientCount>>();
                } else {
                    ic = patients.clone();
                }
            },
            Err(e) => println!("Error: {:?}", e)
        }
    } else {
        println!("Error reading file");
    }

    if let Ok(file) = File::open("test-data/zkh_intake_count.json") {
        let reader = BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(patients) => {
                if from != None {
                    let all_partients: Vec<PatientCount> = patients;
                    rc = all_partients.iter().filter(|&patient| {
                        return patient.date > from.unwrap().naive_utc() 
                    }).map(|patient| patient.clone() ).collect::<Vec<PatientCount>>();
                } else {
                    rc = patients.clone();
                }
            },
            Err(e) => println!("Error: {:?}", e)
        }
    } else {
        println!("Error reading file");
    }

    let mut res: BTreeMap<String, usize> = BTreeMap::new();

    // let mut lcps_data: BTreeMap<NaiveDate, (usize, usize)> = BTreeMap::new();
    
    // let file = File::open("test-data/lcps-covid-19.csv").unwrap();
    // let mut rdr = csv::Reader::from_reader(file);
    // let header_record = rdr.headers().unwrap().clone();
    // for result in rdr.records() {
    //     let record = result.unwrap();
    //     let resrec =  header_record.iter().zip(record.iter());
    //     let mut date: NaiveDate = NaiveDate::from_ymd(2021,1,1);
    //     let mut ic_count = 0;
    //     let mut rc_count = 0;
    //     for (k, v) in resrec {
    //         match k {
    //             "Datum" => { 
    //                 let parts = v.split("-").map(|s| s.parse::<u32>().unwrap()).collect::<Vec<u32>>();
    //                 date = NaiveDate::from_ymd(parts[2] as i32, parts[1], parts[0]); 
    //             },
    //             "IC_Bedden_COVID" => {
    //                 ic_count = v.parse::<usize>().unwrap();
    //             },
    //             "Kliniek_Bedden" => {
    //                 rc_count = v.parse::<usize>().unwrap();
    //             },
    //             _ => {}
    //         }
    //     }
    //     lcps_data.insert( date, (ic_count,rc_count));
    // }

    let mut hospitalizations: Vec<Hospitalization> = vec![];
    let counts = ic.iter().zip(rc.iter());
    for c in counts {
        hospitalizations.push( Hospitalization{ Date_statistics: c.0.date, ic_patients: c.0.value, rc_patients: c.1.value } );
        // if lcps_data.contains_key(&c.0.date) {
        //     let (ic_count, rc_count) = lcps_data[&c.0.date];
        //     // println!("{:?} => {} vs {}   ===   {} vs {}", c.0.date, c.0.value, ic_count, c.1.value, rc_count);
        //     println!("{:?} => {}   ===   {} ", c.0.date, c.0.value as i32 - ic_count as i32, c.1.value as i32 - rc_count as i32);
        // }
    }

    Some(hospitalizations)
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
    let mut nl_datafile = File::create("test-data/COVID-19_casus_landelijk.json").unwrap();
    let mut handle = Easy::new();
    handle.url("https://data.rivm.nl/covid-19/COVID-19_casus_landelijk.json").unwrap();
    handle.write_function(move |data| {
        nl_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

    let mut nl_rc_intake_datafile = File::create("test-data/zkh_intake_count.json").unwrap();
    let mut handle = Easy::new();
    handle.url("https://stichting-nice.nl/covid-19/public/zkh/intake-count/").unwrap();
    handle.write_function(move |data| {
        nl_rc_intake_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

    let mut nl_ic_intake_datafile = File::create("test-data/intake_count.json").unwrap();
    let mut handle = Easy::new();
    handle.url("https://stichting-nice.nl/covid-19/public/intake-count/").unwrap();
    handle.write_function(move |data| {
        nl_ic_intake_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

    let mut world_datafile = File::create("test-data/time_series_covid19_confirmed_global.csv").unwrap();
    let mut handle = Easy::new();
    handle.url("https://raw.githubusercontent.com/CSSEGISandData/COVID-19/master/csse_covid_19_data/csse_covid_19_time_series/time_series_covid19_confirmed_global.csv").unwrap();
    handle.write_function(move |data| {
        world_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

    let mut lcps_datafile = File::create("test-data/lcps-covid-19.csv").unwrap();
    let mut handle = Easy::new();
    handle.url("https://lcps.nu/wp-content/uploads/covid-19.csv").unwrap();
    handle.write_function(move |data| {
        lcps_datafile.write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    handle.perform().unwrap();    

}