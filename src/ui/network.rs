use rand::Rng;
use tokio::process::Command;
use std::collections::HashMap;

use crate::ui::change_status_bar_content;

use super::data;

pub static mut DOWNLOADING: bool = false;

fn should_stop() -> bool {
    if unsafe { !DOWNLOADING } {
        change_status_bar_content(&"Stopped".to_string());
        true
    } else {
        false
    }
}

pub async fn start(min_year: isize, max_year: isize, check_bts: Vec<(bool, String, String)>) -> () {
    println!("Function Start");
    println!("From {} to {}", min_year, max_year);
    for (bt, code, name) in check_bts.iter() {
        if *bt {
            println!("{}", code);
            for year in min_year..max_year+1 {
                println!("{}", year);
                for season in data::SEASONS.iter() {
                    println!("{}", season);
                    change_status_bar_content(&format!("Searching: {} | {} | {}", &name, year, &season));
                    let available_files =
                        search_file(&code, year.to_string().as_str(), season).await;

                    // println!("{}", available_files);

                    if available_files["status"] == 0 {
                        let files_value = &available_files["data"];
                        if let Some(files) = files_value.as_array() {
                            for i in files.iter() {
                                // 如果 downloading 被改成 false 了，表示他要停止了
                                if should_stop() { return ; }
                                let file_name = i[0].to_string();
                                let file_name = file_name[1..file_name.len()-1].to_string();
                                // println!("{}", file_name);
                                let mut url = data::FETCH_URL.to_string();
                                url.push_str(&file_name.as_str());
                                let save_path = format!("{}/{}/{}/{}", data::get_save_dir(), &name, year, file_name);
                                change_status_bar_content(&format!("Downloading: {}", &save_path));
                                let mut status = download(&url, &save_path).await;
                                while !status {
                                    sleep().await;
                                    change_status_bar_content(&format!("Retry: {}", save_path));
                                    status = download(&url, &save_path).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    change_status_bar_content(&"Finish".to_string());
    // return ;
}

async fn sleep() {
    let sleep_time: isize = rand::thread_rng().gen_range(1..5);
    let mut waiter = Command::new("sleep")
        .arg(sleep_time.to_string())
        .spawn()
        .unwrap();
    waiter.wait().await.unwrap();
}

async fn search_file(subject: &str, year: &str, season: &str) -> serde_json::Value {
    let url = data::SEARCH_URL.to_string();
    let mut map = HashMap::new();
    map.insert("subject", subject);
    map.insert("year", year);
    map.insert("season", season);

    let client = reqwest::Client::new();
    let response = client.post(url)
        .form(&map)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let result: serde_json::Value = serde_json::from_str(response.as_str()).unwrap();
    return result;
}

async fn download(url: &String, save_path: &String) -> bool {
    if should_stop() { return true; }
    // 创建文件以及其存在的目录
    let path = std::path::Path::new(save_path);
    if path.exists() {
        return true;
    }
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    // 发送请求，获取文件
    let response = match reqwest::get(url).await {
        Ok(r) => r,
        Err(_) => {
            return false;
        }
    };

    println!("{} : {}", save_path, response.status());

    // 判断是否成功
    if response.status().is_success() {
        let result = response.bytes()
        .await
        .unwrap();

        // 写入文件
        std::fs::write(path, result).unwrap();

        true
    } else {
        false
    }
}
