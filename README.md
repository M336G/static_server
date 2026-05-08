# Static Server
**Lightweight & superfast static file server supporting in-memory caching**

*Even supports nested paths!*

## Prerequisites
- A device or server capable of running 24/7

## Running
***Downloading from the releases tab***

**1.** Download the corresponding archive for your OS and extract it:
- [Windows](https://github.com/M336G/fcp/releases/latest/download/static_server-windows.zip)
- [macOS](https://github.com/M336G/fcp/releases/latest/download/static_server-macos.zip)
- [Linux](https://github.com/M336G/fcp/releases/latest/download/static_server-linux.zip)

**2.** Create a `.env` file and fill it according to your needs using [`.env.example`](https://github.com/M336G/static_server/blob/main/.env.example) as a template

**3.** Run the executable to start your instance

***Building manually***

**1.** Install [Rust](https://rust-lang.org/), then [download the repository manually](https://github.com/M336G/static_server/archive/refs/heads/main.zip) or clone it:
```bash
git clone https://github.com/M336G/static_server.git
cd static_server
```

**2.** Create a `.env` file and fill it according to your needs using [`.env.example`](https://github.com/M336G/static_server/blob/main/.env.example) as a template

**3.** Start the instance with:
- `cargo run --release` for production
- `cargo run` for development/testing

**Once you've done all of this, you should have a running instance!**

## Usage
Once your instance is running, you can use:
- `GET /` - to check if your server's alive
- `GET /<path/to/file>` - to serve any file within your `STORAGE_DIRECTORY`, including files in subdirectories

## License
This project is licensed under the [MIT License](https://github.com/M336G/static_server/blob/main/LICENSE).