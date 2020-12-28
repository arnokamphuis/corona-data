mod cases;

use std::env;

use cases::*;
use chrono::{Date, Utc};
use chrono::prelude::*;

use plotly::common::{Title, Font};
use plotly::layout::{Axis, BarMode, Layout, AxisType };
use plotly::{Scatter, NamedColor, Plot};
use std::collections::BTreeMap;

use std::fs::File;
use std::io::prelude::*;

// let months = vec!["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"download".to_string()) {
        download_data();
    }

    let incr_before_20200227 = 217;
    let mut overview_file = File::create("graphs/overview.html").unwrap();
    let from: Date<Utc> = Utc.ymd(2020, 2, 27);

    let all_cases    = get_cases(Some(from));
    let (mut dutch_tests, test_total) = get_tests(Some(from)).clone();

    // if totals data is not up to date, we need to add the last day
    let total = incr_before_20200227 + all_cases.iter().fold(0, |acc, (_,cases)| acc + cases.len());
    if dutch_tests.len() < all_cases.len() {
        dutch_tests.insert(all_cases.keys().into_iter().last().unwrap().clone(), total - test_total);
    }

    create_age_group_new_cases_graph(&all_cases, &dutch_tests, &mut overview_file);
    create_age_group_percentage_graph(&all_cases, &mut overview_file);
    create_age_group_active_cases_graph(&all_cases, &dutch_tests);
    create_age_group_growth_factor_graph(&all_cases, &dutch_tests);
    create_age_group_growth_of_growth_factor_graph(&all_cases, &dutch_tests);
}

