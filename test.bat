@ECHO OFF

cargo run -- --format "{DateTime:%%Y}\{DateTime:%%m}\{FileName}" --cache-dir "C:\zps-temp\cache.db"  C:\zps-temp\in C:\zps-temp\out