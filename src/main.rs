mod cases;

use std::env;
use cases::*;
use chrono::{Date, Utc, Duration};
use chrono::prelude::*;
use plotly::common::{Title, Font};
use plotly::layout::{Axis, BarMode, Layout, AxisType };
use plotly::{Scatter, NamedColor, Plot};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::io::prelude::*;
use linreg::linear_regression_of;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"download".to_string()) { download_data(); }

    let mut overview_file = File::create("graphs/index.html").unwrap();

    let incr_before_20200227 = 217;
    let from: Date<Utc> = Utc.ymd(2020, 2, 27);

    let all_cases    = get_cases(Some(from));
    let (mut dutch_tests, test_total) = get_tests(Some(from)).clone();
    let all_hospitalizations = get_hospitalizations(Some(from));
    let all_prevalences = get_prevalences(Some(from));

    // if totals data is not up to date, we need to add the last day
    let total = incr_before_20200227 + all_cases.iter().fold(0, |acc, (_,cases)| acc + cases.len());
    if dutch_tests.len() < all_cases.len() {
        dutch_tests.insert(all_cases.keys().into_iter().last().unwrap().clone(), total - test_total);
    }

    let delay = find_delay(&dutch_tests, &all_cases);
    println!("delay: {:?}", delay);

    write_header(&mut overview_file);

    let calculate_active_cases = | cs: &Vec<f32>, factors: &Vec<f32> | {
        let ac = active_cases(&cs, 10);
        let mut res: Vec<f32> = vec![];
        for (f, v) in factors.iter().zip(ac.iter()) {
            res.push(v/f);
        }
        res
    };
    create_graph(&all_cases, &dutch_tests, &all_prevalences, &calculate_active_cases, 10, "Approximate infectious persons", "Active cases", "graphs/active_cases.html", "active_cases", &mut overview_file);

    let calculate_new_cases = | cs: &Vec<f32>, _: &Vec<f32> | {
        windowed_average(&cs, 3)
    };
    create_graph(&all_cases, &dutch_tests, &all_prevalences, &calculate_new_cases, 3, "New cases (3 day average)", "New cases", "graphs/new_cases.html", "new_cases", &mut overview_file);

    let calculate_growth_factor = | cs: &Vec<f32>, _: &Vec<f32> | {
        windowed_average( &growth_factor( &active_cases( &cs , 10)  ), 5)
    };
    create_graph(&all_cases, &dutch_tests, &all_prevalences, &calculate_growth_factor, 10+5+1, "Growth factor per age group", "Growth factor", "graphs/growth_factor.html", "growth", &mut overview_file);

    // let calculate_growth_of_growth_factor = | cs: &Vec<f32> | {
    //     windowed_average(&growth_factor(&windowed_average( &growth_factor( &active_cases( &cs , 10)  ), 5)), 5)
    // };
    // create_graph(&all_cases, &dutch_tests, &calculate_growth_of_growth_factor, 10+5+1+5+1, "Growth of the Growth factor per age group", "Growth factor of the growth factor", "graphs/growth_of_growth_factor.html", "growth_growth", &mut overview_file);

    hospitalization_graph(&all_hospitalizations, "hospitalizations", &mut overview_file);

    trends(&all_cases, &all_hospitalizations, &all_prevalences, "trends", &mut overview_file);
    
    trends_of_trends(&all_cases, &all_hospitalizations, &all_prevalences, "trendsoftrends", &mut overview_file);

    write_footer(&mut overview_file);

    prevalence_factor_graph(&all_cases, &all_prevalences);

    calculate_peaks( &all_cases, &all_prevalences);
}

