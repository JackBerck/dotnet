# Knowledge 02 — Standalone Development Environment Reference

## Komponen

| Komponen | Lokasi / Service | Port |
|---|---|---:|
| PHP | `C:\tools\php85` | - |
| Nginx | `C:\tools\nginx` | 80 |
| PHP FastCGI | `php-cgi.exe` | 9000 |
| MySQL | `MySQL84` | 3306 |
| PostgreSQL | `postgresql-x64-17` | 5432 |
| Memurai | Windows service | 6379 |
| Node | NVM for Windows | - |
| Bun | Standalone | - |

## Service Control

Start:

```powershell
Start-Service MySQL84, postgresql-x64-17, Memurai
```

Stop:

```powershell
Stop-Service MySQL84, postgresql-x64-17, Memurai
```

Check:

```powershell
Get-Service MySQL84, postgresql-x64-17, Memurai |
    Select-Object Name, Status
```

## Nginx Control

```powershell
cd C:\tools\nginx
.\nginx.exe -t
.\nginx.exe
.\nginx.exe -s reload
.\nginx.exe -s stop
```

Nginx tidak wajib aktif saat menggunakan `composer run dev`.

## PHP FastCGI

Start:

```powershell
cd C:\tools\php85
.\php-cgi.exe -b 127.0.0.1:9000
```

Health check:

```powershell
netstat -ano | findstr :9000
```

## Port Inspection

Port penting:

```text
80    Nginx
3306  MySQL
5432  PostgreSQL
6379  Redis/Memurai
9000  PHP FastCGI
```

Inspect:

```powershell
netstat -ano | findstr :3306
netstat -ano | findstr :5432
netstat -ano | findstr :6379
netstat -ano | findstr :80
netstat -ano | findstr :9000
```

Cari proses berdasarkan PID:

```powershell
tasklist /FI "PID eq <PID>"
```

## Docker Conflict

Sebelum menjalankan Docker Compose, manager harus dapat:
1. Membaca effective Compose config.
2. Mendeteksi host port mappings.
3. Membandingkan dengan port standalone.
4. Mengidentifikasi service/proses pemilik port.
5. Menawarkan stop service, ganti port Docker, atau cancel.
6. Tidak force-kill proses tanpa konfirmasi.

Port yang paling penting:

```text
3306 MySQL
5432 PostgreSQL
6379 Redis
80 Nginx
9000 PHP FastCGI
```

## PHP Config

File:

```text
C:\tools\php85\php.ini
```

Manager nantinya dapat mengelola setting penting seperti:
- memory_limit
- upload_max_filesize
- post_max_size
- max_execution_time
- date.timezone
- extension

Selalu backup sebelum perubahan.

## Hosts

File:

```text
C:\Windows\System32\drivers\etc\hosts
```

Contoh:

```text
127.0.0.1 mrebet.test
127.0.0.1 redhub.test
127.0.0.1 rembugkop.test
```

Manager harus menangani privilege Administrator hanya ketika diperlukan, mendeteksi duplicate entry, dan membuat backup sebelum perubahan.

## Nginx Config

```text
C:\tools\nginx\conf\nginx.conf
C:\tools\nginx\conf\sites\
```

Sebelum reload:

```powershell
nginx.exe -t
```

Jika validasi gagal, jangan reload.

## Project Model

Contoh project:

```text
Name: mrebet
Path: D:\Code\mrebet
PHP: 8.5
Database: MySQL
Redis: required
Dev command: composer run dev
Nginx host: mrebet.test
```

Project manager dapat menyediakan:
- Open project
- Open terminal
- Run dev command
- Open browser
- Open `.env`
- Open Nginx config

## Desired UX

```text
Standalone Dev Manager

MySQL        ● Running   [Stop]
PostgreSQL   ● Running   [Stop]
Redis        ● Running   [Stop]
Nginx        ● Running   [Stop]
PHP FastCGI  ● Running   [Stop]

[ Start All ] [ Stop All ]
[ Check Ports ] [ Settings ]
```

## Safety

Manager harus:
- backup konfigurasi sebelum modifikasi;
- tidak silent-delete;
- tidak force-kill sembarang proses;
- tidak menyimpan password plaintext;
- tidak otomatis membersihkan PATH;
- tidak otomatis uninstall Laragon;
- memakai elevation hanya jika perlu;
- menampilkan operasi yang mengubah sistem.

## Prioritas

1. Developer manager.
2. Service lifecycle.
3. Port conflict detection.
4. Centralized config.
5. Docker integration.
6. PATH cleanup.
7. Laragon retirement.

Nginx multi-project boleh ditunda karena tidak urgent.
