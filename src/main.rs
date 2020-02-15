extern crate askama;
extern crate chrono;

use askama::Template;
use chrono::prelude::*;
use chrono::Duration;
use std::fs;
use std::io::prelude::*;

#[derive(Template)]
#[template(path = "date.html")]
struct DateTemplate<'a> {
    date: &'a String,
    closest: &'a String,
    slug: &'a String,
}

fn main() -> std::io::Result<()> {
    let start_year: i32 = 2020;
    let num_years: i32 = 10;
    let offsets = [(0.25, "quarter past"), (0.5, "half past")];

    let mut assignments = assign_shortcut_to_date(start_year, num_years, offsets);

    let extract_date: fn(&(Date<Utc>, String)) -> Date<Utc> = |e| e.0;

    assignments.sort_by_key(extract_date);

    let start_date = Utc.ymd(start_year, 1, 1);
    let max_date = Utc.ymd(start_year + num_years, 1, 1) - Duration::days(1);

    let closest_shortcut_for_dates =
        find_closest_shortcut(&mut assignments, start_date, max_date, extract_date);
    let sitemap_urls = render_pages(closest_shortcut_for_dates)?;

    generate_sitemap(sitemap_urls)?;

    Ok(())
}

fn assign_shortcut_to_date(
    start_year: i32,
    num_years: i32,
    offsets: [(f32, &str); 2],
) -> Vec<(Date<Utc>, String)> {
    let mut assignments = Vec::new();
    for year_offset in 0..num_years {
        let year = start_year + year_offset;
        let year_end = Utc.ymd(year + 1, 1, 1) - Duration::days(1);
        for month_num in 1..=12 {
            let last_day_of_month = if month_num == 12 {
                Utc.ymd(year + 1, 1, 1) - Duration::days(1)
            } else {
                Utc.ymd(year, month_num + 1, 1) - Duration::days(1)
            };
            let month_name = last_day_of_month.format("%B").to_string();
            let days_till_year_end = year_end - last_day_of_month;
            if days_till_year_end.num_seconds() > 0 {
                for (offset, offset_name) in offsets.iter() {
                    let days_offset =
                        (days_till_year_end.num_days() as f32 * offset).floor() as i64;
                    let date = last_day_of_month + Duration::days(days_offset);
                    let relative_date = format!("{} {}", offset_name, month_name);
                    println!(
                        "{} + ({} * {} = {}) = {}, {}",
                        last_day_of_month,
                        days_till_year_end,
                        offset,
                        days_offset,
                        date,
                        relative_date
                    );
                    assignments.push((date, relative_date));
                }
            }
        }
    }
    assignments
}

fn find_closest_shortcut(
    assignments: &Vec<(Date<Utc>, String)>,
    start_date: Date<Utc>,
    max_date: Date<Utc>,
    extract_date: fn(&(Date<Utc>, String)) -> Date<Utc>,
) -> Vec<(Date<Utc>, (Date<Utc>, String))> {
    let mut closest_shortcut_for_dates = Vec::new();
    for date in date_range(start_date, max_date) {
        let closest = match assignments.binary_search_by_key(&date, extract_date) {
            Ok(index) => assignments[index].clone(),
            Err(index) => {
                if index < assignments.len() {
                    assignments[index].clone()
                } else {
                    assignments[assignments.len() - 1].clone()
                }
            }
        };
        println!("{} -> {:?}", date, closest);
        closest_shortcut_for_dates.push((date, closest));
    }
    closest_shortcut_for_dates
}

fn render_pages(
    closest_shortcut_for_dates: Vec<(Date<Utc>, (Date<Utc>, String))>,
) -> std::io::Result<Vec<String>> {
    let mut sitemap_urls = Vec::new();
    for (date, closest) in closest_shortcut_for_dates.iter() {
        println!("{} -> {:?}", date, closest);
        let date_template = DateTemplate {
            date: &date.format("%Y-%m-%d").to_string(),
            closest: &closest.1,
            slug: &closest.1.to_lowercase().replace(" ", ""),
        };
        let rendered = date_template.render().unwrap();
        let dir_name = format!("public/{}", date.format("%Y-%m-%d"));
        fs::create_dir_all(&dir_name)?;
        let path = format!("{}/index.html", &dir_name);
        let mut file = fs::File::create(&path)?;
        file.write_all(rendered.as_bytes())?;

        sitemap_urls.push(format!(
            "https://quarterpastmarch.houseofmoran.io/{}/",
            date.format("%Y-%m-%d")
        ));
    }
    Ok(sitemap_urls)
}

fn generate_sitemap(sitemap_urls: Vec<String>) -> std::io::Result<()> {
    let mut sitemap_file = fs::File::create("public/sitemap.txt")?;
    for sitemap_url in sitemap_urls {
        sitemap_file.write_all(format!("{}\n", sitemap_url).as_bytes())?;
    }
    Ok(())
}

fn date_range(start_date: Date<Utc>, max_date: Date<Utc>) -> Vec<Date<Utc>> {
    let mut range = Vec::new();
    let mut current_date = start_date;
    while current_date <= max_date {
        range.push(current_date);
        current_date = current_date + Duration::days(1);
    }
    range
}
