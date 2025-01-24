/*
Whats new In 2.2.2 :
Add multiple thread for redeem request
Whats new In 2.2.1 :
Add Slug mode
Whats new In 2.2.0 :
Add max 3 retry for all requests
Fix if coupon succesfully redeem
*/

use rquest as reqwest;
use reqwest::tls::Impersonate;
use reqwest::{ClientBuilder, header::HeaderMap, Version};
use reqwest::header::HeaderValue;
use serde_json::{Value};
use std::fs::File;
use std::io::{self, Read, Write};
use chrono::{Local, Duration, NaiveDateTime};
use structopt::StructOpt;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Serialize)]
struct SlugRequest {
    operation_name: String,
    variables: SlugVariables,
    query: String,
}
#[derive(Serialize)]
struct SlugVariables {
    slug: String,
}
#[derive(Clone, Serialize)]
struct RedeemCouponVariables {
    catalog_id: i64,
    is_gift: i32,
    gift_email: String,
    notes: String,
}

#[derive(Clone, Serialize)]
struct RedeemCouponRequest {
    operation_name: String,
    variables: RedeemCouponVariables,
    query: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Auto Claim Tokopedia", about = "Make fast claim Voucher from tokopedia.com")]
struct Opt {
	#[structopt(short, long, help = "selected file cookie")]
	file: Option<String>,	
	#[structopt(short, long, help = "time to run")]
	time: Option<String>,	
	#[structopt(short, long, help = "Set catalog_id")]
	catalog: Option<String>,
    #[structopt(short, long, help = "select modes")]
	mode: Option<String>,
}

enum Mode {
	Fast,
	FastSlug,
	Normal,
}

fn clear_screen() {
	print!("\x1B[2J\x1B[1;1H");
	io::stdout().flush().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let opt = Opt::from_args();
	let version_info = env!("CARGO_PKG_VERSION");
	clear_screen();
	// Welcome Header
	println!("{}", format!("Claim Voucher Tokopedia [Version {}]", version_info));
	println!("Native Version");
	println!("");
	let mode = select_mode(&opt);

	// Get account details
	let selected_file = opt.file.clone().unwrap_or_else(|| select_cookie_file());
	let cookie_content = read_cookie_file(&selected_file);
	
	let task_time_str = opt.time.clone().unwrap_or_else(|| get_user_input("Enter task time (HH:MM:SS.NNNNNNNNN): "));
	let mut slug = "GC10DECA";
		
	match mode {
		Mode::FastSlug => {
			let slug_value = opt.catalog.clone().unwrap_or_else(|| get_user_input("slug: "));
			slug = &slug_value; 
			let catalog_id_itr = get_catalog_id(&slug, &cookie_content).await?;
			let task_time_dt = parse_task_time(&task_time_str)?;
			countdown_to_task(task_time_dt).await;
			
			redeem_builder(&slug, catalog_id_itr, &cookie_content).await?;
		}
		Mode::Normal => {
			let catalog_id = opt.catalog.clone().unwrap_or_else(|| get_user_input("catalog_id: "));
			let catalog_id_itr = catalog_id.parse::<i64>()?; 
			let task_time_dt = parse_task_time(&task_time_str)?;
			countdown_to_task(task_time_dt).await;

			validate(catalog_id_itr, &cookie_content).await?;
			redeem_builder(&slug, catalog_id_itr, &cookie_content).await?;
		}
		Mode::Fast => {
			let catalog_id = opt.catalog.clone().unwrap_or_else(|| get_user_input("catalog_id: "));
			let catalog_id_itr = catalog_id.parse::<i64>()?; 
			let task_time_dt = parse_task_time(&task_time_str)?;
			countdown_to_task(task_time_dt).await;
			
			redeem_builder(&slug, catalog_id_itr, &cookie_content).await?;
		}
	}
	println!("\nTask completed! Current time: {}", Local::now().format("%H:%M:%S.%3f"));
	Ok(())	
}

async fn validate(catalog_id: i64, cookie_content: &str) -> Result<(), Box<dyn std::error::Error>> {
	let body_json = vec![RedeemCouponRequest {
        operation_name: "validateRedeem".to_string(),
        variables: RedeemCouponVariables {
            catalog_id,
            is_gift: 0,
            gift_email: "".to_string(),
            notes: "".to_string(),
        },
        query: "mutation validateRedeem($catalog_id: Int, $is_gift: Int, $gift_user_id: Int, $gift_email: String) {\n  hachikoValidateRedeem(catalog_id: $catalog_id, is_gift: $is_gift, gift_user_id: $gift_user_id, gift_email: $gift_email) {\n	is_valid\n	message_success\n	message_title\n	__typename\n  }\n}\n".to_string(),
    }];
	println!("\nsending Get TKPD request...");
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("Connection", HeaderValue::from_static("keep-alive"));
	headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Origin", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Priority", reqwest::header::HeaderValue::from_static("u=1, i"));
	headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com/rewards/kupon/detail/GC25OCTA"));
	headers.insert("Sec-Ch-Ua", reqwest::header::HeaderValue::from_static("\"Not A(Brand\";v=\"8\", \"Google Chrome\";v=\"129\", \"Chromium\";v=\"129\""));
	headers.insert("Sec-Ch-Ua-Mobile", reqwest::header::HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", reqwest::header::HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", reqwest::header::HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", reqwest::header::HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", reqwest::header::HeaderValue::from_static("same-site"));
	headers.insert("user-agent", reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"));
	headers.insert("X-Source", reqwest::header::HeaderValue::from_static("tokopedia-lite"));
	headers.insert("X-Tkpd-Lite-Service", reqwest::header::HeaderValue::from_static("zeus"));
	headers.insert("X-Version", reqwest::header::HeaderValue::from_static("2e4ea1e"));
	headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content).unwrap());
	//println!("Request Headers:\n{:?}", headers);
	loop{
		let client = ClientBuilder::new()
			.danger_accept_invalid_certs(true)
			.impersonate_without_headers(Impersonate::Chrome130)
			.enable_ech_grease(true)
			.permute_extensions(true)
			.gzip(true)
			.build()?;

		// Buat permintaan HTTP POST
		let response = client
			.post("https://gql.tokopedia.com/graphql/validateRedeem")
			.headers(headers.clone())
			.json(&body_json)
			.version(Version::HTTP_2) 
			.send()
			.await?;

		let status = response.status();
		println!("Validation Status: {}", status);
		//println!("Headers: {:#?}", response.headers());
		if status == reqwest::StatusCode::OK {
			let body: Value = response.json().await?;
			println!("Body: {}", body);
			break;
		}else{
			continue;
		}
	}
	Ok(())
}

