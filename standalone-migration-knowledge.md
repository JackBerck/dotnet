# Knowledge 01 — Laragon → Standalone Windows Development Environment

## Tujuan

Migrasi environment development Windows dari Laragon ke instalasi standalone dengan prinsip:

> **Migrasi → verifikasi → baru cleanup.**

Laragon dipertahankan sebagai fallback sampai project dan tool penting terbukti aman.

## 1. PHP

- PHP 8.5.10
- Lokasi: `C:\tools\php85`
- Config: `C:\tools\php85\php.ini`
- Extension penting: curl, intl, mbstring, mysqli, openssl, pdo_mysql, zip
- PHP CLI sudah terverifikasi.
- PHP standalone digunakan oleh Composer.
- `php-cgi.exe` digunakan untuk FastCGI Windows.

## 2. Composer

- Composer standalone sudah terpasang.
- Menggunakan PHP standalone.
- Packagist/GitHub, TLS/HTTPS, `composer diagnose`, dan security audit sudah diverifikasi.

## 3. Git

Git standalone sudah terpasang dan CLI berjalan.

## 4. Node / NVM / npm / Bun

NVM for Windows digunakan.

Versi Node yang tersedia:
- 24.13.1
- 23.11.0
- 22.17.0

Bun:
- 1.3.14

NVM sudah diverifikasi.

## 5. MySQL

### Source Laragon

- MySQL Community Server 8.0.30
- Data: `C:\laragon\data\mysql-8\`
- Port: 3306

### Backup

- `D:\Code\mysql-migration\laragon-mysql-2026-09-02.sql`
- `D:\Code\mysql-migration\laragon-project-databases-2026-09-02.sql`

### Target

- MySQL 8.4.11
- Service: `MySQL84`
- Data: `C:\ProgramData\MySQL\MySQL Server 8.4\Data\`
- Port: 3306

Restore berhasil:
- 54 database project.
- Total 58 database termasuk system database.
- Legacy FK dari MySQL 8.0 membutuhkan `restrict_fk_on_non_standard_key=OFF` hanya selama restore.
- Setelah restore, global setting dikembalikan ke `ON`.

## 6. PostgreSQL

### Source

- PostgreSQL 17.5
- `C:\laragon\bin\postgresql\pgsql-17.5-2\`
- Port 5432
- 13 non-template database terdeteksi.

### Backup

Globals:
`D:\Code\postgresql-migration\postgresql-globals-2026-09-02.sql`

Database dumps:
`D:\Code\postgresql-migration\databases\`

Format dump: `-Fc`.

### Target

- PostgreSQL 17.11
- `C:\Program Files\PostgreSQL\17\`
- Service: `postgresql-x64-17`
- Port 5432

Globals dan seluruh database project yang dipilih berhasil direstore.

## 7. Redis / Memurai

Memurai berhasil dipasang sebagai Redis-compatible Windows service.

- Port: 6379
- CLI: `memurai-cli`
- Health check sudah diverifikasi dan berhasil.

## 8. Nginx

- Lokasi: `C:\tools\nginx`
- Port: 80
- Config utama: `C:\tools\nginx\conf\nginx.conf`
- Virtual host: `C:\tools\nginx\conf\sites\`

Config utama menggunakan:

```nginx
include sites/*.conf;
```

Nginx standalone sudah berhasil start dan konfigurasi sudah lolos `nginx -t`.

## 9. PHP FastCGI

Windows PHP tidak menggunakan `php-fpm.exe` seperti Linux.

FastCGI menggunakan:

`C:\tools\php85\php-cgi.exe`

Endpoint:

`127.0.0.1:9000`

Arsitektur:

```text
Browser
  ↓
Nginx :80
  ↓ FastCGI
php-cgi :9000
  ↓
Laravel
```

Sudah berhasil dijalankan.

## 10. Laravel

Workflow development utama tetap:

```bash
composer run dev
```

Nginx bukan pengganti workflow tersebut. Nginx dipakai sebagai mode tambahan untuk testing web-server-like.

Project Laravel utama dan project lain yang relevan sudah diuji dan aman.

## 11. Virtual Host

Contoh:

```text
D:\Code\mrebet
```

Host:

```text
mrebet.test
```

Nginx mengarah ke:

```text
D:/Code/mrebet/public
```

Hosts:

```text
127.0.0.1 mrebet.test
```

Akses melalui Nginx sudah berhasil.

## 12. PATH dan Laragon

PATH belum dibersihkan. Entry Laragon masih boleh ada sementara.

Laragon belum dihapus.

Ini disengaja agar tetap menjadi fallback.

## 13. Status

```text
PHP                    DONE
Composer               DONE
Git                    DONE
Node/NVM/npm           DONE
Bun                    DONE
MySQL                  DONE
PostgreSQL             DONE
Redis/Memurai          DONE
Nginx                  DONE
PHP FastCGI            DONE
Laravel testing        DONE
Other project testing  DONE
Nginx multi-project    DEFERRED
PATH cleanup           PENDING
Laragon retirement     PENDING
Developer manager      NEXT
Docker conflict mgmt   NEXT
```

## Prinsip

Jangan menghapus Laragon atau membersihkan PATH sebelum project penting, database, runtime, Redis, dan Docker workflow benar-benar aman.
