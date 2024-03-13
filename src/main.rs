use reqwest::{Client, Error as ReqwestError};
use reqwest::ClientBuilder;
use reqwest::Body;
use reqwest::Version;
use serde_json::json;
use std::thread;
use std::time::Duration as StdDuration;
use std::fs::File;
use std::process::Command;
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

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
	clear_screen();
    // Welcome Header
    println!("Claim Voucher Tokopedia [Version 2.0.0]");
    println!("Native Version");
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
	validate_with_retry(&catalog_id, &cookie_content).await;
	redeem_with_retry(&catalog_id, &cookie_content).await;
	
}

async fn redeem_with_retry(catalog_id: &str, cookie_content: &str) {
    const MAX_RETRIES: usize = 7;
    let mut retries = 0;

    while retries < MAX_RETRIES {
        match redeem(catalog_id, cookie_content).await {
            Ok(_) => {
                println!("Redeem successful!");
                break;
            }
            Err(error) => {
                println!("Error redeeming: {}", error);
                retries += 1;
                println!("Retrying... Attempt {}/{}", retries, MAX_RETRIES);
                thread::sleep(StdDuration::from_secs_f64(0.5)); // Adjust the sleep duration as needed
            }
        }
    }
}

async fn validate_with_retry(catalog_id: &str, cookie_content: &str) {
    const MAX_RETRIES: usize = 3;
    let mut retries = 0;

    while retries < MAX_RETRIES {
        match validate(catalog_id, cookie_content).await {
            Ok(_) => {
                println!("Validation successful!");
                break;
            }
            Err(error) => {
                println!("Error validating: {}", error);
                retries += 1;
                println!("Retrying... Attempt {}/{}", retries, MAX_RETRIES);
                thread::sleep(StdDuration::from_secs_f64(0.5)); // Adjust the sleep duration as needed
            }
        }
    }
}

async fn redeem(catalog_id: &str, cookie_content: &str) -> Result<(), String> {

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
		
	let body_str = serde_json::to_string(&body_json).map_err(|e| format!("Serialization error: {}", e))?;
	let body = Body::from(body_str.clone());
	println!("{:?}", body);
    println!("\nsending Get Shopee request...");
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Origin", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com/rewards/kupon/"));
	headers.insert("Sec-Ch-Ua", reqwest::header::HeaderValue::from_static("\"Not A(Brand\";v=\"99\", \"Google Chrome\";v=\"122\", \"Chromium\";v=\"122\""));
	headers.insert("Sec-Ch-Ua-Mobile", reqwest::header::HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", reqwest::header::HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", reqwest::header::HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", reqwest::header::HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", reqwest::header::HeaderValue::from_static("same-site"));
	headers.insert("user-agent", reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
	headers.insert("X-Source", reqwest::header::HeaderValue::from_static("tokopedia-lite"));
	headers.insert("x-tkpd-akamai", reqwest::header::HeaderValue::from_static("claimcoupon"));
	headers.insert("X-Tkpd-Lite-Service", reqwest::header::HeaderValue::from_static("zeus"));
	headers.insert("X-Version", reqwest::header::HeaderValue::from_static("060a02e"));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content).unwrap());
	println!("Request Headers:\n{:?}", headers);
	
    let client = ClientBuilder::new()
        .gzip(true)
        .use_rustls_tls() // Use Rustls for HTTPS
        .build()
        .map_err(|e| format!("Failed to build reqwest client: {:?}", e))?;

    // Buat permintaan HTTP POST
    let result = client
        .post("https://gql.tokopedia.com/graphql/redeemCoupon")
        .header("Content-Type", "application/json")
        .headers(headers)
        .body(body)
        .version(Version::HTTP_2) 
        .send()
        .await;
    
    match result {
        Ok(response) => {
            println!("Redeem Status: {}", response.status());
            println!("Headers: {:#?}", response.headers());
            let body = response.text().await.map_err(|e| format!("Failed to read response body: {:?}", e))?;
            println!("Body: {}", body);
            Ok(())
        }
        Err(err) => Err(format!("Error: {:?}", err))
    }
}

async fn validate(catalog_id: &str, cookie_content: &str) -> Result<(), String> {

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
	
	let body_str = serde_json::to_string(&body_json).map_err(|e| format!("Serialization error: {}", e))?;
	let body = Body::from(body_str.clone());
	println!("{:?}", body);

    println!("\nsending Get Shopee request...");
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Origin", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com/rewards/kupon/"));
	headers.insert("Sec-Ch-Ua", reqwest::header::HeaderValue::from_static("\"Not A(Brand\";v=\"99\", \"Google Chrome\";v=\"122\", \"Chromium\";v=\"122\""));
	headers.insert("Sec-Ch-Ua-Mobile", reqwest::header::HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", reqwest::header::HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", reqwest::header::HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", reqwest::header::HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", reqwest::header::HeaderValue::from_static("same-site"));
	headers.insert("user-agent", reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"));
	headers.insert("X-Source", reqwest::header::HeaderValue::from_static("tokopedia-lite"));
	headers.insert("X-Tkpd-Lite-Service", reqwest::header::HeaderValue::from_static("zeus"));
	headers.insert("X-Version", reqwest::header::HeaderValue::from_static("060a02e"));
    headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content).unwrap());
	println!("Request Headers:\n{:?}", headers);
	
    let client = ClientBuilder::new()
        .gzip(true)
        .use_rustls_tls() // Use Rustls for HTTPS
        .build()
        .map_err(|e| format!("Failed to build reqwest client: {:?}", e))?;

	// Buat permintaan HTTP POST
	let result = client
		.post("https://gql.tokopedia.com/graphql/validateRedeem")
		.header("Content-Type", "application/json")
		.headers(headers)
		.body(body)
		.version(Version::HTTP_2) 
		.send()
		.await;
    match result {
        Ok(response) => {
            println!("Validation Status: {}", response.status());
            println!("Headers: {:#?}", response.headers());
            let body = response.text().await.unwrap();
            println!("Body: {}", body);
            Ok(())
        }
        Err(err) => Err(format!("Error: {:?}", err))
    }
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
    //let mut winver_executed = false; // Variabel penanda
	loop {
        let current_time = Local::now().naive_local();
        let task_time_naive = task_time_dt.time();
        let time_until_task = task_time_naive.signed_duration_since(current_time.time());

        if time_until_task <= Duration::seconds(0) {
            println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
			tugas_utama();
            break;
        }
        /*// Cek apakah tersisa 10 detik sebelum mencapai 0 detik
        if time_until_task <= Duration::seconds(10) && !winver_executed {
            println!("\nTask scheduled in 10 seconds! Current time: {}", current_time.format("%H:%M:%S.%3f"));
            Command::new("winver.exe").spawn().expect("Failed to execute command");
			winver_executed = true; // Setel variabel penanda menjadi true setelah dijadwalkan
        }*/
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