fn create_graph(
    all_cases: &BTreeMap<String, Vec<Case>>, 
    dutch_tests: &BTreeMap<String, usize>, 
    all_prevalences: &BTreeMap<String, Prevalence>,
    calculation: &dyn Fn(&Vec<f32>, &Vec<f32>) -> Vec<f32>, 
    filter_size_labels: usize,
    title: &str,
    y_axis_title: &str,
    filename: &str,
    div_name: &'static str,
    overview_file: &mut File
) {

    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let dutch_counts = | dt: &BTreeMap<String, usize> | -> Vec<f32> {
        dt.iter().map(|(_,&v)| v as f32).collect::<Vec<f32>>()
    };

    let set_cases: Vec<(Vec<f32>, &str)> = vec![
        ( dutch_counts(dutch_tests)                                                 , "Tests"),
        ( case_counts(&all_cases)                                                   , "All"), 
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_0_9    ])), " 0-9 "), 
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_10_19  ])), "10-19"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_20_29  ])), "20-29"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_30_39  ])), "30-39"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_40_49  ])), "40-49"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_50_59  ])), "50-59"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_60_69  ])), "60-69"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_70_79  ])), "70-79"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_80_89  ])), "80-89"),
        ( case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_90_plus])), "90-xx")
    ];

    let factors = get_scale_factors(&all_cases, &all_prevalences).iter().skip(0).map(|&v| v).collect::<Vec<f32>>();

    let labels = all_cases.iter().skip(filter_size_labels).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let y_data = set_cases.iter().map(|sc| {
        (sc.1.to_string(), calculation(&sc.0, &factors))
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
    let html = plot.to_inline_html(Some(div_name));
    overview_file.write_all(html.as_bytes());
    overview_file.write_all(b"\n");
}


pub fn write_header(file: &mut File) {
    let header = File::open("template/header.html").unwrap();
    let reader = BufReader::new(header);
    for line in reader.lines() {
        file.write_all(line.unwrap().as_bytes());
        // println!("{}", line.unwrap());
    }
}

pub fn write_footer(file: &mut File) {
    let header = File::open("template/footer.html").unwrap();
    let reader = BufReader::new(header);
    for line in reader.lines() {
        file.write_all(line.unwrap().as_bytes());
        // println!("{}", line.unwrap());
    }
}


pub fn trends(
    all_cases: &BTreeMap<String, Vec<Case>>, 
    all_hospitalizations: &BTreeMap<String, Hospitalization>, 
    all_prevalences: &BTreeMap<String, Prevalence>,
    div_name: &'static str, 
    overview_file: &mut File
) {
    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let last_case_date: String = all_cases.iter().last().unwrap().0.clone();

    let set_cases: Vec<(Vec<f32>, &str)> = vec![
        ( active_cases(&case_counts(&all_cases)                                                   ,10), "All"), 
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_0_9    ])),10), " 0-9 "), 
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_10_19  ])),10), "10-19"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_20_29  ])),10), "20-29"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_30_39  ])),10), "30-39"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_40_49  ])),10), "40-49"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_50_59  ])),10), "50-59"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_60_69  ])),10), "60-69"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_70_79  ])),10), "70-79"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_80_89  ])),10), "80-89"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_90_plus])),10), "90-xx"),
        ( all_hospitalizations.iter().filter(|(name,_)| (*name).cmp(&last_case_date) != std::cmp::Ordering::Greater ).map(|(_,h)| h.ic_patients as f32).collect::<Vec<f32>>().clone() , "IC"),
        ( all_hospitalizations.iter().filter(|(name,_)| (*name).cmp(&last_case_date) != std::cmp::Ordering::Greater ).map(|(_,h)| h.rc_patients as f32).collect::<Vec<f32>>().clone() , "RC"),
    ];

    let factors = get_scale_factors(&all_cases, &all_prevalences);

    let start_day = 1;
    let max_days_back = 224;
    let mut results: BTreeMap<String, Vec<(f32,f32)>> = BTreeMap::new();

    for days_back in (start_day..max_days_back).rev() {
        let last_seven = set_cases.iter().map(|cases| 
            (
                cases.1.to_string(), 
                cases.0.iter().rev().skip(days_back).take(7).rev().enumerate().map(|(index, &v)| (index as f32,v / factors[factors.len()-(days_back+6-index)-1]) ).collect::<Vec<(f32,f32)>>()
            ) 
        ).collect::<Vec<(String, Vec<(f32,f32)>)>>();

        for set in &last_seven {
            // let last_value = set.1.iter().rev().next().unwrap().1;
            let lr: (f32, f32) = linear_regression_of(&set.1).unwrap();
            results.entry(set.0.clone()).or_insert(vec![]).push((0.0 - days_back as f32,lr.0));
            // results.entry(set.0.clone()).or_insert(vec![]).push((0.0 - days_back as f32,lr.0 / last_value));
        }
    }

    let labels = all_cases.iter().rev().skip(start_day).take(max_days_back-start_day).rev().map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let begin = labels.iter().rev().skip(7).next().unwrap();
    let end = labels.iter().last().unwrap();

    let y_data = set_cases.iter().map(|sc| {
        (sc.1.to_string(), results[sc.1].iter().map(|(_,v)| *v).collect::<Vec<f32>>() )
    }).collect::<Vec<(String, Vec<f32>)>>();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Rel. change in active cases (7 day lin.reg.)").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Increase/decrease").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/linreg.html");
    let html = plot.to_inline_html(Some(div_name));
    overview_file.write_all(html.as_bytes());
    overview_file.write_all(b"\n");

}

