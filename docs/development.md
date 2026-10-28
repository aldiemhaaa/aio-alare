# Panduan Pengembangan (Development Guide)

Dokumen ini memuat langkah-langkah menyiapkan lingkungan pengembangan lokal, menjalankan migrasi database, pengujian (*testing*), hingga proses build Docker untuk **Inventory Management System (IMS)**.

---

## 🛠️ Prasyarat Perangkat Lunak

Sebelum memulai, pastikan perangkat lokal Anda telah terinstal:
- **Rust Toolchain**: Versi 1.78 atau lebih baru ([rustup.rs](https://rustup.rs/))
- **Docker & Docker Compose**: Untuk menjalankan PostgreSQL dan Redis lokal
- **SQLx CLI**: Utilitas command-line untuk migrasi database:
  ```bash
  cargo install sqlx-cli --no-default-features --features rustls,postgres
  ```
- **Cargo Watch** (opsional untuk live-reloading):
  ```bash
  cargo install cargo-watch
  ```

---

## 🚀 Memulai Pengembangan Lokal (Quick Start)

### 1. Konfigurasi Lingkungan (`.env`)
Salin file konfigurasi contoh:
```bash
cp .env.example .env
```

Isi berkas `.env` dengan kredensial pengembangan lokal:
```env
# Server
PORT=8080
HOST=0.0.0.0
ENVIRONMENT=development
RUST_LOG=info,inventory_system=debug,tower_http=debug

# Database (PostgreSQL)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/inventory_db
DATABASE_MAX_CONNECTIONS=20

# Cache (Redis)
REDIS_URL=redis://localhost:6379

# JWT Secret
JWT_SECRET=super_secret_jwt_key_minimum_32_characters_long_12345
JWT_EXPIRATION_SECONDS=3600
JWT_REFRESH_EXPIRATION_SECONDS=604800
```

### 2. Menjalankan Layanan Pendukung (Docker Compose)
Gunakan `docker-compose.yml` berikut untuk menyalakan PostgreSQL dan Redis:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: ims_postgres
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: inventory_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    container_name: ims_redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

Jalankan container:
```bash
docker compose up -d
```

### 3. Menjalankan Migrasi Database
Gunakan `sqlx` untuk membuat database dan mengaplikasikan file migrasi di folder `migrations/`:
```bash
# Membuat basis data jika belum ada
sqlx database create

# Menjalankan seluruh file migrasi
sqlx migrate run
```

Untuk menambahkan migrasi baru di masa mendatang:
```bash
sqlx migrate add <nama_migrasi>
```

### 4. Menjalankan Server
Jalankan aplikasi dalam mode pengembangan:
```bash
cargo run
```

Atau gunakan `cargo-watch` agar server otomatis restart setiap kali berkas kode diubah:
```bash
cargo watch -x run
```

Server API akan aktif di `http://127.0.0.1:8080`.

---

## 🧪 Pengujian & Kualitas Kode (Testing & Quality)

### 1. Menjalankan Test Suite
```bash
# Menjalankan seluruh unit tests
cargo test --lib

# Menjalankan integration tests dengan output lengkap
cargo test --test '*' -- --nocapture
```

### 2. Formatter & Linter
Pastikan kode memenuhi standar Rust sebelum commit:
```bash
# Cek pemformatan kode
cargo fmt --check

# Cek peringatan / linting dengan Clippy
cargo clippy -- -D warnings
```

---

## 🐳 Containerization (Multi-Stage Dockerfile)

Untuk menghasilkan binary berukuran kecil (~25MB) dan waktu build cepat via caching dependensi, gunakan **cargo-chef**:

```dockerfile
# Tahap 1: Persiapan Chef
FROM rust:1.78-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Tahap 2: Buat cache rencana kompilasi
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Tahap 3: Caching dependensi & Build binary
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependensi - layer ini di-cache oleh Docker jika Cargo.toml tidak berubah
RUN cargo chef cook --release --recipe-path recipe.json
# Build aplikasi utama
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin inventory-system

# Tahap 4: Runtime Minimal
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Buat non-root user demi keamanan
RUN useradd -m -u 1001 appuser
USER appuser

COPY --from=builder /app/target/release/inventory-system /app/inventory-system

ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["/app/inventory-system"]
```

---

## 🩺 Health Check & Observabilitas

Endpoint standar diagnostik:
- **Liveness Probe**: `GET /health/live` $\to$ Cek apakah proses web server berjalan.
- **Readiness Probe**: `GET /health/ready` $\to$ Memastikan koneksi pool PostgreSQL dan Redis terhubung aktif.
- **Metrics**: `GET /metrics` $\to$ Metrik format Prometheus untuk scraping monitor (CPU, memory, query latency, active db connections).
