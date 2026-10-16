//! Embedded HTML/CSS/JS Single-Page Dashboard for Dotnet Desktop Application.

pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Dotnet - Development Environment Engine</title>
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
    .brand-icon {
      width: 36px;
      height: 36px;
      background: linear-gradient(135deg, var(--primary), var(--accent));
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 800;
      color: #fff;
      font-size: 1.1rem;
    }
    .brand-title { font-weight: 700; font-size: 1.1rem; letter-spacing: -0.02em; }
    .brand-version { font-size: 0.7rem; color: var(--primary); font-weight: 600; text-transform: uppercase; }
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
  </style>
</head>
<body>
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-icon">D</div>
      <div>
        <div class="brand-title">Dotnet</div>
        <div class="brand-version">Rust Native Engine</div>
      </div>
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
        <option value="standalone">Standalone (Local DB & Web)</option>
        <option value="docker">Docker Coexistence</option>
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
            <h3>Tools Pengembang & Runtime Terpasang</h3>
            <p style="color:var(--text-muted); font-size:0.8rem; margin-top:0.2rem;">
              Semua tools yang terpasang di sistem Windows Anda otomatis terdeteksi dan disinkronkan ke environment Dotnet.
            </p>
          </div>
          <button class="btn btn-secondary btn-sm" onclick="scanAdoptTools()">🔍 Pindai Ulang Tools Sistem</button>
        </div>
        <div class="grid" id="tools-grid">
          <!-- Rendered dynamically -->
        </div>
      </section>

      <!-- Tab 4: Hosts & Sites -->
      <section id="pane-hosts" class="tab-pane">
        <h3>Windows Hosts File (Managed Entries)</h3>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>IP Address</th>
                <th>Hostname</th>
                <th>Status</th>
                <th>Managed</th>
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
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <h3>PHP Extensions & Presets</h3>
          <div style="display:flex; gap:0.5rem;">
            <button class="btn btn-secondary btn-sm" onclick="applyPhpPreset('development')">Apply Development Preset</button>
            <button class="btn btn-secondary btn-sm" onclick="applyPhpPreset('production')">Apply Production Preset</button>
          </div>
        </div>
        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th>Extension Name</th>
                <th>File Name</th>
                <th>Status</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody id="php-ext-table-body">
              <!-- Rendered dynamically -->
            </tbody>
          </table>
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
        <h3>System Event Logs</h3>
        <div class="table-container" style="padding:1rem; background:#080c14; font-family:monospace; font-size:0.8rem; height:450px; overflow-y:auto;" id="log-box">
          [System] Dotnet Rust Native Engine started successfully.<br>
          [System] Local CA and Nginx manager active.<br>
        </div>
      </section>
    </div>
  </main>

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
      else if (id === 'php') await loadPhpExtensions();
      else if (id === 'diagnostics') await loadDiagnostics();
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
      await fetch(`/api/services/action?id=${id}&action=${action}`, { method: 'POST' });
      showToast(`Service ${id} ${action} triggered`);
      setTimeout(loadServices, 500);
    }

    async function startAllServices() {
      await fetch('/api/services/start-all', { method: 'POST' });
      showToast('Starting all services...');
      setTimeout(loadServices, 1000);
    }

    async function stopAllServices() {
      await fetch('/api/services/stop-all', { method: 'POST' });
      showToast('Stopping all services...');
      setTimeout(loadServices, 1000);
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
      showToast('Memindai tools di sistem Windows...');
      try {
        const res = await fetch('/api/tools/scan', { method: 'POST' });
        const data = await res.json();
        showToast(data.message || 'Pemindaian tools selesai!');
      } catch (e) {
        showToast('Gagal memindai tools');
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
            <div class="card-meta-row"><span>Kategori:</span><span>${t.category}</span></div>
            <div class="card-meta-row"><span>ID:</span><span class="code-tag">${t.id}</span></div>
            ${t.is_installed ? '<div class="card-meta-row"><span style="color:#10b981; font-weight:600;">✓ Terpasang di Sistem</span></div>' : ''}
          </div>
          <div class="card-actions">
            ${t.is_installed
              ? `<button class="btn btn-secondary btn-sm" style="width:100%; justify-content:center;" disabled>✓ Siap Digunakan</button>`
              : `<button class="btn btn-primary btn-sm" style="width:100%; justify-content:center;" onclick="installTool('${t.id}')">Pasang Tool</button>`
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

    async function loadHosts() {
      const res = await fetch('/api/hosts');
      const data = await res.json();
      const tbody = document.getElementById('hosts-table-body');
      tbody.innerHTML = data.map(h => `
        <tr>
          <td><span class="code-tag">${h.ip}</span></td>
          <td><strong>${h.hostname}</strong></td>
          <td>${h.is_enabled ? 'Active' : 'Disabled'}</td>
          <td>${h.is_managed ? 'Managed by Dotnet' : 'System'}</td>
        </tr>
      `).join('');
    }

    async function loadPhpExtensions() {
      const res = await fetch('/api/php/extensions');
      const data = await res.json();
      const tbody = document.getElementById('php-ext-table-body');
      tbody.innerHTML = data.map(e => `
        <tr>
          <td><strong>${e.name}</strong></td>
          <td style="color:#94a3b8; font-family:monospace;">${e.file_name}</td>
          <td>
            <span class="badge ${e.is_enabled ? 'badge-running' : 'badge-stopped'}">
              ${e.is_enabled ? 'ENABLED' : 'DISABLED'}
            </span>
          </td>
          <td>
            <button class="btn btn-secondary btn-sm" onclick="togglePhpExt('${e.name}', ${!e.is_enabled})">
              ${e.is_enabled ? 'Disable' : 'Enable'}
            </button>
          </td>
        </tr>
      `).join('');
    }

    async function togglePhpExt(name, enable) {
      await fetch(`/api/php/toggle-extension?name=${name}&enable=${enable}`, { method: 'POST' });
      showToast(`Extension ${name} ${enable ? 'enabled' : 'disabled'}`);
      loadPhpExtensions();
    }

    async function applyPhpPreset(preset) {
      await fetch(`/api/php/preset?name=${preset}`, { method: 'POST' });
      showToast(`Applied ${preset} preset`);
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

    // Auto-load on load
    loadServices();
    // Poll services status every 3 seconds
    setInterval(() => {
      const activePane = document.querySelector('.tab-pane.active');
      if (activePane && activePane.id === 'pane-services') {
        loadServices();
      }
    }, 3000);
  </script>
</body>
</html>
"#;
