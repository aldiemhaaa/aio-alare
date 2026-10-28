# Spesifikasi RESTful API (API Specification)

Dokumen ini mendefinisikan standar konvensi, otentikasi, struktur respons, serta daftar endpoint utama untuk **Inventory Management System (IMS)**.

---

## 🌐 Konvensi Umum API

- **Base URL**: `/api/v1`
- **Format Payload**: `application/json`
- **Format Tanggal & Waktu**: ISO 8601 UTC (misal: `2026-10-28T06:30:00Z`)
- **Otentikasi**: Header HTTP `Authorization: Bearer <jwt_access_token>`

---

## 🛡️ Format Standar Respons & Error

### 1. Respons Berhasil Terstandar
```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "limit": 20,
    "total_records": 142,
    "total_pages": 8
  }
}
```

### 2. Respons Error (RFC 7807 Problem Details)
```json
{
  "type": "https://api.ims.local/errors/insufficient-stock",
  "title": "Insufficient Stock",
  "status": 409,
  "detail": "Hanya tersedia 4 unit di Gudang Jakarta, diminta 10 unit.",
  "instance": "/api/v1/inventory/adjust",
  "invalid_params": [],
  "timestamp": "2026-10-28T06:30:00Z"
}
```

---

## 📌 Daftar Endpoint Utama

### 1. Autentikasi & Akun (`/auth`)

#### `POST /api/v1/auth/login`
- **Deskripsi**: Masuk ke sistem dan mendapatkan pasangan token JWT.
- **Request Body**:
  ```json
  {
    "email": "manager.jkt@inventory.com",
    "password": "SecurePassword123!"
  }
  ```
- **Response `200 OK`**:
  ```json
  {
    "success": true,
    "data": {
      "access_token": "eyJhbGciOi...",
      "refresh_token": "def456...",
      "token_type": "Bearer",
      "expires_in": 3600,
      "user": {
        "id": "e4b2d56a-6881-432a-bc91-2a4fa96e5b41",
        "email": "manager.jkt@inventory.com",
        "full_name": "Budi Santoso",
        "role": "WAREHOUSE_MANAGER"
      }
    }
  }
  ```

#### `GET /api/v1/auth/me`
- **Deskripsi**: Mengambil profil pengguna yang sedang login.
- **Header**: `Authorization: Bearer <token>`

---

### 2. Manajemen Produk & Varian (`/products`)

#### `GET /api/v1/products`
- **Query Params**: `page=1`, `limit=20`, `search=baut`, `category_id=<uuid>`
- **Response `200 OK`**: Daftar ringkas produk beserta jumlah varian aktif.

#### `POST /api/v1/products`
- **Permission**: `SUPER_ADMIN`
- **Request Body**:
  ```json
  {
    "name": "Baut Baja Hitam M8",
    "category_id": "a1b2c3d4-0000-0000-0000-000000000001",
    "description": "Baut baja high-tensile grade 8.8",
    "uom": "PCS",
    "variants": [
      {
        "sku": "BAUT-M8-30MM",
        "barcode": "8991234567890",
        "variant_name": "Panjang 30mm",
        "cost_price": 2500.00,
        "selling_price": 4000.00
      },
      {
        "sku": "BAUT-M8-50MM",
        "barcode": "8991234567891",
        "variant_name": "Panjang 50mm",
        "cost_price": 3200.00,
        "selling_price": 5500.00
      }
    ]
  }
  ```

---

### 3. Operasi Inventaris & Mutasi Stok (`/inventory`)

#### `GET /api/v1/inventory`
- **Query Params**:
  - `warehouse_id`: UUID gudang
  - `low_stock_only`: boolean (`true` / `false`)
  - `search`: cari berdasarkan SKU atau nama varian
- **Response `200 OK`**:
  ```json
  {
    "success": true,
    "data": [
      {
        "warehouse_id": "99aa88bb-1111-2222-3333-444455556666",
        "warehouse_name": "Gudang Utama Jakarta",
        "variant_id": "77bb88cc-3333-4444-5555-666677778888",
        "sku": "BAUT-M8-30MM",
        "variant_name": "Panjang 30mm",
        "quantity_on_hand": 120,
        "reserved_quantity": 20,
        "available_quantity": 100,
        "min_threshold": 30,
        "location_code": "WH1-Z1-A01-R01-B05"
      }
    ]
  }
  ```

#### `POST /api/v1/inventory/adjust` (Stock Opname)
- **Permission**: `WAREHOUSE_MANAGER`, `SUPER_ADMIN`
- **Request Body**:
  ```json
  {
    "warehouse_id": "99aa88bb-1111-2222-3333-444455556666",
    "product_variant_id": "77bb88cc-3333-4444-5555-666677778888",
    "actual_quantity": 115,
    "reason": "PHYSICAL_COUNT_MISMATCH",
    "notes": "5 unit rusak akibat kebocoran atap di rak B05"
  }
  ```
- **Response `200 OK`**: Mengembalikan saldo baru dan ID mutasi ledger.

#### `POST /api/v1/inventory/transfer` (Transfer Antar Gudang)
- **Request Body**:
  ```json
  {
    "source_warehouse_id": "99aa88bb-1111-2222-3333-444455556666",
    "destination_warehouse_id": "11bb22cc-4444-5555-6666-777788889999",
    "items": [
      {
        "product_variant_id": "77bb88cc-3333-4444-5555-666677778888",
        "quantity": 50
      }
    ],
    "notes": "Alokasi restock gudang Surabaya"
  }
  ```

---

### 4. Buku Besar Mutasi Stok (`/stock-movements`)

#### `GET /api/v1/stock-movements`
- **Query Params**:
  - `warehouse_id`: Filter gudang
  - `product_variant_id`: Filter SKU/Varian
  - `from_date`: `2026-10-01`
  - `to_date`: `2026-10-28`
- **Response `200 OK`**:
  ```json
  {
    "success": true,
    "data": [
      {
        "id": "f5a1b2c3-4d5e-6f7a-8b9c-0d1e2f3a4b5c",
        "timestamp": "2026-10-28T04:15:30Z",
        "warehouse_code": "JKT-01",
        "sku": "BAUT-M8-30MM",
        "movement_type": "ADJUSTMENT_SUB",
        "quantity_delta": -5,
        "balance_after": 115,
        "reference_type": "STOCK_OPNAME",
        "user_email": "manager.jkt@inventory.com"
      }
    ]
  }
  ```

---

### 5. Pengadaan Barang (`/purchase-orders`)

#### `POST /api/v1/purchase-orders`
- **Request Body**:
  ```json
  {
    "supplier_id": "88aa99bb-2222-3333-4444-555566667777",
    "warehouse_id": "99aa88bb-1111-2222-3333-444455556666",
    "items": [
      {
        "product_variant_id": "77bb88cc-3333-4444-5555-666677778888",
        "quantity_ordered": 500,
        "unit_cost": 2400.00
      }
    ]
  }
  ```

#### `POST /api/v1/purchase-orders/{id}/receive`
- **Deskripsi**: Menerima kiriman barang fisik dari supplier. Secara otomatis menambah saldo stok dan mencatat mutasi `INBOUND_PO`.
- **Request Body**:
  ```json
  {
    "received_items": [
      {
        "product_variant_id": "77bb88cc-3333-4444-5555-666677778888",
        "quantity_received": 500,
        "location_id": "cc334455-6677-8899-0011-223344556677"
      }
    ]
  }
  ```
