extern crate chrono;

use chrono::prelude::*;
use chrono::Duration;

fn main() {
    let start_year: i32 = 2020;
    let num_years: i32 = 1;
    let offsets = [0.25, 0.5];
    let offset_names = ["quarter past", "half past"];
    let num_months: u32 = 12;

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
                    println!(
                        "{} + ({} * {} = {}) = {}, {} {}",
                        last_day_of_month,
                        days_till_year_end,
                        offset,
                        days_offset,
                        date,
                        offset_name,
                        month_name
                    );
                }
            }
        }
    }
}