fn create_age_group_percentage_graph(all_cases: &BTreeMap<String, Vec<Case>>, file: &mut File) {
    let window = 5;

    let set1_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_0_9]);
    let set2_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_10_19]);
    let set3_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_20_29]);
    let set4_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_30_39]);
    let set5_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_40_49]);
    let set6_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_50_59]);
    let set7_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_60_69]);
    let set8_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_70_79]);
    let set9_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_80_89]);
    let set10_cases = filter_cases(&all_cases, &vec![&Filters::age_group_90_plus]);

    let labels = all_cases.iter().skip(window).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let all_cases_avg   = windowed_average( &all_cases.iter().map(|(d, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set1            = windowed_average( &set1_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set2            = windowed_average( &set2_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set3            = windowed_average( &set3_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set4            = windowed_average( &set4_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set5            = windowed_average( &set5_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set6            = windowed_average( &set6_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set7            = windowed_average( &set7_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set8            = windowed_average( &set8_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set9            = windowed_average( &set9_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set10           = windowed_average( &set10_cases.iter().map(|(d, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);


    let set1_data  = all_cases_avg.iter().zip(set1.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set3_data  = all_cases_avg.iter().zip(set3.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set4_data  = all_cases_avg.iter().zip(set4.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set2_data  = all_cases_avg.iter().zip(set2.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set5_data  = all_cases_avg.iter().zip(set5.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set6_data  = all_cases_avg.iter().zip(set6.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set7_data  = all_cases_avg.iter().zip(set7.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set8_data  = all_cases_avg.iter().zip(set8.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set9_data  = all_cases_avg.iter().zip(set9.iter()).map(|(&total, &value)|  100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();
    let set10_data = all_cases_avg.iter().zip(set10.iter()).map(|(&total, &value)| 100.0f32 * value as f32 / total as f32).collect::<Vec<f32>>();

    let y_data = vec![
        ("Younger than 10".to_string(),   set1_data), 
        ("Between 10-19".to_string(), set2_data), 
        ("Between 20-29".to_string(), set3_data), 
        ("Between 30-39".to_string(), set4_data), 
        ("Between 40-49".to_string(), set5_data), 
        ("Between 50-59".to_string(), set6_data), 
        ("Between 60-69".to_string(), set7_data), 
        ("Between 70-79".to_string(), set8_data), 
        ("Between 80-89".to_string(), set9_data), 
        ("Older than 89".to_string(),   set10_data) 
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Percentage of cases per age group").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Percentage").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![0, 100]));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/percentage.html");
    let html = plot.to_inline_html(Some("percentage-age-groups"));
    (*file).write_all(html.as_bytes());
}

fn create_age_group_new_cases_graph(all_cases: &BTreeMap<String, Vec<Case>>, dutch_tests: &BTreeMap<String, usize>, file: &mut File) {
    let window = 3;

    let set1_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_0_9]);
    let set2_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_10_19]);
    let set3_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_20_29]);
    let set4_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_30_39]);
    let set5_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_40_49]);
    let set6_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_50_59]);
    let set7_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_60_69]);
    let set8_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_70_79]);
    let set9_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_80_89]);
    let set10_cases = filter_cases(&all_cases, &vec![&Filters::age_group_90_plus]);

    let labels = all_cases.iter().skip(window).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let dutch_cases_avg = windowed_average( &dutch_tests.iter().map(|(_, &count)| count as f32).collect::<Vec<f32>>(), window);
    let all_cases_avg   = windowed_average( &all_cases.iter().map(|(d, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set1_data       = windowed_average( &set1_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set2_data       = windowed_average( &set2_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set3_data       = windowed_average( &set3_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set4_data       = windowed_average( &set4_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set5_data       = windowed_average( &set5_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set6_data       = windowed_average( &set6_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set7_data       = windowed_average( &set7_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set8_data       = windowed_average( &set8_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set9_data       = windowed_average( &set9_cases.iter().map(|(d, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set10_data      = windowed_average( &set10_cases.iter().map(|(d, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);

    let y_data = vec![
        ("All tests".to_string(),       dutch_cases_avg),
        ("All cases".to_string(),       all_cases_avg),
        ("Younger than 10".to_string(), set1_data), 
        ("Between 10-19".to_string(),   set2_data), 
        ("Between 20-29".to_string(),   set3_data), 
        ("Between 30-39".to_string(),   set4_data), 
        ("Between 40-49".to_string(),   set5_data), 
        ("Between 50-59".to_string(),   set6_data), 
        ("Between 60-69".to_string(),   set7_data), 
        ("Between 70-79".to_string(),   set8_data), 
        ("Between 80-89".to_string(),   set9_data), 
        ("Older than 89".to_string(),   set10_data) 
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("New cases per age group").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Percentage").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/new_cases.html");
    let html = plot.to_inline_html(Some("new_cases-age-groups"));
    (*file).write_all(html.as_bytes());
}


fn create_age_group_active_cases_graph(all_cases: &BTreeMap<String, Vec<Case>>, dutch_tests: &BTreeMap<String, usize>) {
    let window = 10;

    let set1_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_0_9]);
    let set2_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_10_19]);
    let set3_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_20_29]);
    let set4_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_30_39]);
    let set5_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_40_49]);
    let set6_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_50_59]);
    let set7_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_60_69]);
    let set8_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_70_79]);
    let set9_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_80_89]);
    let set10_cases = filter_cases(&all_cases, &vec![&Filters::age_group_90_plus]);

    let labels = all_cases.iter().skip(window).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let all_dutch_tests    = active_cases( &dutch_tests.iter().map(|(_, &cases)| cases as f32 ).collect::<Vec<f32>>() , window);
    let all_active_cases   = active_cases( &all_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set1               = active_cases( &set1_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set2               = active_cases( &set2_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set3               = active_cases( &set3_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set4               = active_cases( &set4_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set5               = active_cases( &set5_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set6               = active_cases( &set6_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set7               = active_cases( &set7_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set8               = active_cases( &set8_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set9               = active_cases( &set9_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window);
    let set10              = active_cases( &set10_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window);

    let y_data = vec![
        ("All tests".to_string(),       all_dutch_tests), 
        ("All ages".to_string(),        all_active_cases), 
        ("Younger than 10".to_string(), set1), 
        ("Between 10-19".to_string(),   set2), 
        ("Between 20-29".to_string(),   set3), 
        ("Between 30-39".to_string(),   set4), 
        ("Between 40-49".to_string(),   set5), 
        ("Between 50-59".to_string(),   set6), 
        ("Between 60-69".to_string(),   set7), 
        ("Between 70-79".to_string(),   set8), 
        ("Between 80-89".to_string(),   set9), 
        ("Older than 89".to_string(),   set10) 
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Active cases per age group").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Active cases").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/active_cases.html");
    // let html = plot.to_inline_html(Some("active_cases"));
    // file.write_all(html.as_bytes());
}

fn create_age_group_growth_factor_graph(all_cases: &BTreeMap<String, Vec<Case>>, dutch_tests: &BTreeMap<String, usize>) {
    let window = 10;
    let avg = 5;

    let set1_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_0_9]);
    let set2_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_10_19]);
    let set3_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_20_29]);
    let set4_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_30_39]);
    let set5_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_40_49]);
    let set6_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_50_59]);
    let set7_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_60_69]);
    let set8_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_70_79]);
    let set9_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_80_89]);
    let set10_cases = filter_cases(&all_cases, &vec![&Filters::age_group_90_plus]);

    let labels = all_cases.iter().skip(window+avg+1).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let all_dutch_tests    = windowed_average( &growth_factor( &active_cases( &dutch_tests.iter().map(|(_, &cases)| cases as f32 ).collect::<Vec<f32>>() , window)  ), avg);
    let all_active_cases   = windowed_average( &growth_factor( &active_cases( &all_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window)  ), avg);
    let set1               = windowed_average( &growth_factor( &active_cases( &set1_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set2               = windowed_average( &growth_factor( &active_cases( &set2_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set3               = windowed_average( &growth_factor( &active_cases( &set3_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set4               = windowed_average( &growth_factor( &active_cases( &set4_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set5               = windowed_average( &growth_factor( &active_cases( &set5_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set6               = windowed_average( &growth_factor( &active_cases( &set6_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set7               = windowed_average( &growth_factor( &active_cases( &set7_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set8               = windowed_average( &growth_factor( &active_cases( &set8_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set9               = windowed_average( &growth_factor( &active_cases( &set9_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);
    let set10              = windowed_average( &growth_factor( &active_cases( &set10_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg);

    let y_data = vec![
        ("All tests".to_string(),       all_dutch_tests), 
        ("All ages".to_string(),        all_active_cases), 
        ("Younger than 10".to_string(), set1), 
        ("Between 10-19".to_string(),   set2), 
        ("Between 20-29".to_string(),   set3), 
        ("Between 30-39".to_string(),   set4), 
        ("Between 40-49".to_string(),   set5), 
        ("Between 50-59".to_string(),   set6), 
        ("Between 60-69".to_string(),   set7), 
        ("Between 70-79".to_string(),   set8), 
        ("Between 80-89".to_string(),   set9), 
        ("Older than 89".to_string(),   set10) 
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Growth per age group").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Growth factor").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/growth_factor.html");
    // let html = plot.to_inline_html(Some("active_cases"));
    // file.write_all(html.as_bytes());
}

fn create_age_group_growth_of_growth_factor_graph(all_cases: &BTreeMap<String, Vec<Case>>, dutch_tests: &BTreeMap<String, usize>) {
    let window = 10;
    let avg = 5;

    let set1_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_0_9]);
    let set2_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_10_19]);
    let set3_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_20_29]);
    let set4_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_30_39]);
    let set5_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_40_49]);
    let set6_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_50_59]);
    let set7_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_60_69]);
    let set8_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_70_79]);
    let set9_cases  = filter_cases(&all_cases, &vec![&Filters::age_group_80_89]);
    let set10_cases = filter_cases(&all_cases, &vec![&Filters::age_group_90_plus]);

    let labels = all_cases.iter().skip(window+avg+1+avg+1).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let all_dutch_tests    = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &dutch_tests.iter().map(|(_, &cases)| cases as f32 ).collect::<Vec<f32>>() , window)  ), avg)), avg);
    let all_active_cases   = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &all_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window)  ), avg)), avg);
    let set1               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set1_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set2               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set2_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set3               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set3_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set4               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set4_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set5               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set5_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set6               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set6_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set7               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set7_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set8               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set8_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set9               = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set9_cases.iter().map(|(_, cases)|  cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);
    let set10              = windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &set10_cases.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>() , window)), avg)), avg);

    let y_data = vec![
        ("All tests".to_string(),       all_dutch_tests), 
        ("All ages".to_string(),        all_active_cases), 
        ("Younger than 10".to_string(), set1), 
        ("Between 10-19".to_string(),   set2), 
        ("Between 20-29".to_string(),   set3), 
        ("Between 30-39".to_string(),   set4), 
        ("Between 40-49".to_string(),   set5), 
        ("Between 50-59".to_string(),   set6), 
        ("Between 60-69".to_string(),   set7), 
        ("Between 70-79".to_string(),   set8), 
        ("Between 80-89".to_string(),   set9), 
        ("Older than 89".to_string(),   set10) 
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Growth of the growth factor per age group").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Growth factor").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/growth_of_growth_factor.html");
    // let html = plot.to_inline_html(Some("active_cases"));
    // file.write_all(html.as_bytes());
}
