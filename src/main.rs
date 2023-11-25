use reqwest;
use serde_json::Value;
use serde_yaml;
use rusqlite;
use clap::{Parser, Error};
use std::fs;
use std::io::Read;
use std::env;

const DEFAULT_TOKEN_PATH:&str = "~/.remo/token.yml";

const REMO_DOMAIN:&str = "api.nature.global";
const REMO_PROTOCOL:&str = "https";
const REMO_VERSION:&str = "1";
#[allow(dead_code)]
const REMO_API_APPLIANCES:&str = "appliance_orders";
const REMO_API_DEVICES:&str = "devices";

const WEATHERAPI_DOMAIN:&str = "api.weatherapi.com";
const WEATHERAPI_PROTOCOL:&str = "https";
const WEATHERAPI_VERSION:&str = "v1";
const WEATHERAPI_API:&str = "current.json";
const WEATHERAPI_CITY:&str = "Himeji";
const WEATHERAPI_AQI:&str = "no";


#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[arg(short = 'd', long, help = "sqlite database file path")]
    db_path: String,
    #[arg(short = 't', long, help = "remo api token file(YAML)",default_value = DEFAULT_TOKEN_PATH)]
    token_path: Option<String>,
}

fn open_db(sqlite_file_path: &String) -> Result<rusqlite::Connection, rusqlite::Error> {
    let con = rusqlite::Connection::open(sqlite_file_path)?;
    //println!("{}",con.is_autocommit());
    Ok(con)
}
#[derive(Debug)]
struct Token {
    remo: String,
    weatherapi: String,
}

fn get_token(token_path:&String)-> Result<Token,Error>{
    let mut path = token_path.clone();
    
    if path.starts_with("~/") {
        path = path.replace("~",&env::var("HOME").unwrap());
    }
    let mut file = fs::File::open(path).unwrap();
    let mut y = String::new();
    let _l = file.read_to_string(&mut y).unwrap();
    let yaml:Value = serde_yaml::from_str(&y).unwrap();
    let token = Token{
        remo: yaml.as_object().unwrap()["remo"].as_str().unwrap().to_string(),
        weatherapi: yaml.as_object().unwrap()["weatherapi"].as_str().unwrap().to_string(),
    };
    Ok(token)
}

#[derive(Debug)]
struct RoomTemp{
    temp: f64,
    measured: String,
}

fn get_room_temp(token:String) -> Result<RoomTemp,Error>{
    let url = format!("{}://{}/{}/{}",REMO_PROTOCOL,REMO_DOMAIN,REMO_VERSION,REMO_API_DEVICES);
    let auth = format!("Bearer {}",token);
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).
        header("accept","application/json").
        header("Authorization", auth).send();

    let r = match resp {
        Ok(resp) => resp.text().unwrap(),
        Err(err) => panic!("Error: {}",err)
    };
    
    let json:Value = serde_json::from_str(&r).unwrap();
    
    let devices = json.as_array().unwrap();
    let mut temp:f64 = 0.0;
    let mut measured = "".to_string();
    for d in devices{
        let o = d.as_object().unwrap();
        temp = o["newest_events"]["te"]["val"].as_f64().unwrap();
        measured = o["newest_events"]["te"]["created_at"].as_str().unwrap().to_string().clone();
        break;
    }
    Ok(
        RoomTemp{
            temp: temp,
            measured: measured,
        }
    )
}

#[derive(Debug)]
struct Weather{
    temp: f64,
    measured: String,
}

fn get_weather(token:String) -> Result<Weather,Error>{
    let url = format!("{}://{}/{}/{}?key={}&q={}&api={}",WEATHERAPI_PROTOCOL,WEATHERAPI_DOMAIN,WEATHERAPI_VERSION,WEATHERAPI_API,token,WEATHERAPI_CITY,WEATHERAPI_AQI);

    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).
        header("accept","application/json").send();

    let r = match resp {
        Ok(resp) => resp.text().unwrap(),
        Err(err) => panic!("Error: {}",err)
    };
    
    let json:Value = serde_json::from_str(&r).unwrap();
    let weather = json.as_object().unwrap();
    Ok(
        Weather { 
            temp: weather["current"]["temp_c"].as_f64().unwrap(),
            measured:weather["current"]["last_updated"].as_str().unwrap().to_string().clone()
        }
    )
}

fn main() {
    let args = Args::parse();
    let db_path = args.db_path;
    let mut token_path = args.token_path.unwrap();
    let token = get_token(&mut token_path).unwrap();
    let room  = get_room_temp(token.remo).unwrap();
    let weather = get_weather(token.weatherapi).unwrap();
    let conn = open_db(&db_path).unwrap();
    let mut statement = conn.prepare("insert into temp (stored,room_temp,room_measured,weather_temp,weather_measured) values (datetime('now','localtime'),?,?,?,?)").unwrap();
    let mut rows = statement.query(rusqlite::params![room.temp,room.measured,weather.temp,weather.measured]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        println!("{:?}",row);
    }
}

