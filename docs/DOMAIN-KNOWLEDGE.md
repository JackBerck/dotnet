# DOMAIN-KNOWLEDGE: Core Specifications & Rules

> Essential domain knowledge, business rules, and technical invariants ported from `dotnet-cs`.

---

## 1. Tool Catalog & Safe Installation

1. **Manifest Schema**:
   - Each tool has a JSON manifest (`id`, `displayName`, `category`, `versions`, `layout`, `postInstall`, `verify`).
   - Manifests are embedded in binary via `include_str!` and can be augmented via `%LOCALAPPDATA%\Dotnet\catalog\`.
2. **Security & Validation**:
   - Downloads restricted to `allowedHosts` declared in the manifest.
   - SHA-256 verification performed on stream while writing to disk.
   - Zip extraction enforces canonical prefix matching to block Zip-Slip attacks.
3. **User PATH Integration**:
   - Tools install to `%LOCALAPPDATA%\Dotnet\tools\<id>\<version>\`.
   - Tool directory or junction (`current`) registered in `HKCU\Environment\Path` as `REG_EXPAND_SZ`.
   - `WM_SETTINGCHANGE` broadcasted with 2-second timeout to refresh running processes.

---

## 2. Hosts File Management (Managed Blocks)

1. **Windows Hosts Path**: `C:\Windows\System32\drivers\etc\hosts`.
2. **Managed Block Boundary**:
   ```
   # >>> Dotnet (managed) >>>
   127.0.0.1  myproject.test
   127.0.0.1  api.myproject.test
   # <<< Dotnet (managed) <<<
   ```
3. **Safety Invariants**:
   - The application NEVER edits lines outside the managed block.
   - Backups created in `%LOCALAPPDATA%\Dotnet\backups\hosts_YYYYMMDD_HHMMSS.bak`.
   - Updates written atomically to temporary file before replacing.

---

## 3. Nginx Virtual Host & Local Domains

1. **VHost Directory**: `%LOCALAPPDATA%\Dotnet\nginx\sites\`.
2. **Template Structure**:
   - Port 80 with redirect to 443 (when SSL enabled).
   - Port 443 with self-signed certificate routing.
   - PHP FastCGI proxy forwarding to `127.0.0.1:9000` (or dynamic pool `upstream php_pool`).
3. **Validation & Rollback**:
   - Before applying configuration: run `nginx -t -p <prefix> -c <conf>`.
   - If exit code != 0: roll back configuration and surface error log to developer.
   - If exit code == 0: trigger graceful reload via `nginx -s reload`.

---

## 4. Local Certificate Authority (SSL)

1. **Root CA**:
   - Key: 2048-bit RSA generated once in `%LOCALAPPDATA%\Dotnet\ssl\rootCA.key`.
   - Certificate: `CN=Dotnet Development CA`, validity: 10 years.
   - Automatically installed to Windows `ROOT` (Trusted Root Certification Authorities) certificate store.
2. **Domain Certificates**:
   - Generates SAN certificates for `<domain>.test` and `*.<domain>.test`.
   - Signed by the Root CA; validity: 2 years.

---

## 5. Dual-Mode Services & Process Tracking

1. **Service Modes**:
   - **Portable Process**: Managed directly (e.g. `mysqld --standalone`, `postgres -D`).
   - **Windows Service**: Managed via Windows Service Control Manager (`sc.exe` or Win32 API).
2. **Crash & State Tracking**:
   - Active processes tracked with `{serviceId, pid, startTime}` persisted to `%LOCALAPPDATA%\Dotnet\running-processes.json`.
   - On application startup, re-adopts existing processes by verifying matching PID, executable path, and start time.