async fn redeem_builder(slug: &str, catalog_id: i64, cookie_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let headers = header_redeem(&slug, &cookie_content).await;
	let body_json = vec![RedeemCouponRequest {
        operation_name: "redeemCoupon".to_string(),
        variables: RedeemCouponVariables {
            catalog_id,
            is_gift: 0,
            gift_email: "".to_string(),
            notes: "".to_string(),
        },
        query: "mutation redeemCoupon($catalog_id: Int, $is_gift: Int, $gift_user_id: Int, $gift_email: String, $notes: String) {\n  hachikoRedeem(catalog_id: $catalog_id, is_gift: $is_gift, gift_user_id: $gift_user_id, gift_email: $gift_email, notes: $notes, apiVersion: \"2.0.0\") {\n	coupons {\n	  id\n	  owner\n	  promo_id\n	  code\n	  title\n	  description\n	  cta\n	  cta_desktop\n	  __typename\n	}\n	reward_points\n	redeemMessage\n	__typename\n  }\n}\n".to_string(),
    }];
    let max_threads = if num_cpus::get() > 4 { 
        num_cpus::get() 
    } else {
        4 
    }; 
	let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(max_threads);
	let stop_flag = Arc::new(AtomicBool::new(false));
    for i in 0..max_threads {
        let headers = headers.clone();
        let body_json = body_json.clone();
        let tx = tx.clone();
        let stop_flag = stop_flag.clone();

        tokio::spawn(async move {
            let client = ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .gzip(true)
                .build()
                .expect("Failed to build HTTP client");

            let mut attempt = 0;
            let max_attempts = 3;

            while !stop_flag.load(Ordering::Relaxed) {
                if attempt >= max_attempts {
                    eprintln!("Thread {}: Mencapai batas maksimum {} percobaan.", i, max_attempts);
                    break;
                }
                attempt += 1;
                println!("Thread {}: Percobaan ke-{}", i, attempt);

                // Kirim request
                let response = match client
                    .post("https://gql.tokopedia.com/graphql/redeemCoupon")
                    .headers(headers.clone())
                    .json(&body_json)
                    .version(Version::HTTP_2)
                    .send()
                    .await
                {
                    Ok(res) => res,
                    Err(err) => {
                        eprintln!("Thread {}: Gagal mengirim request: {:?}", i, err);
                        continue;
                    }
                };

                let status = response.status();
                let json_response: Value = match response.json().await {
                    Ok(json) => json,
                    Err(err) => {
                        eprintln!("Thread {}: Gagal membaca JSON: {:?}", i, err);
                        continue;
                    }
                };

                println!(
                    "[{}][Thread {}] Redeem Status: {}",
                    Local::now().format("%H:%M:%S.%3f"),
                    i,
                    status
                );

                if status == reqwest::StatusCode::OK {
                    if let Some(redeem_message) = json_response
                        .pointer("/data/hachikoRedeem/redeemMessage")
                        .and_then(|val| val.as_str())
                    {
                        if redeem_message == "Kupon berhasil diklaim" {
                            println!("[Thread {}] Coupon successfully claimed!", i);
                            let _ = tx.send("Coupon successfully claimed!".to_string()).await;
                            stop_flag.store(true, Ordering::Relaxed);
                            break;
                        } else {
                            println!(
                                "[Thread {}] Unexpected redeem message: {}",
                                i, redeem_message
                            );
                        }
                    } else {
                        println!(
                            "[Thread {}] Redeem message not found in response: {:?}",
                            i, json_response
                        );
                    }
                }
            }
        });
    }
    drop(tx); // Tutup pengirim setelah semua tugas selesai
    while let Some(message) = rx.recv().await {
        println!("{}", message);
    }
	Ok(())
}

