use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use curl::easy::Easy;
use chrono::{NaiveDate, DateTime};
use chrono::prelude::*;

// {"Date_file":"2020-12-26 10:00:00","Date_statistics":"2020-01-01","Date_statistics_type":"DOO","Agegroup":"40-49","Sex":"Female","Province":"Noord-Holland","Hospital_admission":"No","Deceased":"No","Week_of_death":null,"Municipal_health_service":"GGD Amsterdam"},
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Cases {
    items: Vec<Case>
}

pub fn get_data_from_file() -> Option<Vec<Case>> {
    // Open the file in read-only mode with buffer.

    if let Ok(file) = File::open("test-data/COVID-19_casus_landelijk.json") {
    // if let Ok(file) = File::open("test-data/test.json") {
        let mut reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`
        match serde_json::from_reader(reader) {
            Ok(cases) => return Some(cases),
            Err(e) => println!("Error: {:?}", e)
        }
    } else {
        println!("Error reading file");
    }
    None
}

pub fn get_data_from_rivm() {
    let url = "https://data.rivm.nl/covid-19/COVID-19_casus_landelijk.json";

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(url).unwrap();

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    println!("{:?}", data);
}

// fn untyped_example() -> Result<()> {
//     // Some JSON input data as a &str. Maybe this comes from the user.
//     let data = r#"
//         {
//             "name": "John Doe",
//             "age": 43,
//             "phones": [
//                 "+44 1234567",
//                 "+44 2345678"
//             ]
//         }"#;

//     // Parse the string of data into serde_json::Value.
//     let v: Value = serde_json::from_str(data)?;

//     // Access parts of the data by indexing with square brackets.
//     println!("Please call {} at the number {}", v["name"], v["phones"][0]);

//     Ok(())
// }


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