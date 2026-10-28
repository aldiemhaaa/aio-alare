# Spesifikasi Kebutuhan Sistem (System Requirements)

Dokumen ini memuat daftar kebutuhan fungsional (*Functional Requirements*) dan non-fungsional (*Non-Functional Requirements*) untuk **Inventory Management System (IMS)**.

---

## 👥 Peran Pengguna & RBAC (Role-Based Access Control)

Sistem membedakan hak akses berdasarkan peran pengguna (*role*) untuk menjaga keamanan dan akuntabilitas:

| Peran (*Role*) | Deskripsi | Hak Akses Utama |
| :--- | :--- | :--- |
| **Super Admin** | Administrator sistem pusat | Akses penuh (*Full CRUD*) ke semua modul, manajemen user, konfigurasi sistem, dan semua gudang. |
| **Warehouse Manager** | Kepala operasional gudang | Mengelola gudang yang ditugaskan, menyetujui (*approve*) penyesuaian stok (*stock opname*) dan transfer antar gudang. |
| **Inventory Staff / Operator** | Petugas operasional harian | Melakukan scan/input penerimaan barang (*inbound*), pengeluaran (*outbound*), penghitungan fisik, dan transfer barang. |
| **Auditor / Management** | Pengawas & manajemen eksekutif | Akses *read-only* untuk melihat laporan persediaan, rekap mutasi stok (*audit trail*), dan analitik. |

---

## 📋 Kebutuhan Fungsional (Functional Requirements)

### 1. Manajemen Autentikasi & Pengguna (FR-01)
- **FR-01.1**: Sistem harus menyediakan endpoint login dengan *email* dan *password*, mengembalikan *JWT Access Token* (masa berlaku pendek) dan *Refresh Token*.
- **FR-01.2**: Password harus di-hash menggunakan algoritma **Argon2id**.
- **FR-01.3**: Sistem harus memvalidasi permission untuk setiap endpoint menggunakan middleware otorisasi.
- **FR-01.4**: Pengguna dapat ditugaskan ke satu atau beberapa gudang tertentu (*warehouse scoping*).

### 2. Manajemen Katalog Produk & Master Data (FR-02)
- **FR-02.1**: Pengelolaan kategori produk bertingkat (*hierarchical categories*).
- **FR-02.2**: Pengelolaan data produk induk dan varian produk (*Product Variant*).
- **FR-02.3**: Setiap varian produk wajib memiliki **SKU (Stock Keeping Unit)** yang unik dan barcode opsional (EAN-13, UPC, Code128).
- **FR-02.4**: Pengelolaan Satuan Pengukuran (*Unit of Measure* - UOM), seperti PCS, BOX, KG, LITER.

### 3. Manajemen Multi-Warehouse & Bin Location (FR-03)
- **FR-03.1**: Dukungan banyak gudang fisik (*multi-warehouse*).
- **FR-03.2**: Pengelolaan hierarki lokasi internal gudang: **Zona $\to$ Lorong (Aisle) $\to$ Rak (Rack) $\to$ Bin/Wadah**.
- **FR-03.3**: Pemetaan status gudang (Aktif, Maintenance, Nonaktif).

### 4. Pelacakan Status Inventaris (FR-04)
Sistem memisahkan status kuantitas stok untuk mencegah *overselling*:
- **On-Hand Quantity**: Total fisik barang yang ada di gudang.
- **Reserved Quantity**: Barang yang telah dialokasikan untuk pesanan keluar (*pending dispatch*).
- **Available Quantity**: Kuantitas yang siap dijual/dialokasikan:
  $$\text{Available} = \text{On-Hand} - \text{Reserved}$$

### 5. Pengadaan & Penerimaan Barang (Inbound & PO) (FR-05)
- **FR-05.1**: Pembuatan Purchase Order (PO) ke *Supplier* dengan status alur: `DRAFT` $\to$ `ISSUED` $\to$ `PARTIALLY_RECEIVED` $\to$ `RECEIVED` $\to$ `CANCELLED`.
- **FR-05.2**: Verifikasi penerimaan fisik barang (*Goods Receipt*) yang mencocokkan jumlah datang dengan dokumen PO.
- **FR-05.3**: Pencatatan nomor batch (*Batch/Lot Number*) dan tanggal kadaluarsa (*Expiration Date*) jika relevan.
- **FR-05.4**: Mutasi stok otomatis bertambah saat status barang dinyatakan diterima (*Received*).

