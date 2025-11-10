# picture-sorter

A fast, command-line tool for organizing photos and videos into a structured directory layout based on their EXIF
metadata. Point it at a messy folder of images and it'll sort them into date-based (or any metadata-based) directories
automatically.

## Why

Camera rolls, phone backups, and cloud exports all dump files into flat or inconsistent folder structures. Manually
organizing thousands of photos is tedious. `picture-sorter` reads the EXIF data embedded in your files and sorts them
into a clean, predictable directory tree — like `2024/01/IMG_1234.jpg` — in seconds.

## What It Does

- Sorts photos and videos into directories based on EXIF metadata (dates, camera model, or any tag `exiftool` can read)
- Supports flexible format strings with fallback keys — e.g. try `DateTimeOriginal`, fall back to `MediaCreateDate`
- Extracts date/time from filenames (e.g. `20231231_212454.jpg`) when EXIF data is missing
- Handles motion photos (Google/Samsung) by optionally splitting them into a still image and a separate `.mp4`
- Caches metadata in a local SQLite database so re-runs are near-instant
- Processes files in parallel for speed
- Supports copy, move, and hard-link modes
- Dry-run mode to preview what would happen without touching any files
- Glob-style exclusion patterns to skip files like `.trashed-*` or `Thumbs.db`

## When To Use It

- Organizing a phone backup or camera SD card dump
- Cleaning up a cloud photo export (Google Takeout, iCloud, etc.)
- Building a date-sorted archive from years of unsorted photos
- Maintaining an ongoing sorted mirror of a source directory (using hard-link mode)
- Any time you need to impose structure on a pile of media files

## Prerequisites

- [exiftool](https://exiftool.org/) must be installed and available on your `PATH`.

## Installation

```bash
# Clone and build from source
git clone https://github.com/garethl/picture-sorter.git
cd picture-sorter
cargo build --release

# The binary will be at target/release/picture-sorter
```

## Usage

```
picture-sorter [OPTIONS] --format <FORMAT> --cache-file <CACHE_FILE> <SOURCE> <DESTINATION>
```

### Arguments

| Argument        | Description                             |
| --------------- | --------------------------------------- |
| `<SOURCE>`      | Source directory to scan                |
| `<DESTINATION>` | Destination directory to put files into |

### Options

| Option                          | Description                                                             |
| ------------------------------- | ----------------------------------------------------------------------- |
| `-f, --format <FORMAT>`         | Format string (required). See [Format Strings](#format-strings) below.  |
| `-c, --cache-file <CACHE_FILE>` | Path to the cache file (required). Created automatically if missing.    |
| `-m, --mode <MODE>`             | `copy` (default), `move`, or `hard-link`                                |
| `-o, --overwrite`               | Overwrite existing files at the destination instead of skipping         |
| `-e, --exclude <EXCLUDE>...`    | Exclude files matching a pattern (`*` is a wildcard)                    |
| `-d, --dry-run`                 | Preview what would happen without writing anything                      |
| `--motion-extract`              | Extract the video from motion photos into a separate `_motion.mp4` file |
| `--motion-strip`                | Strip the embedded video from motion photos when copying the image      |
| `-q, --quiet`                   | Errors only                                                             |
| `-v, --verbose`                 | Verbose logging                                                         |
| `-h, --help`                    | Print help                                                              |
| `-V, --version`                 | Print version                                                           |

### Format Strings

The `--format` flag controls how destination paths are constructed. It uses `{key:format}` expressions that are
evaluated against each file's EXIF metadata.

**Syntax:** `{key|altkey|...:format}`

- `key` — an EXIF tag name (case-insensitive). Multiple keys separated by `|` act as fallbacks.
- `format` — an optional [chrono format string](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) applied
  when the value is a date/time.

The special key `datetime` is a built-in alias that tries these sources in order: `DateTime` → `DateTimeOriginal` →
`MediaCreateDate` → `GPSDateTime` → filename pattern (`YYYYMMDD_HHMMSS`).

**Examples:**

```bash
# Sort into Year/Month/filename.ext
--format "{datetime:%Y}/{datetime:%B}/{FileName}"

# Sort by camera model, then by year
--format "{Model}/{datetime:%Y}/{FileName}"

# Year/Month with zero-padded month number
--format "{datetime:%Y}/{datetime:%m}/{FileName}"
```

### Examples

Sort photos by year and month, copying them:

```bash
picture-sorter ./photos ./sorted \
  -f "{datetime:%Y}/{datetime:%B}/{FileName}" \
  -c cache.db
```

Move files instead of copying, excluding trashed files:

```bash
picture-sorter ./photos ./sorted \
  -f "{datetime:%Y}/{datetime:%m}/{FileName}" \
  -c cache.db \
  -m move \
  -e ".trashed-*"
```

Preview what would happen without making changes:

```bash
picture-sorter ./photos ./sorted \
  -f "{datetime:%Y}/{FileName}" \
  -c cache.db \
  --dry-run --verbose
```

Hard-link files (source and destination must be on the same volume):

```bash
picture-sorter ./photos ./sorted \
  -f "{datetime:%Y}/{datetime:%m-%d}/{FileName}" \
  -c cache.db \
  -m hard-link
```

## Motion Photo Handling

`picture-sorter` can detect Google/Samsung motion photos (v1) and split them into their still image and video
components. This behaviour is off by default — you need to opt in with one or both of the following flags:

- `--motion-extract` — extracts the embedded video and writes it alongside the image as `<filename>_motion.mp4`
- `--motion-strip` — removes the embedded video from the image file, producing a smaller still image at the destination

If both flags are provided, the video is extracted to a separate file _and_ the destination image has the video data
stripped out. If only `--motion-extract` is used, the video is extracted but the image is copied as-is (with the video
still embedded). If only `--motion-strip` is used, the image is copied without the video but no separate video file is
created.

Without either flag, motion photos are sorted like any other file — the embedded video is left intact and no extraction
occurs.

## Caching

EXIF extraction via `exiftool` is the slowest part of the process. The `--cache-file` flag points to a SQLite database
where metadata is stored after the first read. Subsequent runs skip `exiftool` entirely for already-cached files, making
re-runs and incremental sorts very fast.

## License

See [LICENSE](LICENSE) for details.
