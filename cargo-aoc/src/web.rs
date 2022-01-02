use qu::ick_use::*;
use reqwest::blocking::Client;

thread_local! {
    pub static CLIENT: Client = Client::new();
}

pub fn get_day(cookie: &str, year: u16, day: u8) -> Result<String> {
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    log::info!("fetching {}", url);

    let res = CLIENT
        .with(|client| client.get(url))
        .header("Cookie", format!("session={}", cookie))
        .send()?;
    let res = res.error_for_status()?;
    let res = res.text()?;
    Ok(res)
}

//Cookie: session=.....
