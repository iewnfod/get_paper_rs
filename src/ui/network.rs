use rand::Rng;
use std::{collections::HashMap, path::Path};
use tokio::process::Command;

use crate::ui::change_status_bar_content;

use super::data;

pub static mut DOWNLOADING: bool = false;

/// 判断下载是否应该停下，退出当前循环或函数
/// * return `bool` 表示是否应该停下
fn should_stop() -> bool {
    if unsafe { !DOWNLOADING } {
        change_status_bar_content(&"Stopped".to_string());
        true
    } else {
        false
    }
}

/// 异步下载启动函数
/// * param `min_year: isize` 最小年份，即要从哪一年开始下载
/// * param `max_year: isize` 最大年份，即最后要下载到哪一年
/// * param `check_bts: Vec<(bool, String, String)>` 依据所有的选项生成的数组，包含是否选择，以及每个选项的内容
/// * return `()`
pub async fn start(min_year: isize, max_year: isize, check_bts: Vec<(bool, String, String)>) -> () {
    println!("Function Start");
    println!("From {} to {}", min_year, max_year);
    for (bt, code, name) in check_bts.iter() {
        if *bt {
            println!("{}", code);
            for year in min_year..max_year + 1 {
                println!("{}", year);
                for season in data::SEASONS.iter() {
                    println!("{}", season);
                    change_status_bar_content(&format!(
                        "Searching: {} | {} | {}",
                        &name, year, &season
                    ));
                    let available_files =
                        search_file(&code, year.to_string().as_str(), season).await;

                    // println!("{}", available_files);

                    if available_files["status"] == 0 {
                        let files_value = &available_files["data"];
                        if let Some(files) = files_value.as_array() {
                            for i in files.iter() {
                                // 如果 downloading 被改成 false 了，表示他要停止了
                                if should_stop() {
                                    return;
                                }
                                let file_name = i[0].to_string();
                                let file_name = file_name[1..file_name.len() - 1].to_string();
                                // println!("{}", file_name);
                                let mut url = data::FETCH_URL.to_string();
                                url.push_str(&file_name.as_str());
                                let save_path = Path::new(&data::get_save_dir())
                                    .join(name)
                                    .join(year.to_string())
                                    .join(&file_name)
                                    .to_str()
                                    .unwrap()
                                    .to_string();
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

/// 暂停当前任务，随机 1 到 5 秒，用于解决发送请求过多
/// return `()`
async fn sleep() -> () {
    let sleep_time: isize = rand::thread_rng().gen_range(1..5);
    let mut waiter = Command::new("sleep")
        .arg(sleep_time.to_string())
        .spawn()
        .unwrap();
    waiter.wait().await.unwrap();
}

/// 异步搜索文件
/// * param `subject: &str` 搜索的学科
/// * param `year: &str` 搜索的年份
/// * param `season: &str` 搜索的季节
/// * return `serde_json::Value` 返回Json格式的解析结果，类似于字典的使用
async fn search_file(subject: &str, year: &str, season: &str) -> serde_json::Value {
    let url = data::SEARCH_URL.to_string();
    let mut map = HashMap::new();
    map.insert("subject", subject);
    map.insert("year", year);
    map.insert("season", season);

    let client = reqwest::Client::new();
    let response = client
        .post(url)
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

/// 异步下载
/// * param `url: &String` 目标文件的url地址
/// * param `save_path: &String` 保存文件的路径
/// * return `bool` 是否下载成功
async fn download(url: &String, save_path: &String) -> bool {
    if should_stop() {
        return true;
    }
    // 创建文件以及其存在的目录
    let path = std::path::Path::new(save_path);
    if path.exists() {
        return true;
    }
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    // 发送请求，获取文件
    let response = reqwest::get(url).await.unwrap();

    println!("{} : {}", save_path, response.status());

    // 判断是否成功
    if response.status().is_success() {
        let result = response.bytes().await.unwrap();

        // 写入文件
        std::fs::write(path, result).unwrap();

        true
    } else {
        false
    }
}
