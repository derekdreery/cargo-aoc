use qu::ick_use::*;
use scraper::Html;

pub fn get_year(year: u16) -> Result<Vec<u8>> {
    let body = reqwest::blocking::get(format!("https://adventofcode.com/{}", year))?.text()?;
    let html = Html::parse_document(&body);
    println!("{:?}", html);
    todo!()
}
