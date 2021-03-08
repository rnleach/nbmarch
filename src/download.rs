use chrono::{Datelike, Timelike};

pub fn download_file(
    fname: &str,
    init_time: chrono::NaiveDateTime,
) -> Result<String, crate::Error> {
    let url = build_download_url(fname, init_time);

    Ok(reqwest::blocking::get(&url)?.text()?)
}

fn build_download_url(fname: &str, init_time: chrono::NaiveDateTime) -> String {
    // FIXME: when chrono makes these functions const, make this a const
    let nbm_4_starts: chrono::NaiveDateTime =
        chrono::NaiveDate::from_ymd(2020, 9, 23).and_hms(0, 0, 0);

    let year = init_time.year();
    let month = init_time.month();
    let day = init_time.day();
    let hour = init_time.hour();

    let url_fname = format_file_name_for_download(fname);

    if init_time > nbm_4_starts {
        format!(
            "{}{:04}/{:02}/{:02}/NBM4.0/{:02}/{}",
            BASE_URL, year, month, day, hour, url_fname
        )
    } else {
        format!(
            "{}{:4}/{:02}/{:02}/NBM/{:02}/{}",
            BASE_URL, year, month, day, hour, url_fname
        )
    }
}

fn format_file_name_for_download(fname: &str) -> String {
    fname.replace(" ", "%20")
}

const BASE_URL: &'static str = "https://hwp-viz.gsd.esrl.noaa.gov/wave1d/data/archive/";
