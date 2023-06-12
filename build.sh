# 删除之前的构建文件
rm -R target/release
# 构建
cargo build --release
# 去除调试信息
strip -o target/release/get_paper_rs-stripped target/release/get_paper_rs
