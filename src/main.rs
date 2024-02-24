use cronet_rs::client::{Body, Client};
use http::HeaderValue;
use serde_json::json;
use std::thread;
use std::time::Duration as StdDuration;
use std::fs::File;
use std::io::{self, Read, Write};
use chrono::{Local, Duration, NaiveDateTime};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto Claim Tokopedia", about = "Make fast claim Voucher from tokopedia.com")]
struct Opt {
    #[structopt(short, long, help = "selected file cookie")]
    file: Option<String>,    
    #[structopt(short, long, help = "time to run")]
    time: Option<String>,    
    #[structopt(short, long, help = "Set catalog_id")]
    catalog: Option<String>, 
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn main() {
    let opt = Opt::from_args();
	clear_screen();
    // Welcome Header
    println!("Claim Voucher Tokopedia [Version 0.1.0]");
    println!("");

    // Get account details
    let selected_file = opt.file.clone().unwrap_or_else(|| select_cookie_file());
    let cookie_content = read_cookie_file(&selected_file);
	
    let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
		
    // Get target URL
    let catalog_id = opt.catalog.clone().unwrap_or_else(|| get_user_input("catalog_id: "));
	if let Ok(task_time_dt) = parse_task_time(&task_time_str) {
		countdown_to_task(&task_time_dt);
	} else {
		println!("Error parsing task time");
	}
	validate(&catalog_id, &cookie_content);
	redeem(&catalog_id, &cookie_content);
	
}
fn redeem(catalog_id: &str, cookie_content: &str){
	let client = Client::new();
	let body_json = json!([
	  {
		"operationName": "redeemCoupon",
		"variables": {
		  "catalog_id": catalog_id.parse::<i64>().unwrap(),
		  "is_gift": 0,
		  "gift_email": "",
		  "notes": ""
		},
		"query": "mutation redeemCoupon($catalog_id: Int, $is_gift: Int, $gift_user_id: Int, $gift_email: String, $notes: String) {\n  hachikoRedeem(catalog_id: $catalog_id, is_gift: $is_gift, gift_user_id: $gift_user_id, gift_email: $gift_email, notes: $notes, apiVersion: \"2.0.0\") {\n    coupons {\n      id\n      owner\n      promo_id\n      code\n      title\n      description\n      cta\n      cta_desktop\n      __typename\n    }\n    reward_points\n    redeemMessage\n    __typename\n  }\n}\n"
	  }
	]);
		
	let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());
	println!("{:?}", body);
    println!("\nsending Get Shopee request...");
	let mut headers = http::header::HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", HeaderValue::from_static("application/json"));
	headers.insert("Origin", HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Referer", HeaderValue::from_static("https://www.tokopedia.com/rewards/kupon/"));
	headers.insert("Sec-Ch-Ua", HeaderValue::from_static("\"Not A(Brand\";v=\"99\", \"Google Chrome\";v=\"121\", \"Chromium\";v=\"121\""));
	headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-site"));
	headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"));
	headers.insert("X-Source", HeaderValue::from_static("tokopedia-lite"));
	headers.insert("x-tkpd-akamai", HeaderValue::from_static("claimcoupon"));
	headers.insert("X-Tkpd-Lite-Service", HeaderValue::from_static("zeus"));
	headers.insert("X-Version", HeaderValue::from_static("808d646"));
    headers.insert("cookie", HeaderValue::from_str(&cookie_content).unwrap());
	println!("Request Headers:\n{:?}", headers);
	let mut request = http::Request::builder()
        .method("POST")
        .uri("https://gql.tokopedia.com/graphql/redeemCoupon");
		
    for (name, value) in headers.iter() {
        request = request.header(name.clone(), value.clone());
    }

    let request = request.body(body)
        .unwrap();

    let result = client.send(request);
    print_result(result);
}
fn validate(catalog_id: &str, cookie_content: &str){
	let client = Client::new();
	let body_json = json!([
	  {
		"operationName": "validateRedeem",
		"variables": {
		  "catalog_id": catalog_id.parse::<i64>().unwrap(),
		  "is_gift": 0,
		  "gift_email": ""
		},
		"query": "mutation validateRedeem($catalog_id: Int, $is_gift: Int, $gift_user_id: Int, $gift_email: String) {\n  hachikoValidateRedeem(catalog_id: $catalog_id, is_gift: $is_gift, gift_user_id: $gift_user_id, gift_email: $gift_email) {\n    is_valid\n    message_success\n    message_title\n    __typename\n  }\n}\n"
	  }
	]);
	
	let body_str = serde_json::to_string(&body_json).unwrap();
	let body = Body::from(body_str.clone());
	println!("{:?}", body);

    println!("\nsending Get Shopee request...");
	let mut headers = http::header::HeaderMap::new();
	headers.insert("Accept", HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", HeaderValue::from_static("application/json"));
	headers.insert("Origin", HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Referer", HeaderValue::from_static("https://www.tokopedia.com/rewards/kupon/"));
	headers.insert("Sec-Ch-Ua", HeaderValue::from_static("\"Not A(Brand\";v=\"99\", \"Google Chrome\";v=\"121\", \"Chromium\";v=\"121\""));
	headers.insert("Sec-Ch-Ua-Mobile", HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-site"));
	headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"));
	headers.insert("X-Source", HeaderValue::from_static("tokopedia-lite"));
	headers.insert("X-Tkpd-Lite-Service", HeaderValue::from_static("zeus"));
	headers.insert("X-Version", HeaderValue::from_static("808d646"));
    headers.insert("cookie", HeaderValue::from_str(&cookie_content).unwrap());
	println!("Request Headers:\n{:?}", headers);
	let mut request = http::Request::builder()
        .method("POST")
        .uri("https://gql.tokopedia.com/graphql/validateRedeem");
		
    for (name, value) in headers.iter() {
        request = request.header(name.clone(), value.clone());
    }

    let request = request.body(body)
        .unwrap();

    let result = client.send(request);
    print_result(result);
}
fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let milliseconds = duration.num_milliseconds() % 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}

fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
    match NaiveDateTime::parse_from_str(&format!("2023-01-01 {}", task_time_str), "%Y-%m-%d %H:%M:%S%.f") {
        Ok(dt) => Ok(dt),
        Err(e) => Err(e.into()),
    }
}

fn countdown_to_task(task_time_dt: &NaiveDateTime) {
    loop {
        let current_time = Local::now().naive_local();
        let task_time_naive = task_time_dt.time();
        let time_until_task = task_time_naive.signed_duration_since(current_time.time());

        if time_until_task < Duration::zero() {
            println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
            tugas_utama();
            break;
        }

        let formatted_time = format_duration(time_until_task);
        print!("\r{}", formatted_time);
        io::stdout().flush().unwrap();

        thread::sleep(StdDuration::from_secs_f64(0.001));
    }
}

fn tugas_utama() {
    println!("Performing the task...");
    println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
}

fn print_result(result: Result<http::Response<Body>, cronet_rs::client::ClientError>) {
    match result {
        Ok(response) => {
            println!("Status: {}", response.status());
            println!("Headers: {:#?}", response.headers());
            let body = response.body().as_bytes().unwrap();
            println!("Body: {}", String::from_utf8_lossy(body));
        }
        Err(error) => println!("Error: {}", error),
    }
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
	
fn select_cookie_file() -> String {
    println!("Daftar file cookie yang tersedia:");
    let files = std::fs::read_dir("./akun");
    let mut file_options = Vec::new();
    for (index, file) in files.expect("REASON").enumerate() {
        if let Ok(file) = file {
            let file_name = file.file_name();
            println!("{}. {}", index + 1, file_name.to_string_lossy());
            file_options.push(file_name.to_string_lossy().to_string());
        }
    }

    let selected_file = loop {
        let input = get_user_input("Pilih nomor file cookie yang ingin digunakan: ");

        if let Ok(index) = input.trim().parse::<usize>() {
            if index > 0 && index <= file_options.len() {
                break file_options[index - 1].clone();
            }
        }
    };

    selected_file
}
fn read_cookie_file(file_name: &str) -> String {
    let file_path = format!("./akun/{}", file_name);
    let mut cookie_content = String::new();
    File::open(&file_path).expect("REASON").read_to_string(&mut cookie_content);
    cookie_content
}
