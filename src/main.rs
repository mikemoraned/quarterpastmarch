extern crate chrono;

use chrono::prelude::*;
use chrono::Duration;

fn main() {
    let start_year: i32 = 2020;
    let num_years: i32 = 1;
    let offsets = [0.25, 0.5];
    let offset_names = ["quarter past", "half past"];
    let num_months: u32 = 12;

    let mut assignments = Vec::new();

    for year_offset in 0..num_years {
        let year = start_year + year_offset;
        let year_end = Utc.ymd(year + 1, 1, 1) - Duration::days(1);
        for month_num in 1..=num_months {
            let last_day_of_month = if month_num == 12 {
                Utc.ymd(year + 1, 1, 1) - Duration::days(1)
            } else {
                Utc.ymd(year, month_num + 1, 1) - Duration::days(1)
            };
            let month_name = last_day_of_month.format("%B").to_string();
            let days_till_year_end = year_end - last_day_of_month;
            if days_till_year_end.num_seconds() > 0 {
                for (offset, offset_name) in offsets.iter().zip(offset_names.iter()) {
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

    assignments.sort_by_key(|e| e.0);
    let start_date = Utc.ymd(start_year, 1, 1);
    let max_date = Utc.ymd(start_year + num_years, 1, 1) - Duration::days(1);
    let mut current_date = start_date;
    while current_date <= max_date {
        let closest = match assignments.binary_search_by_key(&current_date, |e| e.0) {
            Ok(index) => assignments[index].clone(),
            Err(index) => {
                if index < assignments.len() {
                    assignments[index].clone()
                } else {
                    assignments[assignments.len() - 1].clone()
                }
            }
        };
        println!("{} -> {:?}", current_date, closest);
        current_date = current_date + Duration::days(1);
    }
}
