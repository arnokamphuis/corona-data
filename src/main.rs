mod cases;

use cases::{Case, get_data_from_file};
use chrono::{NaiveDate, Datelike, Weekday};
use std::collections::HashMap;

fn main() {

    let months = vec!["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let mut case_bins: HashMap<i32, HashMap<u32, HashMap<u32, Vec<Case>>>> = HashMap::new();

    if let Some(cases) = get_data_from_file() {

        cases.iter().for_each(|case| {
            let ds = case.Date_statistics;
            let year: i32 = ds.year();
            let month: u32 = ds.month();
            let day: u32 = ds.day();

            case_bins.entry(year).or_insert(HashMap::new()).entry(month).or_insert(HashMap::new()).entry(day).or_insert(vec![]).push(case.clone());
        });

        case_bins.iter().for_each(|(year, year_cases)| {
            year_cases.iter().for_each(|(month, month_cases)| {
                println!("{} {}: {}", year, months[*month as usize], month_cases.iter().fold(0, |acc, v| acc + v.1.len()));
                // month_cases.iter().for_each(|(day, day_cases)| {
                //     // println!("{};{};{};{};{};{}", year, month, day, day_cases.iter().filter(|case| case.Sex.eq("Male")).count(), day_cases.iter().filter(|case| case.Sex.eq("Female")).count(), day_cases.len());
                // });
            });
        });

        case_bins.iter().for_each(|(year, year_cases)| {
            year_cases.iter().for_each(|(month, month_cases)| {
                println!("{} {}: {}", year, months[*month as usize], month_cases.iter().fold(0, |acc, v| acc + v.1.iter().filter(|c| c.Deceased.eq("Yes") ).count()));
            });
        });


    }
}
