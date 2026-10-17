//! Embedded HTML/CSS/JS Single-Page Dashboard for Dotnet Desktop Application.

pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Dotnet</title>
  <link rel="icon" type="image/x-icon" href="/favicon.ico">
  <link rel="icon" type="image/png" href="/api/app/icon.png">
  <style>
    :root {
      --bg: #0b0f19;
      --card-bg: #131c2e;
      --card-hover: #1a263d;
      --border: #23334d;
      --border-focus: #3b82f6;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
      --primary: #10b981;
      --primary-hover: #059669;
      --accent: #6366f1;
      --danger: #ef4444;
      --warning: #f59e0b;
      --shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
      background: var(--bg);
      color: var(--text);
      display: flex;
      height: 100vh;
      overflow: hidden;
    }
    /* Sidebar */
    .sidebar {
      width: 250px;
      background: #080c14;
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      padding: 1.25rem 0.75rem;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0 0.75rem 1.25rem;
      border-bottom: 1px solid var(--border);
      margin-bottom: 1rem;
    }
    .brand-logo {
      width: 32px;
      height: 32px;
      border-radius: 6px;
      object-fit: contain;
    }
    .brand-title { font-weight: 700; font-size: 1.15rem; letter-spacing: -0.02em; }
    .nav { display: flex; flex-direction: column; gap: 0.35rem; flex: 1; }
    .nav-btn {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.7rem 0.85rem;
      border-radius: 6px;
      background: transparent;
      color: var(--text-muted);
      border: none;
      font-size: 0.9rem;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s ease;
      text-align: left;
      width: 100%;
    }
    .nav-btn:hover { background: var(--card-bg); color: var(--text); }
    .nav-btn.active { background: #1e293b; color: #fff; border-left: 3px solid var(--primary); }
    .profile-switcher {
      padding: 0.75rem;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      margin-top: auto;
    }
    .profile-label { font-size: 0.75rem; color: var(--text-muted); margin-bottom: 0.4rem; font-weight: 600; }
    .profile-select {
      width: 100%;
      background: #0b0f19;
      border: 1px solid var(--border);
      color: var(--text);
      padding: 0.4rem 0.6rem;
      border-radius: 6px;
      font-size: 0.85rem;
      outline: none;
      cursor: pointer;
    }
    /* Main Content */
    .main {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow-y: auto;
    }
    .header {
      padding: 1.25rem 2rem;
      border-bottom: 1px solid var(--border);
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: rgba(11, 15, 25, 0.8);
      backdrop-filter: blur(8px);
      position: sticky;
      top: 0;
      z-index: 10;
    }
    .header-title { font-size: 1.4rem; font-weight: 700; letter-spacing: -0.02em; }
    .actions { display: flex; gap: 0.75rem; }
    .btn {
      padding: 0.5rem 1rem;
      border-radius: 6px;
      font-size: 0.85rem;
      font-weight: 600;
      border: none;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      transition: all 0.15s;
    }
    .btn-primary { background: var(--primary); color: #fff; }
    .btn-primary:hover { background: var(--primary-hover); }
    .btn-danger { background: var(--danger); color: #fff; }
    .btn-danger:hover { background: #dc2626; }
    .btn-secondary { background: var(--card-bg); color: var(--text); border: 1px solid var(--border); }
    .btn-secondary:hover { background: var(--card-hover); }
    .btn-sm { padding: 0.35rem 0.65rem; font-size: 0.75rem; }
    .content { padding: 2rem; display: flex; flex-direction: column; gap: 1.5rem; flex: 1; }
    .tab-pane { display: none; flex-direction: column; gap: 1.5rem; }
    .tab-pane.active { display: flex; }
    /* Cards & Grids */
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.25rem; }
    .card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 1.25rem;
      display: flex;
      flex-direction: column;
      gap: 0.85rem;
      box-shadow: var(--shadow);
      transition: border 0.15s ease;
    }
    .card:hover { border-color: #334e68; }
    .card-header { display: flex; justify-content: space-between; align-items: center; }
    .card-title { font-weight: 600; font-size: 1rem; display: flex; align-items: center; gap: 0.5rem; }
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      padding: 0.2rem 0.55rem;
      border-radius: 9999px;
      font-size: 0.7rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }
    .badge-running { background: rgba(16, 185, 129, 0.15); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); }
    .badge-stopped { background: rgba(148, 163, 184, 0.1); color: #94a3b8; border: 1px solid rgba(148, 163, 184, 0.2); }
    .badge-conflict { background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }
    .badge-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
    .card-meta { display: flex; flex-direction: column; gap: 0.35rem; font-size: 0.82rem; color: var(--text-muted); }
    .card-meta-row { display: flex; justify-content: space-between; }
    .card-actions { display: flex; gap: 0.5rem; margin-top: auto; padding-top: 0.75rem; border-top: 1px solid var(--border); }
    /* Tables */
    .table-container {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 10px;
      overflow: hidden;
      box-shadow: var(--shadow);
    }
    table { width: 100%; border-collapse: collapse; text-align: left; font-size: 0.85rem; }
    th {
      background: #0f172a;
      padding: 0.75rem 1rem;
      font-weight: 600;
      color: var(--text-muted);
      border-bottom: 1px solid var(--border);
    }
    td { padding: 0.85rem 1rem; border-bottom: 1px solid var(--border); }
    tr:last-child td { border-bottom: none; }
    tr:hover td { background: var(--card-hover); }
    .code-tag {
      font-family: monospace;
      background: #0b0f19;
      padding: 0.2rem 0.4rem;
      border-radius: 4px;
      border: 1px solid var(--border);
      color: #38bdf8;
    }
    /* Toast */
    #toast {
      position: fixed;
      bottom: 1.5rem;
      right: 1.5rem;
      background: #1e293b;
      color: #fff;
      padding: 0.75rem 1.25rem;
      border-radius: 8px;
      border-left: 4px solid var(--primary);
      box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
      font-size: 0.85rem;
      font-weight: 500;
      display: none;
      z-index: 1000;
      animation: fadeIn 0.2s ease;
    }
    @keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

    /* Modal */
    .modal-backdrop {
      display: none;
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0, 0, 0, 0.75);
      backdrop-filter: blur(4px);
      z-index: 500;
      align-items: center;
      justify-content: center;
    }
    .modal-backdrop.active { display: flex; }
    .modal-content {
      background: #111827;
      border: 1px solid var(--border);
      border-radius: 12px;
      width: 90%;
      max-width: 650px;
      max-height: 90vh;
      display: flex;
      flex-direction: column;
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
      overflow: hidden;
      animation: modalIn 0.15s ease-out;
    }
    @keyframes modalIn { from { opacity: 0; transform: scale(0.96); } to { opacity: 1; transform: scale(1); } }
    .modal-header {
      padding: 1.25rem 1.5rem;
      border-bottom: 1px solid var(--border);
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: #0b0f19;
    }
    .modal-title { font-size: 1.1rem; font-weight: 700; color: #fff; }
    .modal-body {
      padding: 1.5rem;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    .modal-footer {
      padding: 1rem 1.5rem;
      border-top: 1px solid var(--border);
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      background: #0b0f19;
    }
    .form-group {
      display: flex;
      flex-direction: column;
      gap: 0.4rem;
    }
    .form-label {
      font-size: 0.82rem;
      font-weight: 600;
      color: var(--text-muted);
    }
    .form-input, .form-select, .form-textarea {
      width: 100%;
      background: #080c14;
      border: 1px solid var(--border);
      color: var(--text);
      padding: 0.6rem 0.75rem;
      border-radius: 6px;
      font-size: 0.88rem;
      outline: none;
      transition: border 0.15s ease;
    }
    .form-input:focus, .form-select:focus, .form-textarea:focus {
      border-color: var(--border-focus);
    }
    .form-textarea {
      font-family: Consolas, 'Fira Code', monospace;
      font-size: 0.82rem;
      line-height: 1.5;
      min-height: 320px;
      resize: vertical;
    }
    .subtab-btn.active {
      background: #1e293b;
      color: #fff;
      border-color: var(--primary);
    }
  </style>
</head>
<body>
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">
      <img class="brand-logo" src="/api/app/icon.png" alt="Dotnet" />
      <div class="brand-title">Dotnet</div>
    </div>
    <nav class="nav">
      <button class="nav-btn active" onclick="switchTab('services')">🖥️ Services</button>
      <button class="nav-btn" onclick="switchTab('projects')">📁 Projects</button>
      <button class="nav-btn" onclick="switchTab('tools')">📦 Tools Catalog</button>
      <button class="nav-btn" onclick="switchTab('hosts')">🌐 Hosts & Sites</button>
      <button class="nav-btn" onclick="switchTab('php')">⚙️ PHP & Config</button>
      <button class="nav-btn" onclick="switchTab('diagnostics')">🔍 Diagnostics</button>
      <button class="nav-btn" onclick="switchTab('logs')">📋 Logs</button>
    </nav>
    <div class="profile-switcher">
      <div class="profile-label">ACTIVE PROFILE</div>
      <select id="profile-select" class="profile-select" onchange="changeProfile(this.value)">
        <option value="standalone">Local</option>
        <option value="docker">Docker</option>
      </select>
    </div>
  </aside>

  <!-- Main View -->
  <main class="main">
    <header class="header">
      <div class="header-title" id="page-title">Services Dashboard</div>
      <div class="actions">
        <button class="btn btn-secondary btn-sm" onclick="reloadNginx()">🔄 Reload Nginx</button>
        <button class="btn btn-primary btn-sm" onclick="startAllServices()">🟢 Start All</button>
        <button class="btn btn-danger btn-sm" onclick="stopAllServices()">🛑 Stop All</button>
      </div>
    </header>

    <div class="content">
      <!-- Tab 1: Services -->
      <section id="pane-services" class="tab-pane active">
        <div class="grid" id="services-grid">
          <!-- Rendered dynamically -->
        </div>
      </section>

      <!-- Tab 2: Projects -->
      <section id="pane-projects" class="tab-pane">
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <h3>Local Projects</h3>
          <button class="btn btn-primary btn-sm" onclick="promptAddProject()">+ Add Project</button>
        </div>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>Project Name</th>
                <th>Framework</th>
                <th>Local Domain</th>
                <th>SSL</th>
                <th>Directory</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody id="projects-table-body">
              <!-- Rendered dynamically -->
            </tbody>
          </table>
        </div>
      </section>

      <!-- Tab 3: Tools Catalog -->
      <section id="pane-tools" class="tab-pane">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:1rem;">
          <div>
            <h3>Installed Developer Tools & Runtimes</h3>
            <p style="color:var(--text-muted); font-size:0.8rem; margin-top:0.2rem;">
              All developer tools and runtimes detected on your Windows system are automatically synchronized into the Dotnet environment.
            </p>
          </div>
          <button class="btn btn-secondary btn-sm" onclick="scanAdoptTools()">🔍 Rescan System Tools</button>
        </div>
        <div class="grid" id="tools-grid">
          <!-- Rendered dynamically -->
        </div>
      </section>

      <!-- Tab 4: Hosts & Sites -->
      <section id="pane-hosts" class="tab-pane">
        <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:0.5rem;">
          <div>
            <h3>Windows Hosts File & Local Sites</h3>
            <p style="color:var(--text-muted); font-size:0.8rem; margin-top:0.2rem;">
              Manage domain resolutions in <code>C:\Windows\System32\drivers\etc\hosts</code>. All changes take effect immediately.
            </p>
          </div>
          <div style="display:flex; gap:0.5rem;">
            <button class="btn btn-secondary btn-sm" onclick="loadHosts()">🔄 Refresh</button>
            <button class="btn btn-secondary btn-sm" onclick="openRawHostsModal()">✏️ Edit Raw Hosts File</button>
            <button class="btn btn-primary btn-sm" onclick="openAddHostModal()">+ Add Host</button>
          </div>
        </div>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>IP Address</th>
                <th>Hostname / Domain</th>
                <th>Status</th>
                <th>Category</th>
                <th style="text-align:right;">Actions</th>
              </tr>
            </thead>
            <tbody id="hosts-table-body">
              <!-- Rendered dynamically -->
            </tbody>
          </table>
        </div>
      </section>

      <!-- Tab 5: PHP & Config -->
      <section id="pane-php" class="tab-pane">
        <!-- PHP Runtime Banner Card -->
        <div class="card" style="padding:1.25rem; background:linear-gradient(135deg, #131c2e, #1a263d);">
          <div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:1rem;">
            <div>
              <div style="display:flex; align-items:center; gap:0.6rem;">
                <span class="badge badge-running" id="php-status-badge">PHP ACTIVE</span>
                <h3 id="php-runtime-ver" style="margin:0; font-size:1.15rem;">Loading PHP Environment...</h3>
              </div>
              <div style="color:var(--text-muted); font-size:0.82rem; margin-top:0.4rem; display:flex; flex-direction:column; gap:0.2rem;">
                <div><span style="color:#94a3b8; font-weight:600;">Directory:</span> <span class="code-tag" id="php-runtime-dir">-</span></div>
                <div><span style="color:#94a3b8; font-weight:600;">Configuration File:</span> <span class="code-tag" id="php-runtime-ini">-</span></div>
              </div>
            </div>
            <div style="display:flex; gap:0.5rem; flex-wrap:wrap;">
              <button class="btn btn-secondary btn-sm" onclick="applyPhpPreset('development')">Apply Dev Preset</button>
              <button class="btn btn-secondary btn-sm" onclick="applyPhpPreset('production')">Apply Prod Preset</button>
              <button class="btn btn-primary btn-sm" onclick="openRawIniModal()">✏️ Edit php.ini Directly</button>
            </div>
          </div>
        </div>

        <!-- Subtabs: Settings / Extensions -->
        <div style="display:flex; gap:0.5rem; border-bottom:1px solid var(--border); padding-bottom:0.5rem;">
          <button class="btn btn-secondary btn-sm subtab-btn active" id="btn-subtab-php-config" onclick="switchPhpSubTab('config')">⚙️ Core Directives</button>
          <button class="btn btn-secondary btn-sm subtab-btn" id="btn-subtab-php-ext" onclick="switchPhpSubTab('extensions')">🧩 PHP Extensions (<span id="php-ext-count">0</span>)</button>
        </div>

        <!-- Section 1: Core Configuration Settings -->
        <div id="php-subtab-config" style="display:flex; flex-direction:column; gap:1.25rem;">
          <div class="grid" id="php-settings-grid">
            <!-- Rendered dynamically -->
          </div>
          <div style="display:flex; justify-content:flex-end; gap:0.75rem;">
            <button class="btn btn-secondary btn-sm" onclick="loadPhpConfig()">Reset Form</button>
            <button class="btn btn-primary" onclick="savePhpSettingsForm()">💾 Save All PHP Directives</button>
          </div>
        </div>

        <!-- Section 2: Extensions Table -->
        <div id="php-subtab-extensions" style="display:none; flex-direction:column; gap:1rem;">
          <div style="display:flex; justify-content:space-between; align-items:center;">
            <input type="text" id="php-ext-search" class="form-input" style="max-width:320px;" placeholder="Search extension (e.g. curl, mbstring, pdo)..." oninput="filterPhpExtensions(this.value)">
            <button class="btn btn-secondary btn-sm" onclick="loadPhpExtensions()">🔄 Refresh Extensions</button>
          </div>
          <div class="table-container">
            <table>
              <thead>
                <tr>
                  <th>Extension Name</th>
                  <th>File Name</th>
                  <th>Status</th>
                  <th style="text-align:right;">Action</th>
                </tr>
              </thead>
              <tbody id="php-ext-table-body">
                <!-- Rendered dynamically -->
              </tbody>
            </table>
          </div>
        </div>
      </section>


      <!-- Tab 6: Diagnostics -->
      <section id="pane-diagnostics" class="tab-pane">
        <h3>Active TCP Listeners</h3>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>Port</th>
                <th>PID</th>
                <th>Local Address</th>
              </tr>
            </thead>
            <tbody id="ports-table-body">
              <!-- Rendered dynamically -->
            </tbody>
          </table>
        </div>

        <h3 style="margin-top:1.5rem;">PATH Shadow Analyzer</h3>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>Binary Name</th>
                <th>User PATH</th>
                <th>System PATH</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody id="shadow-table-body">
              <!-- Rendered dynamically -->
            </tbody>
          </table>
        </div>
      </section>

      <!-- Tab 7: Logs -->
      <section id="pane-logs" class="tab-pane">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.75rem;">
          <div>
            <h3>Live Process & Service Logs</h3>
            <p style="color:var(--text-muted); font-size:0.8rem; margin-top:0.2rem;">
              Real-time lifecycle tracking for service processes, TCP port listeners, UAC elevation, and error diagnostics.
            </p>
          </div>
          <div style="display:flex; gap:0.5rem;">
            <button class="btn btn-secondary btn-sm" onclick="loadLogs()">🔄 Refresh Logs</button>
            <button class="btn btn-secondary btn-sm" onclick="clearLogs()">🧹 Clear Logs</button>
          </div>
        </div>
        <div class="table-container" style="padding:1rem; background:#050811; font-family:Consolas, 'Fira Code', monospace; font-size:0.82rem; height:520px; overflow-y:auto; border:1px solid var(--border);" id="log-box">
          <div style="color:var(--text-muted);">Loading log history...</div>
        </div>
      </section>
    </div>
  </main>

  <!-- Modal: Add / Edit Host -->
  <div class="modal-backdrop" id="modal-host">
    <div class="modal-content">
      <div class="modal-header">
        <div class="modal-title" id="modal-host-title">Add Virtual Host Entry</div>
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-host')">✕</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">IP Address</label>
          <input type="text" id="modal-host-ip" class="form-input" value="127.0.0.1" placeholder="127.0.0.1">
          <span style="font-size:0.75rem; color:var(--text-muted);">Use 127.0.0.1 for local projects or virtual hosts.</span>
        </div>
        <div class="form-group">
          <label class="form-label">Hostname / Domain</label>
          <input type="text" id="modal-host-name" class="form-input" placeholder="e.g. myproject.test, api.local">
          <span style="font-size:0.75rem; color:var(--text-muted);">RFC 1123 compliant domain name.</span>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-host')">Cancel</button>
        <button class="btn btn-primary btn-sm" onclick="submitHostEntry()">Save Host Entry</button>
      </div>
    </div>
  </div>

  <!-- Modal: Raw Hosts Editor -->
  <div class="modal-backdrop" id="modal-raw-hosts">
    <div class="modal-content" style="max-width:800px;">
      <div class="modal-header">
        <div class="modal-title">Edit Windows Hosts File (Raw)</div>
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-raw-hosts')">✕</button>
      </div>
      <div class="modal-body">
        <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.25rem;">
          Editing <code>C:\Windows\System32\drivers\etc\hosts</code> directly. Managed block entries are kept intact.
        </div>
        <textarea id="raw-hosts-textarea" class="form-textarea" spellcheck="false" style="height:360px;"></textarea>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-raw-hosts')">Cancel</button>
        <button class="btn btn-primary btn-sm" onclick="saveRawHosts()">💾 Save Hosts File</button>
      </div>
    </div>
  </div>

  <!-- Modal: Raw php.ini Editor -->
  <div class="modal-backdrop" id="modal-raw-ini">
    <div class="modal-content" style="max-width:850px;">
      <div class="modal-header">
        <div class="modal-title">Edit php.ini Configuration</div>
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-raw-ini')">✕</button>
      </div>
      <div class="modal-body">
        <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.25rem;">
          Direct editing of active <code>php.ini</code>. An automated backup is created before saving.
        </div>
        <textarea id="raw-ini-textarea" class="form-textarea" spellcheck="false" style="height:400px;"></textarea>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary btn-sm" onclick="closeModal('modal-raw-ini')">Cancel</button>
        <button class="btn btn-primary btn-sm" onclick="saveRawIni()">💾 Save php.ini</button>
      </div>
    </div>
  </div>

  <div id="toast">Operation completed</div>


  <script>
    const tabs = ['services', 'projects', 'tools', 'hosts', 'php', 'diagnostics', 'logs'];
    const titles = {
      services: 'Services Dashboard',
      projects: 'Managed Projects',
      tools: 'Tools Catalog',
      hosts: 'Hosts & Virtual Hosts',
      php: 'PHP Configuration',
      diagnostics: 'Diagnostics & Port Conflict Analyzer',
      logs: 'Live Event Logs'
    };

    function showToast(msg) {
      const t = document.getElementById('toast');
      t.innerText = msg;
      t.style.display = 'block';
      setTimeout(() => t.style.display = 'none', 3000);
    }

    function switchTab(id) {
      document.querySelectorAll('.nav-btn').forEach((b, idx) => {
        b.classList.toggle('active', tabs[idx] === id);
      });
      document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
      const active = document.getElementById('pane-' + id);
      if (active) active.classList.add('active');
      document.getElementById('page-title').innerText = titles[id] || 'Dashboard';
      refreshTab(id);
    }

    async function refreshTab(id) {
      if (id === 'services') await loadServices();
      else if (id === 'projects') await loadProjects();
      else if (id === 'tools') await loadTools();
      else if (id === 'hosts') await loadHosts();
      else if (id === 'php') await loadPhp();
      else if (id === 'diagnostics') await loadDiagnostics();
      else if (id === 'logs') await loadLogs();
    }

    async function loadServices() {
      try {
        const res = await fetch('/api/services');
        const data = await res.json();
        const grid = document.getElementById('services-grid');
        grid.innerHTML = data.map(s => `
          <div class="card">
            <div class="card-header">
              <div class="card-title">${s.name}</div>
              <span class="badge ${s.is_running ? 'badge-running' : 'badge-stopped'}">
                <span class="badge-dot"></span>
                ${s.is_running ? 'RUNNING' : 'STOPPED'}
              </span>
            </div>
            <div class="card-meta">
              <div class="card-meta-row"><span>Port:</span><span class="code-tag">${s.port || '-'}</span></div>
              <div class="card-meta-row"><span>PID:</span><span>${s.pid || '-'}</span></div>
              <div class="card-meta-row"><span>Type:</span><span>${s.type}</span></div>
            </div>
            <div class="card-actions">
              ${s.is_running
                ? `<button class="btn btn-danger btn-sm" onclick="serviceAction('${s.id}', 'stop')">Stop</button>`
                : `<button class="btn btn-primary btn-sm" onclick="serviceAction('${s.id}', 'start')">Start</button>`
              }
              <button class="btn btn-secondary btn-sm" onclick="serviceAction('${s.id}', 'restart')">Restart</button>
            </div>
          </div>
        `).join('');
      } catch (e) { console.error(e); }
    }

    async function serviceAction(id, action) {
      showToast(`Sending ${action} command for ${id}...`);
      try {
        const res = await fetch(`/api/services/action?id=${id}&action=${action}`, { method: 'POST' });
        const json = await res.json().catch(() => ({}));
        if (res.ok) {
          showToast(json.message || `Service ${id} ${action} succeeded`);
        } else {
          showToast(`Error: ${json.error || 'Operation failed'}`);
        }
      } catch (err) {
        showToast(`Failed to connect to server: ${err}`);
      }
      setTimeout(loadServices, 500);
      loadLogs();
    }

    async function startAllServices() {
      showToast('Starting all services...');
      try {
        const res = await fetch('/api/services/start-all', { method: 'POST' });
        const json = await res.json().catch(() => ({}));
        if (res.ok) {
          showToast(json.message || 'All services started successfully');
        } else {
          showToast(`Error: ${json.error || 'Operation failed'}`);
        }
      } catch (err) {
        showToast(`Error: ${err}`);
      }
      setTimeout(loadServices, 1000);
      loadLogs();
    }

    async function stopAllServices() {
      showToast('Stopping all services...');
      try {
        const res = await fetch('/api/services/stop-all', { method: 'POST' });
        const json = await res.json().catch(() => ({}));
        if (res.ok) {
          showToast(json.message || 'All services stopped');
        } else {
          showToast(`Error: ${json.error || 'Operation failed'}`);
        }
      } catch (err) {
        showToast(`Error: ${err}`);
      }
      setTimeout(loadServices, 1000);
      loadLogs();
    }

    async function reloadNginx() {
      const res = await fetch('/api/nginx/reload', { method: 'POST' });
      const data = await res.json();
      showToast(data.message || 'Nginx reloaded');
    }

    async function changeProfile(id) {
      await fetch(`/api/profile/switch?id=${id}`, { method: 'POST' });
      showToast(`Switched active profile to ${id}`);
      setTimeout(loadServices, 500);
    }

    async function loadProjects() {
      const res = await fetch('/api/projects');
      const data = await res.json();
      const tbody = document.getElementById('projects-table-body');
      tbody.innerHTML = data.map(p => `
        <tr>
          <td><strong>${p.Name}</strong></td>
          <td><span class="code-tag">${p.Framework}</span></td>
          <td><a href="http://${p.NginxHost || 'localhost'}" target="_blank" style="color:#38bdf8; text-decoration:none;">${p.NginxHost || '-'}</a></td>
          <td>${p.SslEnabled ? '🔒 HTTPS' : 'HTTP'}</td>
          <td style="color:#94a3b8; font-size:0.8rem;">${p.Path}</td>
          <td>
            <button class="btn btn-secondary btn-sm" onclick="launchProject('${p.Id}', 'run')">Run</button>
            <button class="btn btn-secondary btn-sm" onclick="launchProject('${p.Id}', 'explorer')">Folder</button>
            <button class="btn btn-secondary btn-sm" onclick="launchProject('${p.Id}', 'code')">VSCode</button>
            <button class="btn btn-danger btn-sm" onclick="deleteProject('${p.Id}')">Delete</button>
          </td>
        </tr>
      `).join('');
    }

    async function launchProject(id, action) {
      await fetch(`/api/projects/launch?id=${id}&action=${action}`, { method: 'POST' });
      showToast(`Launched ${action} for project`);
    }

    async function deleteProject(id) {
      if (confirm('Delete this project?')) {
        await fetch(`/api/projects/delete?id=${id}`, { method: 'POST' });
        showToast('Project deleted');
        loadProjects();
      }
    }

    async function promptAddProject() {
      const path = prompt('Enter project directory path:');
      if (path) {
        await fetch('/api/projects/add', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ path: path })
        });
        showToast('Project registered');
        loadProjects();
      }
    }

    async function scanAdoptTools() {
      showToast('Scanning developer tools on Windows...');
      try {
        const res = await fetch('/api/tools/scan', { method: 'POST' });
        const data = await res.json();
        showToast(data.message || 'Tools scan completed!');
      } catch (e) {
        showToast('Failed to scan tools');
      }
      setTimeout(loadTools, 500);
    }

    async function loadTools() {
      const res = await fetch('/api/tools');
      const data = await res.json();
      const grid = document.getElementById('tools-grid');
      grid.innerHTML = data.map(t => `
        <div class="card">
          <div class="card-header">
            <div class="card-title">${t.display_name}</div>
            <span class="badge ${t.is_installed ? 'badge-running' : 'badge-stopped'}">
              <span class="badge-dot"></span>
              ${t.is_installed ? 'v' + t.installed_version : 'NOT INSTALLED'}
            </span>
          </div>
          <div class="card-meta">
            <div class="card-meta-row"><span>Category:</span><span>${t.category}</span></div>
            <div class="card-meta-row"><span>ID:</span><span class="code-tag">${t.id}</span></div>
            ${t.is_installed ? '<div class="card-meta-row"><span style="color:#10b981; font-weight:600;">✓ Installed on System</span></div>' : ''}
          </div>
          <div class="card-actions">
            ${t.is_installed
              ? `<button class="btn btn-secondary btn-sm" style="width:100%; justify-content:center;" disabled>✓ Ready to Use</button>`
              : `<button class="btn btn-primary btn-sm" style="width:100%; justify-content:center;" onclick="installTool('${t.id}')">Install Tool</button>`
            }
          </div>
        </div>
      `).join('');
    }

    async function installTool(id) {
      showToast(`Installing ${id}...`);
      await fetch(`/api/tools/install?id=${id}`, { method: 'POST' });
      showToast(`Installed ${id}`);
      loadTools();
    }

    async function uninstallTool(id) {
      if (confirm(`Uninstall ${id}?`)) {
        await fetch(`/api/tools/uninstall?id=${id}`, { method: 'POST' });
        showToast(`Uninstalled ${id}`);
        loadTools();
      }
    }

    // Modal Helpers
    function openModal(id) {
      const el = document.getElementById(id);
      if (el) el.classList.add('active');
    }
    function closeModal(id) {
      const el = document.getElementById(id);
      if (el) el.classList.remove('active');
    }

    // Hosts Management
    async function loadHosts() {
      try {
        const res = await fetch('/api/hosts');
        const data = await res.json();
        const tbody = document.getElementById('hosts-table-body');
        if (!data || data.length === 0) {
          tbody.innerHTML = '<tr><td colspan="5" style="text-align:center; color:var(--text-muted); padding:2rem;">No hosts entries found. Click "+ Add Host" to create one.</td></tr>';
          return;
        }
        tbody.innerHTML = data.map(h => `
          <tr>
            <td><span class="code-tag">${h.ip}</span></td>
            <td><strong>${h.hostname}</strong></td>
            <td>
              <span class="badge ${h.is_enabled ? 'badge-running' : 'badge-stopped'}">
                ${h.is_enabled ? 'Active' : 'Disabled'}
              </span>
            </td>
            <td>${h.is_managed ? '<span style="color:#10b981; font-weight:600;">Dotnet Managed</span>' : '<span style="color:#94a3b8;">System</span>'}</td>
            <td style="text-align:right;">
              <div style="display:inline-flex; gap:0.4rem; justify-content:flex-end;">
                <button class="btn btn-secondary btn-sm" onclick="toggleHost('${h.hostname}', ${!h.is_enabled})">
                  ${h.is_enabled ? 'Disable' : 'Enable'}
                </button>
                <button class="btn btn-secondary btn-sm" onclick="openEditHostModal('${h.ip}', '${h.hostname}')">Edit</button>
                <button class="btn btn-danger btn-sm" onclick="deleteHost('${h.hostname}')">Delete</button>
              </div>
            </td>
          </tr>
        `).join('');
      } catch (e) {
        showToast('Failed to load hosts entries');
      }
    }

    function openAddHostModal() {
      document.getElementById('modal-host-title').innerText = 'Add Virtual Host Entry';
      document.getElementById('modal-host-ip').value = '127.0.0.1';
      document.getElementById('modal-host-name').value = '';
      openModal('modal-host');
    }

    function openEditHostModal(ip, hostname) {
      document.getElementById('modal-host-title').innerText = 'Edit Virtual Host Entry';
      document.getElementById('modal-host-ip').value = ip;
      document.getElementById('modal-host-name').value = hostname;
      openModal('modal-host');
    }

    async function submitHostEntry() {
      const ip = document.getElementById('modal-host-ip').value.trim();
      const hostname = document.getElementById('modal-host-name').value.trim();
      if (!hostname) {
        alert('Hostname / Domain is required (e.g. myproject.test)');
        return;
      }
      try {
        const res = await fetch('/api/hosts/add', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ ip, hostname })
        });
        const data = await res.json();
        if (res.ok) {
          showToast(data.message || `Saved host ${hostname}`);
          closeModal('modal-host');
          loadHosts();
        } else {
          alert(data.message || 'Error saving host');
        }
      } catch (e) {
        alert('Failed to connect to backend engine: ' + e);
      }
    }

    async function deleteHost(hostname) {
      if (!confirm(`Are you sure you want to remove host entry for '${hostname}'?`)) return;
      try {
        const res = await fetch(`/api/hosts/delete?hostname=${encodeURIComponent(hostname)}`, { method: 'POST' });
        const data = await res.json();
        if (res.ok) {
          showToast(data.message || `Deleted ${hostname}`);
          loadHosts();
        } else {
          alert(data.message || 'Error deleting host');
        }
      } catch (e) {
        showToast('Error deleting host: ' + e);
      }
    }

    async function toggleHost(hostname, enable) {
      try {
        const res = await fetch(`/api/hosts/toggle?hostname=${encodeURIComponent(hostname)}&enable=${enable}`, { method: 'POST' });
        const data = await res.json();
        if (res.ok) {
          showToast(`Host '${hostname}' ${enable ? 'enabled' : 'disabled'}`);
          loadHosts();
        } else {
          alert(data.message || 'Error toggling host');
        }
      } catch (e) {
        showToast('Failed to toggle host');
      }
    }

    async function openRawHostsModal() {
      try {
        const res = await fetch('/api/hosts/raw');
        const data = await res.json();
        document.getElementById('raw-hosts-textarea').value = data.content || '';
        openModal('modal-raw-hosts');
      } catch (e) {
        alert('Failed to read hosts file: ' + e);
      }
    }

    async function saveRawHosts() {
      const content = document.getElementById('raw-hosts-textarea').value;
      try {
        const res = await fetch('/api/hosts/raw', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content })
        });
        const data = await res.json();
        if (res.ok) {
          showToast('Hosts file saved successfully');
          closeModal('modal-raw-hosts');
          loadHosts();
        } else {
          alert(data.message || 'Error saving hosts file');
        }
      } catch (e) {
        alert('Error saving hosts file: ' + e);
      }
    }

    // PHP & Configuration Management
    let allPhpExtensions = [];

    async function loadPhp() {
      await Promise.all([loadPhpInfo(), loadPhpConfig(), loadPhpExtensions()]);
    }

    async function loadPhpInfo() {
      try {
        const res = await fetch('/api/php/info');
        if (!res.ok) return;
        const data = await res.json();
        document.getElementById('php-runtime-ver').innerText = data.detected_version || 'PHP Environment';
        document.getElementById('php-runtime-dir').innerText = data.php_dir || '-';
        document.getElementById('php-runtime-ini').innerText = data.ini_path || '-';
        document.getElementById('php-status-badge').className = data.is_installed ? 'badge badge-running' : 'badge badge-stopped';
        document.getElementById('php-status-badge').innerText = data.is_installed ? 'PHP ACTIVE' : 'PHP NOT FOUND';
      } catch (e) {
        console.error('Error loading PHP info', e);
      }
    }

    async function loadPhpConfig() {
      try {
        const res = await fetch('/api/php/config');
        if (!res.ok) return;
        const items = await res.json();
        const grid = document.getElementById('php-settings-grid');
        grid.innerHTML = items.map(item => `
          <div class="card" style="padding:1rem;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.4rem;">
              <strong style="color:#f1f5f9; font-size:0.9rem;">${item.key}</strong>
              <span style="font-size:0.7rem; color:var(--text-muted); background:#0b0f19; padding:0.15rem 0.4rem; border-radius:4px; border:1px solid var(--border);">${item.category}</span>
            </div>
            <p style="font-size:0.75rem; color:var(--text-muted); margin-bottom:0.6rem;">${item.description}</p>
            <div class="form-group">
              ${item.options && item.options.length > 0 ? `
                <select class="form-select php-setting-input" data-key="${item.key}">
                  ${item.options.map(opt => `<option value="${opt}" ${opt === item.current_value ? 'selected' : ''}>${opt}</option>`).join('')}
                  ${!item.options.includes(item.current_value) ? `<option value="${item.current_value}" selected>${item.current_value} (custom)</option>` : ''}
                </select>
              ` : `
                <input type="text" class="form-input php-setting-input" data-key="${item.key}" value="${item.current_value}">
              `}
            </div>
          </div>
        `).join('');
      } catch (e) {
        console.error('Error loading PHP config directives', e);
      }
    }

    async function savePhpSettingsForm() {
      const inputs = document.querySelectorAll('.php-setting-input');
      const settings = {};
      inputs.forEach(input => {
        const k = input.getAttribute('data-key');
        if (k) settings[k] = input.value;
      });

      showToast('Saving PHP configuration directives...');
      try {
        const res = await fetch('/api/php/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ settings })
        });
        const data = await res.json();
        if (res.ok) {
          showToast(data.message || 'PHP directives saved to php.ini');
          loadPhpConfig();
        } else {
          alert(data.message || 'Error updating PHP settings');
        }
      } catch (e) {
        showToast('Failed to save settings: ' + e);
      }
    }

    async function loadPhpExtensions() {
      try {
        const res = await fetch('/api/php/extensions');
        if (!res.ok) return;
        allPhpExtensions = await res.json();
        const countSpan = document.getElementById('php-ext-count');
        if (countSpan) countSpan.innerText = allPhpExtensions.length;
        renderPhpExtensions(allPhpExtensions);
      } catch (e) {
        console.error('Error loading extensions', e);
      }
    }

    function renderPhpExtensions(list) {
      const tbody = document.getElementById('php-ext-table-body');
      if (!list || list.length === 0) {
        tbody.innerHTML = '<tr><td colspan="4" style="text-align:center; color:var(--text-muted); padding:2rem;">No matching PHP extensions discovered.</td></tr>';
        return;
      }
      tbody.innerHTML = list.map(e => `
        <tr>
          <td><strong>${e.name}</strong></td>
          <td style="color:#94a3b8; font-family:monospace; font-size:0.8rem;">${e.file_name}</td>
          <td>
            <span class="badge ${e.is_enabled ? 'badge-running' : 'badge-stopped'}">
              ${e.is_enabled ? 'ENABLED' : 'DISABLED'}
            </span>
          </td>
          <td style="text-align:right;">
            <button class="btn btn-secondary btn-sm" onclick="togglePhpExt('${e.name}', ${!e.is_enabled})">
              ${e.is_enabled ? 'Disable' : 'Enable'}
            </button>
          </td>
        </tr>
      `).join('');
    }

    function filterPhpExtensions(query) {
      const q = (query || '').toLowerCase().trim();
      if (!q) {
        renderPhpExtensions(allPhpExtensions);
        return;
      }
      const filtered = allPhpExtensions.filter(e => e.name.toLowerCase().includes(q) || e.file_name.toLowerCase().includes(q));
      renderPhpExtensions(filtered);
    }

    function switchPhpSubTab(sub) {
      const isConfig = sub === 'config';
      document.getElementById('php-subtab-config').style.display = isConfig ? 'flex' : 'none';
      document.getElementById('php-subtab-extensions').style.display = isConfig ? 'none' : 'flex';
      document.getElementById('btn-subtab-php-config').classList.toggle('active', isConfig);
      document.getElementById('btn-subtab-php-ext').classList.toggle('active', !isConfig);
    }

    async function togglePhpExt(name, enable) {
      try {
        const res = await fetch(`/api/php/toggle-extension?name=${encodeURIComponent(name)}&enable=${enable}`, { method: 'POST' });
        const data = await res.json();
        showToast(data.message || `Extension ${name} ${enable ? 'enabled' : 'disabled'}`);
        loadPhpExtensions();
      } catch (e) {
        showToast('Error toggling extension');
      }
    }

    async function applyPhpPreset(preset) {
      if (!confirm(`Apply official PHP '${preset}' preset? A backup of your current php.ini will be created.`)) return;
      try {
        const res = await fetch(`/api/php/preset?name=${preset}`, { method: 'POST' });
        const data = await res.json();
        showToast(data.message || `Applied ${preset} preset`);
        loadPhp();
      } catch (e) {
        showToast('Error applying preset: ' + e);
      }
    }

    async function openRawIniModal() {
      try {
        const res = await fetch('/api/php/ini');
        const data = await res.json();
        document.getElementById('raw-ini-textarea').value = data.content || '';
        openModal('modal-raw-ini');
      } catch (e) {
        alert('Failed to load php.ini: ' + e);
      }
    }

    async function saveRawIni() {
      const content = document.getElementById('raw-ini-textarea').value;
      try {
        const res = await fetch('/api/php/ini', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content })
        });
        const data = await res.json();
        if (res.ok) {
          showToast('php.ini saved successfully (backup created)');
          closeModal('modal-raw-ini');
          loadPhp();
        } else {
          alert(data.message || 'Error saving php.ini');
        }
      } catch (e) {
        alert('Error saving php.ini: ' + e);
      }
    }

    async function loadDiagnostics() {
      const [portsRes, shadowRes] = await Promise.all([
        fetch('/api/ports'),
        fetch('/api/shadow')
      ]);
      const ports = await portsRes.json();
      const shadow = await shadowRes.json();

      document.getElementById('ports-table-body').innerHTML = ports.slice(0, 50).map(p => `
        <tr>
          <td><span class="code-tag">${p.port}</span></td>
          <td>PID ${p.owning_pid}</td>
          <td style="color:#94a3b8;">${p.local_addr}</td>
        </tr>
      `).join('');

      document.getElementById('shadow-table-body').innerHTML = shadow.map(s => `
        <tr>
          <td><strong>${s.binary_name}</strong></td>
          <td style="font-size:0.8rem; color:#94a3b8;">${s.user_path_location || '-'}</td>
          <td style="font-size:0.8rem; color:#94a3b8;">${s.system_path_location || '-'}</td>
          <td>
            <span class="badge ${s.is_shadowed ? 'badge-conflict' : 'badge-running'}">
              ${s.is_shadowed ? 'COLLISION' : 'OK'}
            </span>
          </td>
        </tr>
      `).join('');
    }

    async function loadLogs() {
      try {
        const res = await fetch('/api/logs');
        if (!res.ok) return;
        const logs = await res.json();
        const box = document.getElementById('log-box');
        if (!box) return;
        if (!logs || logs.length === 0) {
          box.innerHTML = '<div style="color:var(--text-muted); font-style:italic;">No event logs recorded yet.</div>';
          return;
        }
        box.innerHTML = logs.map(l => {
          let color = '#94a3b8';
          if (l.includes('SUCCESS') || l.includes('started') || l.includes('running') || l.includes('active')) {
            color = '#34d399';
          } else if (l.includes('ERROR') || l.includes('FAILED') || l.includes('Failed') || l.includes('error') || l.includes('not found')) {
            color = '#f87171';
          } else if (l.includes('WARNING') || l.includes('Warning') || l.includes('COLLISION')) {
            color = '#fbbf24';
          } else if (l.includes('[SERVICE]') || l.includes('[BATCH]') || l.includes('[APP]') || l.includes('[ACTION]')) {
            color = '#38bdf8';
          }
          return `<div style="color:${color}; margin-bottom:4px; line-height:1.4;">${escapeHtml(l)}</div>`;
        }).join('');
        box.scrollTop = box.scrollHeight;
      } catch (e) {
        console.error(e);
      }
    }

    async function clearLogs() {
      try {
        await fetch('/api/logs/clear', { method: 'POST' });
        showToast('Logs cleared');
        loadLogs();
      } catch (e) {
        showToast('Failed to clear logs');
      }
    }

    function escapeHtml(text) {
      const div = document.createElement('div');
      div.innerText = text;
      return div.innerHTML;
    }

    // Auto-load on load
    loadServices();
    loadLogs();

    // Poll services status or live logs every 2 seconds
    setInterval(() => {
      const activePane = document.querySelector('.tab-pane.active');
      if (activePane && activePane.id === 'pane-services') {
        loadServices();
      } else if (activePane && activePane.id === 'pane-logs') {
        loadLogs();
      }
    }, 2000);
  </script>
</body>
</html>
"#;
