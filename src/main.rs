mod cases;

use std::env;
use cases::*;
use chrono::{Date, Utc};
use chrono::prelude::*;
use plotly::common::{Title, Font};
use plotly::layout::{Axis, BarMode, Layout, AxisType };
use plotly::{Scatter, NamedColor, Plot};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"download".to_string()) { download_data(); }

    let incr_before_20200227 = 217;
    let from: Date<Utc> = Utc.ymd(2020, 2, 27);

    let all_cases    = get_cases(Some(from));
    let (mut dutch_tests, test_total) = get_tests(Some(from)).clone();

    // if totals data is not up to date, we need to add the last day
    let total = incr_before_20200227 + all_cases.iter().fold(0, |acc, (_,cases)| acc + cases.len());
    if dutch_tests.len() < all_cases.len() {
        dutch_tests.insert(all_cases.keys().into_iter().last().unwrap().clone(), total - test_total);
    }

    let calculate_active_cases = | cs: &Vec<f32> | {
        active_cases(&cs, 10)
    };
    create_graph(&all_cases, &dutch_tests, &calculate_active_cases, 10, "Active cases (last 10 days)", "Active cases", "graphs/active_cases.html");


    let calculate_new_cases = | cs: &Vec<f32> | {
        windowed_average(&cs, 3)
    };
    create_graph(&all_cases, &dutch_tests, &calculate_new_cases, 3, "New cases (3 day average)", "New cases", "graphs/new_cases.html");

    let calculate_growth_factor = | cs: &Vec<f32> | {
        windowed_average( &growth_factor( &active_cases( &cs , 10)  ), 5)
    };
    create_graph(&all_cases, &dutch_tests, &calculate_growth_factor, 10+5+1, "Growth factor per age group", "Growth factor", "graphs/growth_factor.html");

    let calculate_growth_of_growth_factor = | cs: &Vec<f32> | {
        windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &cs , 10)  ), 5)), 5)
    };
    create_graph(&all_cases, &dutch_tests, &calculate_growth_of_growth_factor, 10+5+1+5+1, "Growth of the Growth factor per age group", "Growth factor of the growth factor", "graphs/growth_of_growth_factor.html");
}

fn create_graph(
    all_cases: &BTreeMap<String, Vec<Case>>, 
    dutch_tests: &BTreeMap<String, usize>, 
    calculation: &dyn Fn(&Vec<f32>) -> Vec<f32>, 
    filter_size_labels: usize,
    title: &str,
    y_axis_title: &str,
    filename: &str
) {

    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let dutch_counts = | dt: &BTreeMap<String, usize> | -> Vec<f32> {
        dt.iter().map(|(_,&v)| v as f32).collect::<Vec<f32>>()
    };

    let set_cases: Vec<(Vec<f32>, &str)> = vec![
        ( dutch_counts(dutch_tests)                                                 , "All tests"),
        ( case_counts(&all_cases)                                                   , "All cases"), 
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_0_9    ])), "Younger than 10"), 
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_10_19  ])), "Between 10-19"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_20_29  ])), "Between 20-29"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_30_39  ])), "Between 30-39"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_40_49  ])), "Between 40-49"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_50_59  ])), "Between 50-59"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_60_69  ])), "Between 60-69"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_70_79  ])), "Between 70-79"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_80_89  ])), "Between 80-89"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_90_plus])), "Older than 89")
    ];

    let labels = all_cases.iter().skip(filter_size_labels).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let y_data = set_cases.iter().map(|sc| {
        (sc.1.to_string(), calculation(&sc.0))
    }).collect::<Vec<(String, Vec<f32>)>>();

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new(title).font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new(y_axis_title).font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html(filename);
    // let html = plot.to_inline_html(Some("active_cases"));
    // file.write_all(html.as_bytes());
}
