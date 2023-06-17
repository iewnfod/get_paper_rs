# 构建
cargo build --release
# 去除调试信息
strip -o target/release/get_paper_rs-stripped target/release/get_paper_rs
# 重命名
mv target/release/get_paper_rs target/release/get_paper_rs-darwin
mv target/release/get_paper_rs-stripped target/release/get_paper_rs-darwin-stripped
