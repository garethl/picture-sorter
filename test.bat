@ECHO OFF

cargo run -- --format "{DateTime:%%Y}\{DateTime:%%m}\{FileName}" --cache-dir "C:\zps-temp\cache.db" --use-hard-links  C:\zps-temp\in C:\zps-temp\out