async fn get_catalog_id(slug: &str, cookie_content: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let headers = header_redeem(&slug, &cookie_content).await;
	let body_json = vec![SlugRequest {
        operation_name: "CatalogDetailQuery".to_string(),
        variables: SlugVariables {
			slug: slug.to_string(),
        },
        query: "query CatalogDetailQuery($slug: String) {\n  tokopoints {\n    status {\n      points {\n        reward\n        __typename\n      }\n      __typename\n    }\n    __typename\n  }\n  hachikoCatalogDetail(slug: $slug, apiVersion: \"3.0.0\") {\n    id\n    title\n    expired_label\n    expired_str\n    is_disabled\n    is_disabled_button\n    disable_error_message\n    upper_text_desc\n    how_to_use\n    tnc\n    icon\n    quotaPercentage\n    is_gift\n    points_str\n    button_str\n    points_slash_str\n    discount_percentage_str\n    minimumUsageLabel\n    minimumUsage\n    product_recommendation {\n      is_show\n      param\n      __typename\n    }\n    activePeriod\n    activePeriodDate\n    globalPromoCodes {\n      title\n      code\n      dynamicInfos\n      __typename\n    }\n    actionCTA {\n      text\n      url\n      type\n      isShown\n      isDisabled\n      __typename\n    }\n    catalog_type\n    __typename\n  }\n}\n".to_string(),
    }];

	loop{
		let client = ClientBuilder::new()
			.danger_accept_invalid_certs(true)
			.impersonate_without_headers(Impersonate::Chrome130)
			.enable_ech_grease(true)
			.permute_extensions(true)
			.gzip(true)
			.build()?;

		// Buat permintaan HTTP POST
		let response = client
			.post("https://gql.tokopedia.com/graphql/CatalogDetailQuery")
			.headers(headers.clone())
			.json(&body_json)
			.version(Version::HTTP_2) 
			.send()
			.await?;

		let status = response.status();
		println!("[{}]Redeem Status: {}", Local::now().format("%H:%M:%S.%3f"), response.status());
		let json_response: Value = response.json().await?;
		if status == reqwest::StatusCode::OK {
			println!("Body: {}", json_response);
			// Parse the response body as an array
			// Extract the ID
			if let Some(id) = json_response[0]["data"]["hachikoCatalogDetail"]["id"].as_i64() {
				println!("ID: {}", id);
				return Ok(id);
			} else {
				println!("ID not found");
			}
		}else{
			println!("Permintaan gagal dengan status: {}", status);
		}
	}
}
fn select_mode(opt: &Opt) -> Mode {
	loop {
		println!("Pilih mode:");
		println!("1. Normal");
		println!("2. Cepat");
		println!("3. Slug");

        let input = opt.mode.clone().unwrap_or_else(|| get_user_input("Masukkan pilihan: "));

		match input.trim() {
			"1" => return Mode::Normal,
			"2" => return Mode::Fast,
			"3" => return Mode::FastSlug,
			_ => println!("Pilihan tidak valid, coba lagi."),
		}
	}
}
async fn header_redeem(slug: &str, cookie_content: &str) -> HeaderMap {
	let refe = format!("https://www.tokopedia.com/rewards/kupon/detail/{}", slug);
	let mut headers = reqwest::header::HeaderMap::new();
	headers.insert("Connection", reqwest::header::HeaderValue::from_static("keep-alive"));
	headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
	headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9,id;q=0.8"));
	headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
	headers.insert("Origin", reqwest::header::HeaderValue::from_static("https://www.tokopedia.com"));
	headers.insert("Priority", reqwest::header::HeaderValue::from_static("u=1, i"));
	headers.insert("Referer", reqwest::header::HeaderValue::from_str(&refe).unwrap());
	headers.insert("Sec-Ch-Ua", reqwest::header::HeaderValue::from_static("\"Not A(Brand\";v=\"8\", \"Google Chrome\";v=\"129\", \"Chromium\";v=\"129\""));
	headers.insert("Sec-Ch-Ua-Mobile", reqwest::header::HeaderValue::from_static("?0"));
	headers.insert("Sec-Ch-Ua-Platform", reqwest::header::HeaderValue::from_static("\"Windows\""));
	headers.insert("Sec-Fetch-Dest", reqwest::header::HeaderValue::from_static("empty"));
	headers.insert("Sec-Fetch-Mode", reqwest::header::HeaderValue::from_static("cors"));
	headers.insert("Sec-Fetch-Site", reqwest::header::HeaderValue::from_static("same-site"));
	headers.insert("user-agent", reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"));
	headers.insert("X-Source", reqwest::header::HeaderValue::from_static("tokopedia-lite"));
	headers.insert("x-tkpd-akamai", reqwest::header::HeaderValue::from_static("claimcoupon"));
	headers.insert("X-Tkpd-Lite-Service", reqwest::header::HeaderValue::from_static("zeus"));
	headers.insert("X-Version", reqwest::header::HeaderValue::from_static("2e4ea1e"));
	headers.insert("cookie", reqwest::header::HeaderValue::from_str(&cookie_content).unwrap());
    // Return the created headers
    headers
}
fn format_duration(duration: Duration) -> String {
	let hours = duration.num_hours();
	let minutes = duration.num_minutes() % 60;
	let seconds = duration.num_seconds() % 60;
	let milliseconds = duration.num_milliseconds() % 1_000;

	format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}

fn parse_task_time(task_time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
	let today = Local::now().date_naive();
	let dt = NaiveDateTime::parse_from_str(&format!("{} {}", today.format("%Y-%m-%d"), task_time_str), "%Y-%m-%d %H:%M:%S%.f")?;
	Ok(dt)
}

async fn countdown_to_task(task_time_dt: NaiveDateTime) {
	let task_time_dt = check_and_adjust_time(task_time_dt).await;

	loop {
		let current_time = Local::now().naive_local();
		let time_until_task = task_time_dt.signed_duration_since(current_time);

		if time_until_task <= Duration::zero() {
			println!("\nTask completed! Current time: {}", current_time.format("%H:%M:%S.%3f"));
			tugas_utama();
			break;
		}

		let formatted_time = format_duration(time_until_task);
		print!("\r{}", formatted_time);
		io::stdout().flush().unwrap();

		tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
	}
}
async fn check_and_adjust_time(task_time_dt: NaiveDateTime) -> NaiveDateTime {
	let mut updated_task_time_dt = task_time_dt;
	let current_time = Local::now().naive_local();
	let time_until_task = updated_task_time_dt.signed_duration_since(current_time);

	if time_until_task < Duration::zero() {
		// Jika waktu sudah berlalu, tawarkan untuk menyesuaikan waktu
		println!("Waktu yang dimasukkan telah berlalu.");
		println!("Apakah Anda ingin menyetel waktu untuk besok? (yes/no): ");
		
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Gagal membaca baris");

		match input.trim().to_lowercase().as_str() {
			"yes" | "y" => {
				// Tambahkan satu hari ke waktu target
				updated_task_time_dt += Duration::days(1);
				println!("Waktu telah disesuaikan untuk hari berikutnya: {}", updated_task_time_dt);
			}
			_ => println!("Pengaturan waktu tidak diubah."),
		}
	}

	updated_task_time_dt
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
    let file = File::open(&file_path);
    let mut cookie_content = String::new();
    let _ = file.expect("REASON").read_to_string(&mut cookie_content);
    // Trim and return the content
    let trimmed_content = cookie_content.trim().to_string();
    if trimmed_content.is_empty() {
        " ".to_string()
    } else {
        trimmed_content
    }
}