fn hospitalization_graph(all_hospitalizations: &BTreeMap<String, Hospitalization>, div_name: &'static str, overview_file: &mut File) {
    let labels = all_hospitalizations.iter().skip(5).map(|(name,_)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let y_data = vec![
        ("IC".to_string(), windowed_average(&all_hospitalizations.iter().map(|(_,h)| h.ic_patients as f32 ).collect::<Vec<f32>>(), 3)),
        ("RC".to_string(),   windowed_average(&all_hospitalizations.iter().map(|(_,h)| h.rc_patients as f32 ).collect::<Vec<f32>>(), 3))
    ];

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Hospitalizations per day").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Patients in care").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/hospitalizations.html");
    let html = plot.to_inline_html(Some(div_name));
    overview_file.write_all(html.as_bytes());
    overview_file.write_all(b"\n");

}

fn find_delay(dutch_tests: &BTreeMap<String, usize>, all_cases: &BTreeMap<String, Vec<Case>>) -> (f32, f32) {
    let in_between = |v: f32, b: f32, e: f32| { (b <= v && v <= e) || (e <= v && v <= b) };
    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let all_counts = case_counts(all_cases);
    let all_tests = dutch_tests.iter().map(|(_,&c)| c as f32 ).collect::<Vec<f32>>();

    let diffs = all_counts.iter().enumerate().map(|(index, &value)|  {
        let b: usize = index+1;
        let e: usize = &all_tests.len()-1;
        for index2 in b..e {
            if in_between(value, all_tests[index2], all_tests[index2+1]) {
                return (index2-index) as f32;
            }
        }
        0f32
    }).filter(|&v| v > 0.0f32).collect::<Vec<f32>>();

    let cases_delay = diffs.iter().sum::<f32>() / diffs.len() as f32;

    let all_counts = active_cases(&case_counts(all_cases), 10);
    let all_tests = active_cases(&dutch_tests.iter().map(|(_,&c)| c as f32 ).collect::<Vec<f32>>(), 10);

    let diffs = all_counts.iter().enumerate().map(|(index, &value)|  {
        let b: usize = index+1;
        let e: usize = &all_tests.len()-1;
        for index2 in b..e {
            if in_between(value, all_tests[index2], all_tests[index2+1]) {
                return (index2-index) as f32;
            }
        }
        0f32
    }).filter(|&v| v > 0.0f32).collect::<Vec<f32>>();

    let infectious_delay = diffs.iter().sum::<f32>() / diffs.len() as f32;

    (cases_delay, infectious_delay)
    
}





