use fltk::{prelude::*};
use std::collections::HashMap;

use super::{Buffer, data};

pub async fn start(buffer: Buffer) -> () {
    let min_year: isize = buffer.min_year_input.value().parse().unwrap();
    let max_year: isize = buffer.max_year_input.value().parse().unwrap();
    let check_bts = buffer.check_bts.clone();
    for (bt, code, name) in check_bts {
        if bt.value() {
            println!("{}", code);
            for year in min_year..max_year+1 {
                println!("{}", year);
                for season in data::SEASONS {
                    println!("{}", season);
                    let available_files =
                        search_file(&code, year.to_string().as_str(), season).await;

                    println!("{}", available_files);

                    if available_files["status"] == 0 {
                        let files_value = &available_files["data"];
                        if let Some(files) = files_value.as_array() {
                            for i in files.iter(){
                                let file_name = i[0].to_string();
                                let file_name = file_name[1..file_name.len()-1].to_string();
                                println!("{}", file_name);
                                let mut url = data::FETCH_URL.to_string();
                                url.push_str(&file_name.as_str());
                                let save_path = format!("{}/{}/{}/{}", data::SAVE_DIR, &name, year, file_name);
                                download(&url, &save_path).await;
                            }
                        }
                    }
                }
            }
        }
    }
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

async fn download(url: &String, save_path: &String) {
    // 创建文件以及其存在的目录
    let path = std::path::Path::new(save_path);
    if path.exists() {
        return ;
    }
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    // 发送请求，获取文件
    let result = reqwest::get(url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    // 写入文件
    std::fs::write(path, result).unwrap();
}
