use reqwest;
use serde_json::Value;
use serde_yaml;
use rusqlite;
use clap::{Parser, Error};
use std::fs;
use std::io::Read;
use std::env;

const DOMAIN:&str = "api.nature.global";
const PROTOCOL:&str = "https";
const VERSION:&str = "1";
#[allow(dead_code)]
const API_APPLIANCES:&str = "appliance_orders";
const API_DEVICES:&str = "devices";
const TOKEN_PATH:&str = ".remo/token.yaml";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[arg(short = 'd', long, help = "sqlite database file path")]
    db_path: String,
    #[arg(short = 't', long, help = "remo api token file(YAML)")]
    token_path: Option<String>,
}

fn open_db(sqlite_file_path: &String) -> Result<rusqlite::Connection, rusqlite::Error> {
    let con = rusqlite::Connection::open(sqlite_file_path)?;
    //println!("{}",con.is_autocommit());
    Ok(con)
}

fn get_token(token_path:&String) -> Result<String,Error>{
    let mut path = token_path.clone();
    if path == ""{
        let home = env::var("HOME").unwrap();
        path = format!("{}/{}",home,TOKEN_PATH);
    }
    let mut file = fs::File::open(path).unwrap();
    let mut y = String::new();
    let _l = file.read_to_string(&mut y).unwrap();
    let yaml:Value = serde_yaml::from_str(&y).unwrap();
    let token =  yaml.as_object().unwrap()["token"].as_array().unwrap()[0].as_str().unwrap();
    Ok(token.to_string())
}

fn main() {
    let args = Args::parse();
    let db_path = args.db_path;
    let url = format!("{}://{}/{}/{}",PROTOCOL,DOMAIN,VERSION,API_DEVICES);
    let token_path = match args.token_path {
        Some(s) => s,
        None => "".to_string()
    };
    let auth = format!("Bearer {}",get_token(&token_path).unwrap());

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
    let mut create_at = "".to_string();
    for d in devices{
        let o = d.as_object().unwrap();
        temp = o["newest_events"]["te"]["val"].as_f64().unwrap();
        create_at = o["newest_events"]["te"]["created_at"].as_str().unwrap().to_string().clone();
        break;
    }
    let conn = open_db(&db_path).unwrap();
    let mut statement = conn.prepare("insert into temp (temp,created_at) values (?,?)").unwrap();
    let mut rows = statement.query(rusqlite::params![temp,create_at]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        println!("{:?}",row);
    }
}

