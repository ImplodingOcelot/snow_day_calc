use json::JsonValue;
use reqwest::{self};
use core::time;
use std::{i128::MAX, process, vec};
use tokio;

#[tokio::main]
async fn main() {
    println!("Enter zip code: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input: i32 = input.trim().parse().unwrap();
    let result = get_zip_code_areas(input).await;
    if result.is_err() {
        println!("An error occurred: {}", result.err().unwrap());
        process::exit(1);
    }
    let mut areas = result.unwrap();
    areas.remove(0);
    println!("Select area: ");
    let mut i = 1;
    for area in &areas {
        println!("Area: {:?}: {:?}",i, area[0]);
        i+=1;
    }
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let mut input: usize = input.trim().parse().unwrap();
    input -= 1;
    println!("You selected: {:?}", input);
    println!("Area: {:?}", areas[input][0]);
    let long = areas[input][2].parse::<f64>().unwrap();
    let lat = areas[input][1].parse::<f64>().unwrap();
    println!("Latitude: {:?}", lat);
    println!("Longitude: {:?}", long);
    let main_prediction = get_weather_prediction(areas[input][1].parse().unwrap(), areas[input][2].parse().unwrap()).await;
    if main_prediction.is_err() {
        println!("An error occurred: {}", main_prediction.err().unwrap());
        process::exit(1);
    }
    let jsonval = main_prediction.as_ref().unwrap();
    let mut snowdaypoints = 0;
    let mut temp = 0f64;
    let mut temp2;

    // add all snowfall values to snowdaypoints
    for i in 0..48 {
        let temp_2m = getparem(jsonval.clone(), "snowfall".to_owned(), i);
        temp += temp_2m;
    }
    if temp > 0f64 {
        println!("Added 3 points for snowfall initial");
        snowdaypoints += 3;
    }
    println!("Added {:?} points for snowfall", temp as i32);
    snowdaypoints += (temp as i32)/2;
    temp = 0f64;

    // add average temp of each hour
    temp2 = 1000f64;
    for i in 0..48 {
        let temp_2m = getparem(jsonval.clone(), "temperature_2m".to_owned(), i);
        if temp_2m < temp2 {
            temp2 = temp_2m;
        }
        temp += temp_2m;
    }
    temp /= 48f64;
    if temp < 0f64 {
        println!("Added 2 points for average temp initial");
        snowdaypoints += 2;
    }
    if temp2 < -8f64 {
        println!("Added 7 points for lowest temp initial and {:?} points for lowest temp", (temp2 as i32).abs());
        snowdaypoints += 7 + (temp2 as i32).abs();
    }
    for i in 0..48 {
        let temp_2m = getparem(jsonval.clone(), "visibility".to_owned(), i);
        if temp_2m < 1000f64 {
            println!("Added 1 point for low visibility");
            snowdaypoints += 1;
            if temp_2m < 100f64 {
                println!("Added 5 points for very low visibility");
                snowdaypoints += 5;
                if temp_2m < 10f64 {
                    println!("Added 10 points for extremely low visibility");
                    snowdaypoints += 10;
                }
            }
        }
    }
    temp = 0f64;
    temp2 = 0f64;
    for i in 0..48  {
        let temp_2m = getparem(jsonval.clone(), "wind_speed_10m".to_owned(), i);
        if temp < temp_2m {
            temp = temp_2m;
        }
        temp2 += temp_2m;
    }
    if temp2 > 20f64 {
        println!("Added {} points for high average wind speed", (temp2 as i32)/200);
        snowdaypoints += (temp2 as i32)/200;
    }
    if temp > 20f64 {println!("Added 5 points for high wind speed");
        snowdaypoints += 5;
    }
    for i in 0..48 {
        if getparem(jsonval.clone(), "precipitation_probability".to_owned(), i) > 50f64 {
            println!("Added 1 point for high precipitation probability");
            snowdaypoints += 1;
            break;
        }
    }
    temp = 10000f64;
    for i in 0..48 {
        if getparem(jsonval.clone(), "apparent_temperature".to_owned(), i) < temp{
            temp = getparem(jsonval.clone(), "apparent_temperature".to_owned(), i);
        }
    }
    if temp < 10f64 {
        println!("Added 2 points for low apparent temperature");
        snowdaypoints += 2;
    }
    snowdaypoints -= 15; // to  account for normal weather
    if snowdaypoints < 1 {
        snowdaypoints = 1;
    }
    if snowdaypoints > 99 {
        snowdaypoints = 99;
    }
    println!("{:?}% chance of snow day!", snowdaypoints);

}

async fn get_zip_code_areas(zipcode: i32) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut areas: Vec<Vec<String>> = vec![vec![]];
    let url = format!(
        "http://api.geonames.org/postalCodeSearch?postalcode={}&maxRows=10&username=aaaa",
        zipcode
    );
    let mut response = reqwest::get(url.clone())
        .await?
        .text()
        .await?;
    // this is here because the api breaks sometimes and i dont know why so ill keep pinging until it works lmfao
    while response == "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n<geonames>\n\t<status message=\"net/geonames/lucene/PostalCodeSearchResult\" value=\"12\"/>\n</geonames>"   {
    response = reqwest::get(url.clone())
        .await?
        .text()
        .await?;
    }
    // xml is a pain to parse so we are just going to do it the dumb way
    let mut start = 0;
    let mut end;
    while start < response.len() {
        let mut area = vec![];
        start = response.find("<name>").unwrap();
        start += 6;
        end = response.find("</name>").unwrap();
        area.push(response[start..end].to_string());
        start = response.find("<lat>").unwrap();
        start += 5;
        end = response.find("</lat>").unwrap();
        area.push(response[start..end].to_string());
        start = response.find("<lng>").unwrap();
        start += 5;
        end = response.find("</lng>").unwrap();
        area.push(response[start..end].to_string());
        response = response[end + 5..].to_string();
        start = end;
        // println!("pushing: {:?}", area);
        areas.push(area);
    }
    return Ok(areas);
}

async fn get_weather_prediction(lat: f64, lng: f64) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,snowfall,visibility,wind_speed_10m,&forecast_days=2",
        lat, lng
    );
    /*
    Parematers:
    hourly_temp
    relative_humidity
    dew_point
    apparent_temperature
    precipitation_probability
    precipitation
    snowfall
    visibility
    wind_speed
    */
    println!("URL: {:?}", url);
    let response = reqwest::get(url)
        .await?
        .text()
        .await?;
    // println!("AAA: {:?}" , response);
    let parsed = json::parse(&response).unwrap();
    // println!("parsed: {:?}", parsed["hourly"]["temperature_2m"][0].as_f32().unwrap());
    return Ok(parsed);
}

fn getparem(jsonval: JsonValue, parem: String, hour: usize) -> f64 {
    return jsonval["hourly"][parem][hour].as_f64().unwrap();
}