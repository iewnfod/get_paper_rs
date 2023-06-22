remove-item 'target\release\get_paper_rs.exe'
remove-item 'target\release\get_paper_rs-windows.exe'
cargo build --release
rename-item 'target\release\get_paper_rs.exe' -newname 'get_paper_rs-windows.exe'
