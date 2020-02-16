extern crate askama;
extern crate async_std;
extern crate chrono;
extern crate itertools;

use askama::Template;
use async_std::fs;
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use chrono::prelude::*;
use chrono::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

#[derive(Template)]
#[template(path = "date.html")]
struct DateTemplate<'a> {
    date: &'a String,
    closest: &'a String,
    slug: &'a String,
}

fn main() -> io::Result<()> {
    let start_year: i32 = 2020;
    let num_years: i32 = 10;
    let offsets = [(0.25, "quarter past"), (0.5, "half past")];

    let assignments = assign_shortcut_to_date(start_year, num_years, offsets);

    let start_date = Utc.ymd(start_year, 1, 1);
    let max_date = Utc.ymd(start_year + num_years, 1, 1) - Duration::days(1);

    let closest_shortcut_for_dates =
        find_closest_shortcut_in_each_year(&assignments, start_date, max_date);

    task::block_on(async {
        let sitemap_urls = render_pages(closest_shortcut_for_dates).await?;

        generate_sitemap(sitemap_urls).await?;

        Ok(())
    })
}

type ShortcutAssignment = (Date<Utc>, String);

fn assign_shortcut_to_date(
    start_year: i32,
    num_years: i32,
    offsets: [(f32, &str); 2],
) -> Vec<ShortcutAssignment> {
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
                    assignments.push((date, relative_date));
                }
            }
        }
    }
    assignments.sort_by_key(|a| a.0);
    assignments
}

fn find_closest_shortcut_in_each_year(
    assignments: &[ShortcutAssignment],
    start_date: Date<Utc>,
    max_date: Date<Utc>,
) -> Vec<(Date<Utc>, ShortcutAssignment)> {
    let mut closest_shortcut_for_dates = Vec::new();
    let pb = ProgressBar::new((max_date - start_date).num_days() as u64 + 1);
    pb.set_style(default_progress_style());
    pb.set_message("find closest");
    for (year, dates) in date_range(start_date, max_date)
        .into_iter()
        .group_by(|d| d.year())
        .into_iter()
    {
        let year_assignments = assignments
            .iter()
            .filter(|a| a.0.year() == year)
            .collect::<Vec<&ShortcutAssignment>>();
        find_closest_shortcut(
            &year_assignments,
            &dates.collect::<Vec<Date<Utc>>>(),
            &mut closest_shortcut_for_dates,
        );
        pb.set_position(closest_shortcut_for_dates.len() as u64);
    }
    pb.finish();
    closest_shortcut_for_dates
}

fn find_closest_shortcut(
    assignments: &[&ShortcutAssignment],
    dates: &[Date<Utc>],
    closest_shortcut_for_dates: &mut Vec<(Date<Utc>, ShortcutAssignment)>,
) {
    for date in dates {
        let closest = match assignments.binary_search_by_key(date, |a| a.0) {
            Ok(index) => assignments[index].clone(),
            Err(index) => {
                if index < assignments.len() {
                    assignments[index].clone()
                } else {
                    assignments[assignments.len() - 1].clone()
                }
            }
        };
        closest_shortcut_for_dates.push((*date, closest));
    }
}

async fn render_pages(
    closest_shortcut_for_dates: Vec<(Date<Utc>, ShortcutAssignment)>,
) -> std::io::Result<Vec<String>> {
    let mut sitemap_urls = Vec::new();
    let mut spawned = Vec::new();
    let pb = ProgressBar::new(closest_shortcut_for_dates.len() as u64);
    pb.set_style(default_progress_style());
    pb.set_message("render and save");
    for (date, closest) in closest_shortcut_for_dates.iter() {
        spawned.push(task::spawn(render_page(*date, closest.clone())));
        sitemap_urls.push(format!(
            "https://quarterpastmarch.houseofmoran.io/{}/",
            date.format("%Y-%m-%d")
        ));
    }
    for s in spawned {
        s.await?;
        pb.inc(1);
    }
    pb.finish();
    Ok(sitemap_urls)
}

async fn render_page(date: Date<Utc>, closest: ShortcutAssignment) -> std::io::Result<()> {
    let date_template = DateTemplate {
        date: &date.format("%Y-%m-%d").to_string(),
        closest: &closest.1,
        slug: &closest.1.to_lowercase().replace(" ", ""),
    };
    let rendered = date_template.render().unwrap();
    let dir_name = format!("public/{}", date.format("%Y-%m-%d"));
    fs::create_dir_all(&dir_name).await?;
    let path = format!("{}/index.html", &dir_name);
    let mut file = fs::File::create(&path).await?;
    file.write_all(rendered.as_bytes()).await?;
    Ok(())
}

async fn generate_sitemap(sitemap_urls: Vec<String>) -> std::io::Result<()> {
    let mut sitemap_file = fs::File::create("public/sitemap.txt").await?;
    for sitemap_url in sitemap_urls {
        sitemap_file
            .write_all(format!("{}\n", sitemap_url).as_bytes())
            .await?;
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

fn default_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
}
