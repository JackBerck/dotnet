# Project Dependencies Audit

This document tracks all external NuGet packages, runtime components, and tools used by **Dotnet (DevManager)** in accordance with Section 15.2 of `PROJECT-SPEC.md`.

## NuGet Dependencies

| Package | Version | Used By | License | Purpose |
|---|---|---|---|---|
| `System.ServiceProcess.ServiceController` | 10.0.0 | `Dotnet.Core` | MIT | Controls Windows Services (`MySQL84`, `postgresql`, `Memurai`) without invoking `net.exe` or `sc.exe`. |
| `xunit` | 2.9.3 | `Dotnet.Core.Tests` | Apache-2.0 | Unit test runner and assertions. |
| `xunit.runner.visualstudio` | 3.0.2 | `Dotnet.Core.Tests` | Apache-2.0 | Visual Studio / `dotnet test` execution adapter. |
| `Microsoft.NET.Test.Sdk` | 17.13.0 | `Dotnet.Core.Tests` | MIT | .NET test execution platform. |

## Managed External Tools (Catalog)

All developer runtimes, servers, and tools are downloaded on demand directly from their official upstream distributions. Dotnet DevManager does not redistribute these binaries:

| Tool | Upstream Source | License | Integration Method |
|---|---|---|---|
| **PHP** | `windows.php.net` | PHP License 3.01 | Standalone NTS zip + FastCGI runner |
| **Node.js** | `nodejs.org` | MIT | Standalone win-x64 zip + npm |
| **MinGit** | `git-for-windows` | GPL-2.0 / LGPL-2.1 | Standalone MinGit zip archive |
| **Nginx** | `nginx.org` | 2-clause BSD | Standalone win-x64 zip |
| **Composer** | `getcomposer.org` | MIT | Single-file PHAR + wrapper batch shim |
| **Bun** | `github.com/oven-sh/bun` | MIT | Standalone zip with root-stripping |
| **Go** | `go.dev` | BSD-3-Clause | Standalone win-x64 zip |
| **MySQL** | `dev.mysql.com` | GPL-2.0 | Portable win-x64 zip + `--initialize-insecure` |
| **PostgreSQL** | `get.enterprisedb.com` | PostgreSQL License | Portable binaries zip + `initdb` |
| **Memurai (Redis)** | `memurai.com` | Proprietary Developer License | Windows Service adoption / portable |

## Native P/Invoke References

- **iphlpapi.dll:** `GetExtendedTcpTable` (IPv4 & IPv6 TCP snapshot).
- **user32.dll:** `SendMessageTimeout` (broadcasting `WM_SETTINGCHANGE` on environment updates) and `PostMessage` (single instance restore).
- **advapi32.dll:** Windows Service controller query support.
