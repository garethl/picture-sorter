@ECHO OFF

cargo run -- --format "{DateTime:%%Y}\{DateTime:%%m}\{FileName}" --cache-dir "C:\zps-temp\cache.db" --use-hard-links --verbose C:\zps-temp\in\t C:\zps-temp\out