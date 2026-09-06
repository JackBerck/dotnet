# Prompt — Native Windows Standalone Development Manager

## Role

Act as a senior Windows desktop application architect and developer.

Design and build a simple, native-feeling Windows desktop application that centrally manages a standalone local development environment.

The app replaces the **operational convenience** of Laragon, not Laragon's entire feature set.

## Existing Environment

PHP:
```text
C:\tools\php85
PHP 8.5.10
Config: C:\tools\php85\php.ini
FastCGI: C:\tools\php85\php-cgi.exe
FastCGI: 127.0.0.1:9000
```

Nginx:
```text
C:\tools\nginx
Config: C:\tools\nginx\conf\nginx.conf
Sites: C:\tools\nginx\conf\sites\
Port: 80
```

MySQL:
```text
MySQL 8.4.11
Service: MySQL84
Port: 3306
```

PostgreSQL:
```text
PostgreSQL 17.11
Service: postgresql-x64-17
Port: 5432
```

Redis:
```text
Memurai
Port: 6379
CLI: memurai-cli
```

Node/NVM/npm and Bun are already installed.

Laravel development workflow remains:

```bash
composer run dev
```

Nginx is optional and should not replace this workflow.

## Main Goal

The application must make these operations simple:

1. Start/stop development services.
2. View service health.
3. Detect port conflicts.
4. Check Docker Compose port conflicts before Docker starts.
5. Manage selected PHP configuration.
6. Manage Windows hosts entries.
7. Validate/reload Nginx.
8. Manage Laravel project shortcuts.
9. Centralize operational configuration.
10. Provide safe diagnostics.

## Dashboard

Show:

```text
MySQL
PostgreSQL
Redis/Memurai
Nginx
PHP FastCGI
```

Each should show:

```text
RUNNING
STOPPED
ERROR
UNKNOWN
```

and relevant port.

Provide Start, Stop, Restart where appropriate.

## Start All / Stop All

Allow configurable groups.

Default standalone group:

```text
MySQL84
postgresql-x64-17
Memurai
Nginx
PHP FastCGI
```

Nginx/PHP FastCGI must be independently toggleable because normal Laravel development uses `composer run dev`.

## Service Management

Use Windows Service APIs where practical.

For:
- MySQL84
- postgresql-x64-17
- Memurai

Do not depend solely on parsing command-line output.

For PHP FastCGI and Nginx, track managed processes and PIDs reliably.

## Port Conflict Detection

Before starting a service or Docker:

- inspect requested host ports;
- identify current owner;
- identify service/process;
- show a clear explanation.

Example:

```text
Port conflict detected

Port: 3306
Current owner: MySQL84
Docker wants: 3306:3306

[ Stop MySQL84 & Continue ]
[ Change Docker Port ]
[ Cancel ]
```

Never force-kill arbitrary processes.

## Docker Integration

Before Docker Compose startup:

1. Resolve effective Compose configuration where practical.
2. Parse host port mappings.
3. Compare against active ports.
4. Identify owners.
5. Offer safe actions.

The app should support a workflow such as:

```text
Standalone
   ↓
Check Docker ports
   ↓
No conflict → start Docker

Conflict → explain
          → stop service / change port / cancel
```

Never silently stop services.

## PHP Config Manager

Manage:

```text
C:\tools\php85\php.ini
```

Useful settings:

- memory_limit
- upload_max_filesize
- post_max_size
- max_execution_time
- date.timezone
- extension enable/disable

Before writing:
- backup;
- validate;
- preserve comments where practical;
- show what changed;
- never silently corrupt the file.

## Hosts Manager

Manage:

```text
C:\Windows\System32\drivers\etc\hosts
```

Support entries such as:

```text
127.0.0.1 mrebet.test
127.0.0.1 redhub.test
127.0.0.1 rembugkop.test
```

Features:
- add;
- edit;
- remove;
- enable/disable;
- duplicate detection;
- backup;
- Administrator elevation only when necessary.

## Nginx Manager

Manage:

```text
C:\tools\nginx\conf\nginx.conf
C:\tools\nginx\conf\sites\
```

Actions:
- start;
- stop;
- reload;
- validate config;
- open config;
- view recent error log.

Before reload:

```powershell
nginx.exe -t
```

If validation fails, do not reload.

## PHP FastCGI Manager

Run:

```text
php-cgi.exe -b 127.0.0.1:9000
```

The app should:
- launch it in background;
- track PID;
- detect unexpected exit;
- stop it safely;
- detect port 9000 conflict.

Do not assume `php-fpm.exe` exists on Windows.

## Project Manager

Allow registration:

```text
Name
Path
PHP version
Database type
Database name
Redis required
Development command
Nginx host
```

Example:

```text
mrebet
D:\Code\mrebet
PHP 8.5
MySQL
Redis: yes
composer run dev
mrebet.test
```

Actions:
- Open project;
- Open terminal;
- Run development command;
- Open browser;
- Open `.env`;
- Open Nginx config.

Do not silently modify project files.

## Configuration Profiles

At minimum:

```text
Standalone
Docker
```

Standalone can own:
- MySQL;
- PostgreSQL;
- Redis;
- optional Nginx;
- optional PHP FastCGI.

Docker profile should check and resolve conflicts before starting.

## Logging

Provide concise operation logs:

```text
09:12:03 Started MySQL84
09:12:04 Started PostgreSQL
09:12:04 Started Memurai
09:12:05 Started PHP FastCGI :9000
09:12:06 Nginx config valid
09:12:06 Started Nginx
```

Never log passwords or secrets.

## Safety

The application manages infrastructure, so:

- never silently delete files;
- never silently kill processes;
- backup before config changes;
- never store database passwords plaintext;
- never automatically uninstall Laragon;
- never automatically clean PATH;
- never modify firewall rules silently;
- use elevation only when required;
- make destructive actions explicit.

## UX

Keep the application small and developer-focused.

Primary questions the UI should answer:

```text
Are my services running?
What is using this port?
Can I safely start Docker?
What config changed?
Can I start/stop my development environment?
Can I launch my Laravel project?
```

Avoid unnecessary monitoring dashboards.

## Suggested Architecture

```text
UI
│
├── Service Manager
├── Process Manager
├── Port Manager
├── Docker Manager
├── PHP Config Manager
├── Hosts Manager
├── Nginx Manager
├── Project Manager
└── Logging
```

Keep modules independent so additional PHP versions, projects, Docker profiles, and tools can be added later.

## MVP

The first version is successful when the user can:

1. See MySQL/PostgreSQL/Redis/Nginx/PHP FastCGI status.
2. Start/stop them.
3. Detect port conflicts.
4. Check Docker port conflicts.
5. Start/stop standalone mode safely.
6. Change selected PHP settings.
7. Manage hosts entries.
8. Validate/reload Nginx.
9. Register Laravel projects.
10. Launch `composer run dev`.

Do not overbuild the first version.