pub fn trends_of_trends(
    all_cases: &BTreeMap<String, Vec<Case>>, 
    all_hospitalizations: &BTreeMap<String, Hospitalization>, 
    all_prevalences: &BTreeMap<String, Prevalence>,
    div_name: &'static str, 
    overview_file: &mut File
) {
    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let last_case_date: String = all_cases.iter().last().unwrap().0.clone();

    let set_cases: Vec<(Vec<f32>, &str)> = vec![
        ( active_cases(&case_counts(&all_cases)                                                   ,10), "All"), 
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_0_9    ])),10), " 0-9 "), 
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_10_19  ])),10), "10-19"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_20_29  ])),10), "20-29"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_30_39  ])),10), "30-39"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_40_49  ])),10), "40-49"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_50_59  ])),10), "50-59"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_60_69  ])),10), "60-69"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_70_79  ])),10), "70-79"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_80_89  ])),10), "80-89"),
        ( active_cases(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_90_plus])),10), "90-xx"),
        ( all_hospitalizations.iter().filter(|(name,_)| (*name).cmp(&last_case_date) != std::cmp::Ordering::Greater ).map(|(_,h)| h.ic_patients as f32).collect::<Vec<f32>>().clone() , "IC"),
        ( all_hospitalizations.iter().filter(|(name,_)| (*name).cmp(&last_case_date) != std::cmp::Ordering::Greater ).map(|(_,h)| h.rc_patients as f32).collect::<Vec<f32>>().clone() , "RC"),
    ];

    let factors = get_scale_factors(&all_cases, &all_prevalences);

    let start_day = 1;
    let max_days_back = 224;
    // let max_days_back = set_cases[0].0.len()-7;
    let mut trends: BTreeMap<String, Vec<(f32,f32)>> = BTreeMap::new();
    let mut trends_of_trends: BTreeMap<String, Vec<(f32,f32)>> = BTreeMap::new();

    set_cases.iter().skip(start_day).for_each(|(_,name)| { trends.insert(name.to_string(), vec![]); trends_of_trends.insert(name.to_string(), vec![]); });

    for days_back in (start_day..max_days_back).rev() {
        // let last_seven = set_cases.iter().map(|cases| (cases.1.to_string(), cases.0.iter().rev().skip(days_back).take(7).rev().enumerate().map(|(index, &v)| (index as f32,v) ).collect::<Vec<(f32,f32)>>()) ).collect::<Vec<(String, Vec<(f32,f32)>)>>();
        let last_seven = set_cases.iter().map(|cases| 
            (
                cases.1.to_string(), 
                cases.0.iter().rev().skip(days_back).take(7).rev().enumerate().map(|(index, &v)| (index as f32,v / factors[factors.len()-(days_back+6-index)-1]) ).collect::<Vec<(f32,f32)>>()
            ) 
        ).collect::<Vec<(String, Vec<(f32,f32)>)>>();

        for set in &last_seven {
            let last_value = set.1.iter().rev().next().unwrap().1;
            let lr: (f32, f32) = linear_regression_of(&set.1).unwrap();
            trends.entry(set.0.clone()).or_insert(vec![]).push((0.0 - days_back as f32,lr.0));
        }
    }

    let trend_data = set_cases.iter().map(|sc| {
        ( trends[sc.1].iter().map(|(_,v)| *v).collect::<Vec<f32>>(), sc.1.to_string() )
    }).collect::<Vec<(Vec<f32>,String)>>();

    let start_day = 0;
    let max_days_back = trend_data[0].0.len()-7;
    for days_back in (start_day..max_days_back).rev() {
        let last_seven = trend_data.iter().map(|cases| (cases.1.to_string(), cases.0.iter().rev().skip(days_back).take(7).rev().enumerate().map(|(index, &v)| (index as f32,v) ).collect::<Vec<(f32,f32)>>()) ).collect::<Vec<(String, Vec<(f32,f32)>)>>();

        for set in &last_seven {
            let lr: (f32, f32) = linear_regression_of(&set.1).unwrap();
            trends_of_trends.entry(set.0.clone()).or_insert(vec![]).push((0.0 - days_back as f32,lr.0));
        }
    }

    let labels = all_cases.iter().rev().skip(1).take(max_days_back-1).rev().map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let begin = labels.iter().rev().skip(7).next().unwrap();
    let end = labels.iter().last().unwrap();

    let y_data = set_cases.iter().map(|sc| {
        ( sc.1.to_string(), trends_of_trends[sc.1].iter().map(|(_,v)| *v).collect::<Vec<f32>>() )
    }).collect::<Vec<(String, Vec<f32>)>>();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Change of the change in active cases (7 day lin.reg.)").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("Increase/decrease").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    y_data.iter().for_each(|(name, data)| {
        plot.add_trace( Scatter::new( labels.clone(), data.clone() ).name(name) )
    });
    plot.set_layout(layout);

    plot.to_html("graphs/trends_of_trends.html");
    let html = plot.to_inline_html(Some(div_name));
    overview_file.write_all(html.as_bytes());
    overview_file.write_all(b"\n");

}

pub fn get_scale_factors(all_cases: &BTreeMap<String, Vec<Case>>, all_prevalences: &BTreeMap<String, Prevalence>) -> Vec<f32> {
    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let set_cases = active_cases(&case_counts(all_cases), 10);
    let mut set_prevs = all_prevalences.iter().skip(10).map(|(name, prev)|  (prev.prev_up + prev.prev_low) as f32 / 2.0f32).collect::<Vec<f32>>();
    let mut res: Vec<f32> = vec![];
    for (c, p) in set_cases.iter().zip(set_prevs.iter()) {
        res.push( *c / *p );
    }
    let last_value = *res.iter().last().unwrap();
    while res.len() < set_cases.len() {
        res.push(last_value);
    }
    res
}


pub fn prevalence_factor_graph(all_cases: &BTreeMap<String, Vec<Case>>, all_prevalences: &BTreeMap<String, Prevalence>) {
    let factors = get_scale_factors(all_cases, all_prevalences).iter().map(|&v| 1.0f32 / v).collect::<Vec<f32>>();

    let labels = all_cases.iter().skip(10).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        return dashed_name;
    }).collect::<Vec<String>>();

    let begin = labels.iter().rev().skip(30).next().unwrap();
    let end = labels.iter().last().unwrap();

    let layout = Layout::new().bar_mode(BarMode::Group)
        .title(Title::new("Factors").font(Font::new().color(NamedColor::Black).size(24).family("Droid Serif")))
        .x_axis(Axis::new().type_(AxisType::Date).title(Title::new("Day").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))).range(vec![begin,end]))
        .y_axis(Axis::new().title(Title::new("factor").font(Font::new().color(NamedColor::Black).size(12).family("Droid Serif"))));

    let mut plot = Plot::new();
    plot.add_trace( Scatter::new( labels.clone(), factors.clone()));
    plot.set_layout(layout);
    plot.to_html("graphs/factors.html");
}