### 6. Pengeluaran & Pemenuhan Barang (Outbound) (FR-06)
- **FR-06.1**: Alokasi stok (*Stock Reservation*) saat pesanan penjualan (*Sales Order*) dibuat.
- **FR-06.2**: Pembuatan dokumen daftar pengambilan barang (*Picking List*) berdasarkan lokasi Bin terdekat.
- **FR-06.3**: Pemotongan stok final saat status barang dikonfirmasi keluar dari gudang (*Dispatched*).

### 7. Transfer Stok Antar Gudang (FR-07)
- **FR-07.1**: Pengajuan permintaan transfer (*Transfer Request*) dari gudang asal ke gudang tujuan.
- **FR-07.2**: Siklus transfer: `REQUESTED` $\to$ `APPROVED` $\to$ `IN_TRANSIT` $\to$ `RECEIVED`.
- **FR-07.3**: Stok sementara berada di status virtual *In-Transit* hingga diverifikasi sampai oleh staf gudang tujuan.

### 8. Stock Opname & Penyesuaian (Adjustment) (FR-08)
- **FR-08.1**: Fitur audit fisik berkala (*Cycle Count / Stock Opname*).
- **FR-08.2**: Pencatatan selisih antara perhitungan fisik dengan data sistem.
- **FR-08.3**: Penyesuaian stok yang menghasilkan selisih bernilai besar wajib membutuhkan persetujuan (*Approval*) dari **Warehouse Manager**.
- **FR-08.4**: Setiap penyesuaian wajib menyertakan alasan (*Adjustment Reason*, misal: Rusak, Kadaluarsa, Selisih Hitung).

### 9. Peringatan Stok Rendah (*Low Stock Alerts*) (FR-09)
- **FR-09.1**: Setiap varian di setiap gudang dapat dikonfigurasi ambang batas minimum (*Minimum Threshold / Reorder Point*).
- **FR-09.2**: Sistem secara otomatis menandai item yang stoknya berada di bawah ambang batas dan menyediakannya dalam laporan *Restock Need*.

### 10. Audit Trail & Double-Entry Stock Ledger (FR-10)
- **FR-10.1**: Setiap mutasi stok fisik harus menghasilkan 1 baris entri permanen di `stock_movements`.
- **FR-10.2**: Tabel pergerakan stok bersifat **immutable** (hanya boleh INSERT; tidak boleh UPDATE atau DELETE).
- **FR-10.3**: Data mutasi mencakup: Timestamp, ID Pengguna, Gudang, SKU, Perubahan Kuantitas (+/-), Referensi Transaksi, dan Kuantitas Saldo Akhir.

---

## ⚡ Kebutuhan Non-Fungsional (Non-Functional Requirements)

### NFR-01: Performa & Latensi
- Latensi pembacaan data inventaris (*Read Query*) $P_{99} < 50\text{ ms}$.
- Latensi mutasi stok dalam transaksi basis data $P_{99} < 100\text{ ms}$.
- Kapasitas throughput minimum 1.500 *mutations per second* pada spesifikasi server standar.

### NFR-02: Integritas Data & Konsistensi (ACID)
- Menjamin kepatuhan penuh terhadap prinsip ACID.
- Mutasi stok dan pencatatan buku besar (*ledger*) wajib berjalan dalam transaksi tunggal atomik (*Atomic Transaction*).
- Menjamin tidak ada stok bernilai negatif (*Strict Non-Negative Stock Invariant*).

### NFR-03: Keamanan (Security)
- Transport layer wajib menggunakan TLS 1.3 (HTTPS).
- Password disimpan dengan algoritma hash terakreditasi (*Argon2id*).
- Proteksi terhadap serangan umum web: *SQL Injection* (dicegah oleh SQLx parameterized queries), *Rate Limiting*, dan *Cross-Site Scripting*.

### NFR-04: Ketersediaan & Skalabilitas (Scalability)
- Layanan backend dirancang *stateless* sehingga dapat diskalakan secara horizontal (*Horizontal Pod Autoscaling* di Kubernetes).
- Dukungan *connection pooling* database menggunakan fitur bawaan SQLx.
