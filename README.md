# Blazingly Fast URL Shortener

A high-performance URL shortener service built with Rust, featuring encryption, MongoDB storage, and a clean REST API.

## Features

- ⚡ High-performance Rust implementation
- 🔒 URL encryption using AES-256-GCM
- 📦 MongoDB storage with indexed lookups
- 🔄 Automatic URL deduplication
- ⏰ Optional URL expiration
- 📊 Click tracking
- 🌐 RESTful API

## Demo 
[![Blazingly Fast URL Shortener](https://img.youtube.com/vi/-KUp34PHJuc/0.jpg)](https://youtu.be/-KUp34PHJuc)

## Prerequisites

- Rust (latest stable)
- MongoDB (4.x or later)
- OpenSSL development libraries

## Quick Start

### Clone the repository

```bash
git clone https://github.com/sanjaysah101/blazinglyfast-url-shortener
cd url-shortener
```

### Create a `.env` file

```env
MONGODB_URI=mongodb://localhost:27017/
ENCRYPTION_KEY=your_base64_encoded_32_byte_key
```

Note: To generate a 32-byte key, you can use the following command:

```bash
openssl rand -base64 32
```

### Build and run

```bash
cargo run
```

The server will start at `http://127.0.0.1:8080`

## API Endpoints

### Create Short URL

```http
POST /api/urls
Content-Type: application/json

{
    "url": "https://example.com/very/long/url",
    "expires_in_days": 30  // optional
}
```

### Get All URLs

```http
GET /api/urls
```

### Redirect to Original URL

```http
GET /r/{short_code}
```

## Security Features

- AES-256-GCM encryption for stored URLs
- Nonce-based encryption for enhanced security
- Base64-encoded encryption key support
- URL validation before storage

## Performance

- Asynchronous request handling with Actix Web
- Indexed MongoDB queries for fast lookups
- Connection pooling for database operations
- Efficient in-memory encryption/decryption

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run with logging:

```bash
RUST_LOG=debug cargo run
```

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