pub fn calculate_peaks(all_cases: &BTreeMap<String, Vec<Case>>, all_prevalences: &BTreeMap<String, Prevalence>) {
    let calculate_active_cases = | cs: &Vec<f32>, factors: &Vec<f32> | {
        let ac = active_cases(&cs, 10);
        let mut res: Vec<f32> = vec![];
        for (f, v) in factors.iter().zip(ac.iter()) {
            res.push(v/f);
        }
        res
    };

    let case_counts = | cs: &BTreeMap<String, Vec<Case>> | -> Vec<f32> {
        cs.iter().map(|(_, cases)| cases.len() as f32 ).collect::<Vec<f32>>()
    };

    let factors = get_scale_factors(&all_cases, &all_prevalences).iter().skip(2).map(|&v| v).collect::<Vec<f32>>();

    let set_cases: Vec<(Vec<f32>, &str)> = vec![
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_0_9    ])),2), &factors), " 0-9 "), 
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_10_19  ])),2), &factors), "10-19"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_20_29  ])),2), &factors), "20-29"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_30_39  ])),2), &factors), "30-39"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_40_49  ])),2), &factors), "40-49"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_50_59  ])),2), &factors), "50-59"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_60_69  ])),2), &factors), "60-69"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_70_79  ])),2), &factors), "70-79"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_80_89  ])),2), &factors), "80-89"),
        ( calculate_active_cases(&windowed_average(&case_counts(&filter_cases(&all_cases, &vec![&Filters::age_group_90_plus])),2), &factors), "90-xx")
    ];

    assert!(set_cases[0].0.len()==factors.len());

    let determine_peaks = | cs: &Vec<f32> | {
        cs.iter().enumerate().skip(2).rev().skip(2).rev().filter(|(index, &v)|
            cs[index-2] < cs[index-1] && 
            cs[index-1] < v && 
            v > cs[index+1] &&
            cs[index+1] > cs[index+2]
        ).map(|(index,_)| (index-1) as usize ).collect::<Vec<usize>>()
    };

    let labels = all_cases.iter().skip(10).map(|(name,case)| {
        let mut dashed_name = name.clone();
        dashed_name.insert(6,'-',);
        dashed_name.insert(4,'-',);
        let parts = dashed_name.split('-').collect::<Vec<&str>>();
        return NaiveDate::from_ymd(parts[0].parse::<i32>().unwrap(), parts[1].parse::<u32>().unwrap(), parts[2].parse::<u32>().unwrap());
    }).collect::<Vec<NaiveDate>>();

    let mut peaks = set_cases.iter().fold(BTreeMap::new(), |mut acc, (v, name)| {
        acc.entry(*name).or_insert(determine_peaks(v).iter().rev().map(|&index| labels[index].clone()).collect::<Vec<NaiveDate>>()); acc
    });

    let mut clusters: Vec<BTreeMap<NaiveDate, Vec<String>>> = vec![];

    while peaks.iter().fold(0, |acc, (_, dates)| acc + dates.len() ) > 0 {
        let mut current_date = peaks.iter().fold( NaiveDate::from_ymd(2000, 1, 1), |acc, (_,v)| 
            if v.len() > 0 && acc < v[0] { v[0] } else { acc });

        let mut cluster: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();
        loop {
            cluster = peaks.iter_mut().fold(cluster, |mut acc, (&name, dates)| {
                if dates.len() > 0 {
                    if current_date - dates[0] <= Duration::days(8) {
                    // if dates[0] == current_date {
                        acc.entry(dates[0]).or_insert(vec![]).push(name.to_string()); 
                        dates.remove(0);
                    }
                }
                acc
            });

            let first_date = cluster.iter().fold( current_date, |acc , (&d, _)| std::cmp::min(acc, d) );
            if current_date == first_date {
                break;
            } else {
                current_date = first_date;
            }
        }

        if cluster.len() > 1 {

            let first_date = cluster.iter().fold( current_date, |acc , (&d, _)| std::cmp::min(acc, d) );
            let last_date = cluster.iter().fold( current_date, |acc , (&d, _)| std::cmp::max(acc, d) );

            clusters.push(cluster.clone());
            println!("cluster of {} days:", (last_date - first_date).num_days());
            for c in &cluster {
                let mut sorted = c.1.clone();
                sorted.sort();
                println!("{:?} => {:?}", c.0, sorted);
            }
            println!("");
            println!("");
        }
    }
}