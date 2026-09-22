using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Forms;
using dotnet.Config;
using dotnet.Models;
using dotnet.Services;

namespace dotnet;

public partial class Form1 : Form
{
    private readonly List<DevServiceInfo> _services = new();
    private TabControl _tabControl = null!;
    private RichTextBox _logBox = null!;
    private FlowLayoutPanel _servicesPanel = null!;
    private DataGridView _projectsGrid = null!;
    private DataGridView _hostsGrid = null!;
    private TextBox _phpMemoryLimit = null!;
    private TextBox _phpUploadMax = null!;
    private TextBox _phpPostMax = null!;
    private TextBox _phpMaxExec = null!;
    private Label _adminStatusLabel = null!;
    private NotifyIcon _notifyIcon = null!;
    private ContextMenuStrip _trayMenu = null!;
    private CheckBox _chkAutoStart = null!;
    private bool _allowClose = false;
    private bool _startMinimized = false;

    public Form1(bool startMinimized = false)
    {
        _startMinimized = startMinimized;
        InitializeComponent();
        SetupServicesList();
        BuildCustomUi();
        SetupSystemTray();
        AppLogger.OnLog += AppendLog;
        AppLogger.Log("Dotnet initialized.");
        RefreshServicesStatus();

        this.Shown += (s, e) => {
            // Automatically start services marked for Boot Auto-Start
            Task.Run(async () => await AutoStartBootServices());
        };

        if (_startMinimized)
        {
            this.WindowState = FormWindowState.Minimized;
            this.ShowInTaskbar = false;
            this.Hide();
        }
    }

    private async Task AutoStartBootServices()
    {
        foreach (var svc in _services)
        {
            if (svc.AutoStartOnBoot)
            {
                await StartSingleService(svc);
            }
        }
        if (InvokeRequired)
        {
            Invoke(new Action(RefreshServicesStatus));
        }
        else
        {
            RefreshServicesStatus();
        }
    }

    private void SetupServicesList()
    {
        _services.Add(new DevServiceInfo
        {
            Id = "mysql",
            Name = "MySQL 8.4",
            Type = DevServiceType.WindowsService,
            WindowsServiceName = "MySQL84",
            Port = 3306,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("mysql", true)
        });

        _services.Add(new DevServiceInfo
        {
            Id = "postgres",
            Name = "PostgreSQL 17",
            Type = DevServiceType.WindowsService,
            WindowsServiceName = "postgresql-x64-17",
            Port = 5432,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("postgres", true)
        });

        _services.Add(new DevServiceInfo
        {
            Id = "redis",
            Name = "Redis (Memurai)",
            Type = DevServiceType.WindowsService,
            WindowsServiceName = "Memurai",
            Port = 6379,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("redis", true)
        });

        _services.Add(new DevServiceInfo
        {
            Id = "nginx",
            Name = "Nginx Web Server",
            Type = DevServiceType.ManagedProcess,
            ExecutablePath = @"C:\tools\nginx\nginx.exe",
            Port = 80,
            AutoStartWithGroup = false,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("nginx", false)
        });

        _services.Add(new DevServiceInfo
        {
            Id = "php-cgi",
            Name = "PHP 8.5 FastCGI",
            Type = DevServiceType.ManagedProcess,
            ExecutablePath = @"C:\tools\php85\php-cgi.exe",
            Arguments = "-b 127.0.0.1:9000",
            Port = 9000,
            AutoStartWithGroup = false,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("php-cgi", false)
        });
    }

    private void BuildCustomUi()
    {
        this.Text = "Dotnet (Native Laragon Replacement)";
        this.Size = new Size(1100, 750);
        this.BackColor = SystemColors.Control;
        this.ForeColor = SystemColors.ControlText;
        this.Font = new Font("Tahoma", 8.25F, FontStyle.Regular, GraphicsUnit.Point);

        // Header Panel
        var headerPanel = new Panel
        {
            Dock = DockStyle.Top,
            Height = 55,
            BackColor = SystemColors.Control,
            BorderStyle = BorderStyle.FixedSingle,
            Padding = new Padding(10)
        };

        var titleLabel = new Label
        {
            Text = "Standalone Local Dev Environment Manager",
            Font = new Font("Tahoma", 9.75F, FontStyle.Bold),
            ForeColor = SystemColors.ControlText,
            AutoSize = true,
            Location = new Point(12, 16)
        };

        bool isAdmin = HostsFileManager.IsElevated();
        _adminStatusLabel = new Label
        {
            Text = isAdmin ? "[ Administrator ]" : "[ Standard User ]",
            ForeColor = isAdmin ? Color.DarkGreen : Color.DarkRed,
            AutoSize = true,
            Location = new Point(340, 18),
            Font = new Font("Tahoma", 8.25F, FontStyle.Bold)
        };

        _chkAutoStart = new CheckBox
        {
            Text = "Run on Boot",
            ForeColor = SystemColors.ControlText,
            AutoSize = true,
            Location = new Point(480, 18),
            Checked = StartupManager.IsRunOnStartupEnabled(),
            Cursor = Cursors.Hand
        };
        _chkAutoStart.CheckedChanged += (s, e) =>
        {
            StartupManager.SetRunOnStartup(_chkAutoStart.Checked);
        };

        var btnStartAll = new Button
        {
            Text = "▶ Start All",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(90, 28),
            Location = new Point(620, 13),
            Cursor = Cursors.Hand
        };
        btnStartAll.Click += async (s, e) => await StartAllGroupServices();

        var btnStopAll = new Button
        {
            Text = "⏹ Stop All",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(90, 28),
            Location = new Point(720, 13),
            Cursor = Cursors.Hand
        };
        btnStopAll.Click += async (s, e) => await StopAllGroupServices();

        var btnRefresh = new Button
        {
            Text = "🔄 Refresh",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(85, 28),
            Location = new Point(820, 13),
            Cursor = Cursors.Hand
        };
        btnRefresh.Click += (s, e) => RefreshServicesStatus();

        headerPanel.Controls.Add(titleLabel);
        headerPanel.Controls.Add(_adminStatusLabel);
        headerPanel.Controls.Add(_chkAutoStart);
        headerPanel.Controls.Add(btnStartAll);
        headerPanel.Controls.Add(btnStopAll);
        headerPanel.Controls.Add(btnRefresh);

        // Bottom Log Panel
        var logPanel = new Panel
        {
            Dock = DockStyle.Bottom,
            Height = 150,
            BackColor = SystemColors.Control,
            Padding = new Padding(6)
        };

        var logTitle = new Label
        {
            Text = "Operational Log Console:",
            Dock = DockStyle.Top,
            Height = 20,
            ForeColor = SystemColors.ControlDarkDark,
            Font = new Font("Tahoma", 8.25F, FontStyle.Italic)
        };

        _logBox = new RichTextBox
        {
            Dock = DockStyle.Fill,
            BackColor = SystemColors.Window,
            ForeColor = SystemColors.WindowText,
            Font = new Font("Consolas", 9F),
            ReadOnly = true,
            BorderStyle = BorderStyle.Fixed3D
        };

        logPanel.Controls.Add(_logBox);
        logPanel.Controls.Add(logTitle);

        // Tab Control Main Area
        _tabControl = new TabControl
        {
            Dock = DockStyle.Fill,
            Padding = new Point(10, 5)
        };

        // Tab 1: Dashboard
        var tabDashboard = new TabPage("Services Dashboard") { BackColor = SystemColors.Control };
        _servicesPanel = new FlowLayoutPanel
        {
            Dock = DockStyle.Fill,
            AutoScroll = true,
            Padding = new Padding(10)
        };
        tabDashboard.Controls.Add(_servicesPanel);

        // Tab 2: Projects
        var tabProjects = CreateProjectsTab();

        // Tab 3: Hosts File Manager
        var tabHosts = CreateHostsTab();

        // Tab 4: PHP & Nginx Configuration
        var tabConfig = CreateConfigTab();

        // Tab 5: Port Diagnostics & Docker Check
        var tabDiagnostics = CreateDiagnosticsTab();

        _tabControl.TabPages.Add(tabDashboard);
        _tabControl.TabPages.Add(tabProjects);
        _tabControl.TabPages.Add(tabHosts);
        _tabControl.TabPages.Add(tabConfig);
        _tabControl.TabPages.Add(tabDiagnostics);

        this.Controls.Add(_tabControl);
        this.Controls.Add(logPanel);
        this.Controls.Add(headerPanel);
    }

    private void AppendLog(string message)
    {
        if (InvokeRequired)
        {
            Invoke(new Action<string>(AppendLog), message);
            return;
        }

        _logBox.AppendText(message + Environment.NewLine);
        _logBox.SelectionStart = _logBox.Text.Length;
        _logBox.ScrollToCaret();
    }

    private void RefreshServicesStatus()
    {
        _servicesPanel.Controls.Clear();

        foreach (var svc in _services)
        {
            if (svc.Type == DevServiceType.WindowsService)
            {
                svc.Status = WindowsServiceManager.GetStatus(svc.WindowsServiceName);
            }
            else
            {
                svc.Status = ProcessServiceManager.GetStatus(svc);
            }

            var card = CreateServiceCard(svc);
            _servicesPanel.Controls.Add(card);
        }
    }

    private Panel CreateServiceCard(DevServiceInfo svc)
    {
        var card = new Panel
        {
            Size = new Size(1030, 60),
            BackColor = SystemColors.Control,
            BorderStyle = BorderStyle.FixedSingle,
            Margin = new Padding(0, 0, 0, 8),
            Padding = new Padding(8)
        };

        var lblName = new Label
        {
            Text = svc.Name,
            Font = new Font("Tahoma", 9F, FontStyle.Bold),
            ForeColor = SystemColors.ControlText,
            Location = new Point(12, 18),
            AutoSize = true
        };

        var lblPort = new Label
        {
            Text = $"Port: {svc.Port}",
            ForeColor = SystemColors.ControlDarkDark,
            Location = new Point(230, 20),
            AutoSize = true
        };

        string statusText = svc.Status switch
        {
            ServiceStatus.Running => "● RUNNING",
            ServiceStatus.Stopped => "○ STOPPED",
            ServiceStatus.Starting => "⏳ STARTING",
            ServiceStatus.Stopping => "⏳ STOPPING",
            ServiceStatus.NotInstalled => "⛔ NOT INSTALLED",
            ServiceStatus.PortConflict => "⚠️ CONFLICT",
            ServiceStatus.Error => "❌ ERROR",
            _ => "❓ UNKNOWN"
        };

        Color statusColor = svc.Status switch
        {
            ServiceStatus.Running => Color.DarkGreen,
            ServiceStatus.Stopped => Color.DarkRed,
            ServiceStatus.NotInstalled => Color.Gray,
            ServiceStatus.PortConflict => Color.Crimson,
            ServiceStatus.Error => Color.Red,
            _ => Color.DarkGoldenrod
        };

        var lblStatus = new Label
        {
            Text = statusText,
            ForeColor = statusColor,
            Font = new Font("Tahoma", 8.5F, FontStyle.Bold),
            Location = new Point(370, 19),
            AutoSize = true
        };

        var lblPid = new Label
        {
            Text = svc.ProcessId.HasValue ? $"PID: {svc.ProcessId}" : "",
            ForeColor = SystemColors.ControlDarkDark,
            Location = new Point(510, 20),
            AutoSize = true
        };

        var chkAutoBoot = new CheckBox
        {
            Text = "Auto-Start Boot",
            Checked = svc.AutoStartOnBoot,
            ForeColor = SystemColors.ControlText,
            Font = new Font("Tahoma", 8.25F),
            Location = new Point(620, 19),
            AutoSize = true,
            Cursor = Cursors.Hand
        };
        chkAutoBoot.CheckedChanged += (s, e) =>
        {
            svc.AutoStartOnBoot = chkAutoBoot.Checked;
            ServiceSettingsManager.SetAutoStartOnBoot(svc.Id, svc.AutoStartOnBoot);
        };

        var btnToggle = new Button
        {
            Text = svc.Status == ServiceStatus.Running ? "Stop" : "Start",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(80, 26),
            Location = new Point(780, 14),
            Cursor = Cursors.Hand,
            Enabled = svc.Status != ServiceStatus.NotInstalled
        };
        btnToggle.Click += async (s, e) =>
        {
            btnToggle.Enabled = false;
            if (svc.Status == ServiceStatus.Running)
            {
                await StopSingleService(svc);
            }
            else
            {
                await StartSingleService(svc);
            }
            RefreshServicesStatus();
        };

        var btnRestart = new Button
        {
            Text = "Restart",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(75, 26),
            Location = new Point(870, 14),
            Cursor = Cursors.Hand,
            Enabled = svc.Status == ServiceStatus.Running
        };
        btnRestart.Click += async (s, e) =>
        {
            btnRestart.Enabled = false;
            await StopSingleService(svc);
            await Task.Delay(1000);
            await StartSingleService(svc);
            RefreshServicesStatus();
        };

        var btnLogs = new Button
        {
            Text = "Config",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(65, 26),
            Location = new Point(950, 14),
            Cursor = Cursors.Hand
        };
        btnLogs.Click += (s, e) => OpenServiceConfig(svc);

        card.Controls.Add(lblName);
        card.Controls.Add(lblPort);
        card.Controls.Add(lblStatus);
        card.Controls.Add(lblPid);
        card.Controls.Add(chkAutoBoot);
        card.Controls.Add(btnToggle);
        card.Controls.Add(btnRestart);
        card.Controls.Add(btnLogs);

        return card;
    }

    private void OpenServiceConfig(DevServiceInfo svc)
    {
        string path = "";
        if (svc.Id == "nginx")
        {
            path = @"C:\tools\nginx\conf\nginx.conf";
        }
        else if (svc.Id == "php-cgi")
        {
            path = PhpConfigManager.GetIniPath();
        }
        else if (svc.Id == "mysql")
        {
            path = @"C:\tools\mysql84\my.ini";
        }

        if (!string.IsNullOrEmpty(path) && File.Exists(path))
        {
            System.Diagnostics.Process.Start("notepad.exe", path);
        }
        else
        {
            MessageBox.Show($"Configuration file not found or not configured for {svc.Name}.", "Info", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
    }

    private async Task StartSingleService(DevServiceInfo svc)
    {
        var conflict = PortConflictDetector.CheckPort(svc.Port);
        if (conflict.IsInUse)
        {
            bool isOwnProcess = false;
            if (svc.ProcessId.HasValue && svc.ProcessId.Value == conflict.ProcessId)
            {
                isOwnProcess = true;
            }
            else if (!string.IsNullOrEmpty(conflict.ProcessName))
            {
                string procName = conflict.ProcessName.ToLowerInvariant();
                if (svc.Id == "mysql" && procName.Contains("mysqld")) isOwnProcess = true;
                else if (svc.Id == "postgres" && procName.Contains("postgres")) isOwnProcess = true;
                else if (svc.Id == "redis" && (procName.Contains("memurai") || procName.Contains("redis"))) isOwnProcess = true;
                else if (svc.Id == "nginx" && procName.Contains("nginx")) isOwnProcess = true;
                else if (svc.Id == "php-cgi" && procName.Contains("php-cgi")) isOwnProcess = true;
            }

            if (isOwnProcess)
            {
                AppLogger.Log($"{svc.Name} is already active on port {svc.Port} (PID {conflict.ProcessId}).");
                return;
            }

            MessageBox.Show(
                $"Port conflict detected!\nPort {svc.Port} is already occupied by external process: {conflict.ProcessName} (PID {conflict.ProcessId}).\nPlease stop the conflicting process first.",
                "Port Conflict Warning",
                MessageBoxButtons.OK,
                MessageBoxIcon.Warning
            );
            return;
        }

        if (svc.Type == DevServiceType.WindowsService)
        {
            var res = await WindowsServiceManager.StartServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
            if (!res.Success)
            {
                MessageBox.Show(res.Message, "Service Start Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }
        else
        {
            ProcessServiceManager.StartProcess(svc);
        }
    }

    private async Task StopSingleService(DevServiceInfo svc)
    {
        if (svc.Type == DevServiceType.WindowsService)
        {
            var res = await WindowsServiceManager.StopServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
            if (!res.Success)
            {
                MessageBox.Show(res.Message, "Service Stop Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }
        else
        {
            ProcessServiceManager.StopProcess(svc);
        }
    }

    private async Task StartAllGroupServices()
    {
        AppLogger.Log("Executing Start All Services...");
        foreach (var svc in _services)
        {
            if (svc.AutoStartWithGroup)
            {
                await StartSingleService(svc);
            }
        }
        RefreshServicesStatus();
    }

    private async Task StopAllGroupServices()
    {
        AppLogger.Log("Executing Stop All Services...");
        foreach (var svc in _services)
        {
            await StopSingleService(svc);
        }
        RefreshServicesStatus();
    }

    // TAB 2: PROJECTS
    private TabPage CreateProjectsTab()
    {
        var tab = new TabPage("Projects") { BackColor = SystemColors.Control };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 40, Padding = new Padding(5) };
        var btnAddProject = new Button
        {
            Text = "+ Add Project",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(110, 26),
            Location = new Point(8, 6),
            Cursor = Cursors.Hand
        };
        btnAddProject.Click += (s, e) => ShowAddProjectDialog();

        var btnReloadProjects = new Button
        {
            Text = "🔄 Refresh List",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(100, 26),
            Location = new Point(125, 6),
            Cursor = Cursors.Hand
        };
        btnReloadProjects.Click += (s, e) => RefreshProjectsGrid();

        topBar.Controls.Add(btnAddProject);
        topBar.Controls.Add(btnReloadProjects);

        _projectsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = SystemColors.Window,
            ForeColor = SystemColors.ControlText,
            BorderStyle = BorderStyle.Fixed3D,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            AllowUserToAddRows = false
        };

        _projectsGrid.Columns.Add("Id", "ID");
        if (_projectsGrid.Columns["Id"] != null)
        {
            _projectsGrid.Columns["Id"]!.Visible = false;
        }
        _projectsGrid.Columns.Add("Name", "Project Name");
        _projectsGrid.Columns.Add("Path", "Directory Path");
        _projectsGrid.Columns.Add("DevCmd", "Dev Command");
        _projectsGrid.Columns.Add("Host", "Local Domain");

        var btnColRun = new DataGridViewButtonColumn
        {
            HeaderText = "Action",
            Text = "Run Command",
            UseColumnTextForButtonValue = true,
            FlatStyle = FlatStyle.Standard
        };
        _projectsGrid.Columns.Add(btnColRun);

        _projectsGrid.CellClick += (s, e) =>
        {
            if (e.RowIndex >= 0 && e.ColumnIndex == _projectsGrid.Columns.Count - 1)
            {
                string id = _projectsGrid.Rows[e.RowIndex].Cells["Id"].Value?.ToString() ?? "";
                var p = ProjectManager.GetProjects().FirstOrDefault(proj => proj.Id == id);
                if (p != null)
                {
                    ProjectManager.LaunchDevCommand(p);
                }
            }
        };

        RefreshProjectsGrid();

        tab.Controls.Add(_projectsGrid);
        tab.Controls.Add(topBar);

        return tab;
    }

    private void RefreshProjectsGrid()
    {
        _projectsGrid.Rows.Clear();
        foreach (var p in ProjectManager.GetProjects())
        {
            _projectsGrid.Rows.Add(p.Id, p.Name, p.Path, p.DevCommand, p.NginxHost);
        }
    }

    private void ShowAddProjectDialog()
    {
        using var dlg = new Form
        {
            Text = "Add New Project",
            Size = new Size(450, 310),
            StartPosition = FormStartPosition.CenterParent,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = SystemColors.Control,
            ForeColor = SystemColors.ControlText,
            Font = new Font("Tahoma", 8.25F)
        };

        var lbl1 = new Label { Text = "Project Name:", Location = new Point(20, 20), AutoSize = true };
        var txtName = new TextBox { Location = new Point(140, 18), Width = 260, BorderStyle = BorderStyle.Fixed3D };

        var lbl2 = new Label { Text = "Project Directory:", Location = new Point(20, 55), AutoSize = true };
        var txtPath = new TextBox { Location = new Point(140, 53), Width = 180, BorderStyle = BorderStyle.Fixed3D };
        var btnBrowse = new Button { Text = "Browse...", Location = new Point(330, 51), Width = 70, Height = 24, FlatStyle = FlatStyle.Standard };
        btnBrowse.Click += (s, e) =>
        {
            using var fbd = new FolderBrowserDialog();
            if (fbd.ShowDialog() == DialogResult.OK) txtPath.Text = fbd.SelectedPath;
        };

        var lbl3 = new Label { Text = "Dev Command:", Location = new Point(20, 90), AutoSize = true };
        var txtCmd = new TextBox { Text = "composer run dev", Location = new Point(140, 88), Width = 260, BorderStyle = BorderStyle.Fixed3D };

        var lbl4 = new Label { Text = "Nginx Host Domain:", Location = new Point(20, 125), AutoSize = true };
        var txtHost = new TextBox { Text = "project.test", Location = new Point(140, 123), Width = 260, BorderStyle = BorderStyle.Fixed3D };

        var btnSave = new Button
        {
            Text = "Save Project",
            DialogResult = DialogResult.OK,
            Location = new Point(140, 180),
            Width = 110,
            Height = 30,
            FlatStyle = FlatStyle.Standard
        };

        btnSave.Click += (s, e) =>
        {
            if (string.IsNullOrWhiteSpace(txtName.Text) || string.IsNullOrWhiteSpace(txtPath.Text))
            {
                MessageBox.Show("Please fill Name and Path.", "Validation Error", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                dlg.DialogResult = DialogResult.None;
                return;
            }

            ProjectManager.AddOrUpdateProject(new ProjectInfo
            {
                Name = txtName.Text.Trim(),
                Path = txtPath.Text.Trim(),
                DevCommand = txtCmd.Text.Trim(),
                NginxHost = txtHost.Text.Trim()
            });

            RefreshProjectsGrid();
        };

        dlg.Controls.Add(lbl1); dlg.Controls.Add(txtName);
        dlg.Controls.Add(lbl2); dlg.Controls.Add(txtPath); dlg.Controls.Add(btnBrowse);
        dlg.Controls.Add(lbl3); dlg.Controls.Add(txtCmd);
        dlg.Controls.Add(lbl4); dlg.Controls.Add(txtHost);
        dlg.Controls.Add(btnSave);

        dlg.ShowDialog(this);
    }

    // TAB 3: HOSTS FILE MANAGER
    private TabPage CreateHostsTab()
    {
        var tab = new TabPage("Hosts File") { BackColor = SystemColors.Control };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 40, Padding = new Padding(5) };
        var btnAddHost = new Button
        {
            Text = "+ Add Host Entry",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(120, 26),
            Location = new Point(8, 6),
            Cursor = Cursors.Hand
        };
        btnAddHost.Click += (s, e) => ShowAddHostDialog();

        var btnReloadHosts = new Button
        {
            Text = "🔄 Reload Hosts",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(110, 26),
            Location = new Point(135, 6),
            Cursor = Cursors.Hand
        };
        btnReloadHosts.Click += (s, e) => RefreshHostsGrid();

        topBar.Controls.Add(btnAddHost);
        topBar.Controls.Add(btnReloadHosts);

        _hostsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = SystemColors.Window,
            ForeColor = SystemColors.ControlText,
            BorderStyle = BorderStyle.Fixed3D,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            AllowUserToAddRows = false
        };

        _hostsGrid.Columns.Add("Ip", "IP Address");
        _hostsGrid.Columns.Add("Hostname", "Domain / Hostname");
        _hostsGrid.Columns.Add("Enabled", "Enabled");
        _hostsGrid.Columns.Add("Managed", "Type");

        RefreshHostsGrid();

        tab.Controls.Add(_hostsGrid);
        tab.Controls.Add(topBar);

        return tab;
    }

    private void RefreshHostsGrid()
    {
        _hostsGrid.Rows.Clear();
        var entries = HostsFileManager.ReadEntries();
        foreach (var entry in entries)
        {
            string type = entry.IsManaged ? "Dotnet Managed" : "System / External";
            _hostsGrid.Rows.Add(entry.IpAddress, entry.Hostname, entry.IsEnabled ? "Yes" : "No (#)", type);
        }
    }

    private void ShowAddHostDialog()
    {
        using var dlg = new Form
        {
            Text = "Add Hosts Entry (Dotnet Managed Block)",
            Size = new Size(420, 220),
            StartPosition = FormStartPosition.CenterParent,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = SystemColors.Control,
            ForeColor = SystemColors.ControlText,
            Font = new Font("Tahoma", 8.25F)
        };

        var lbl1 = new Label { Text = "IP Address:", Location = new Point(20, 25), AutoSize = true };
        var txtIp = new TextBox { Text = "127.0.0.1", Location = new Point(120, 23), Width = 250, BorderStyle = BorderStyle.Fixed3D };

        var lbl2 = new Label { Text = "Hostname:", Location = new Point(20, 65), AutoSize = true };
        var txtHost = new TextBox { Text = "myproject.test", Location = new Point(120, 63), Width = 250, BorderStyle = BorderStyle.Fixed3D };

        var btnSave = new Button
        {
            Text = "Add Entry",
            DialogResult = DialogResult.OK,
            Location = new Point(120, 115),
            Width = 100,
            Height = 28,
            FlatStyle = FlatStyle.Standard
        };

        btnSave.Click += (s, e) =>
        {
            if (string.IsNullOrWhiteSpace(txtHost.Text))
            {
                MessageBox.Show("Please enter a hostname.", "Validation Error", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                dlg.DialogResult = DialogResult.None;
                return;
            }

            bool success = HostsFileManager.AddOrUpdateEntry(txtIp.Text.Trim(), txtHost.Text.Trim());
            if (success)
            {
                RefreshHostsGrid();
            }
            else
            {
                MessageBox.Show("Failed to write to hosts file. Administrator elevation required.", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        };

        dlg.Controls.Add(lbl1); dlg.Controls.Add(txtIp);
        dlg.Controls.Add(lbl2); dlg.Controls.Add(txtHost);
        dlg.Controls.Add(btnSave);

        dlg.ShowDialog(this);
    }

    // TAB 4: PHP & NGINX CONFIG
    private TabPage CreateConfigTab()
    {
        var tab = new TabPage("PHP & Nginx Config") { BackColor = SystemColors.Control };

        var pnlPhp = new GroupBox
        {
            Text = "PHP Configuration (php.ini)",
            Size = new Size(500, 330),
            Location = new Point(15, 15),
            ForeColor = SystemColors.ControlText
        };

        var lblPath = new Label { Text = $"Path: {PhpConfigManager.GetIniPath()}", Location = new Point(15, 25), AutoSize = true, ForeColor = SystemColors.ControlDarkDark };

        var l1 = new Label { Text = "memory_limit:", Location = new Point(15, 60), AutoSize = true };
        _phpMemoryLimit = new TextBox { Location = new Point(160, 58), Width = 150, BorderStyle = BorderStyle.Fixed3D };

        var l2 = new Label { Text = "upload_max_filesize:", Location = new Point(15, 95), AutoSize = true };
        _phpUploadMax = new TextBox { Location = new Point(160, 93), Width = 150, BorderStyle = BorderStyle.Fixed3D };

        var l3 = new Label { Text = "post_max_size:", Location = new Point(15, 130), AutoSize = true };
        _phpPostMax = new TextBox { Location = new Point(160, 128), Width = 150, BorderStyle = BorderStyle.Fixed3D };

        var l4 = new Label { Text = "max_execution_time:", Location = new Point(15, 165), AutoSize = true };
        _phpMaxExec = new TextBox { Location = new Point(160, 163), Width = 150, BorderStyle = BorderStyle.Fixed3D };

        var btnLoadPhp = new Button { Text = "Read INI", Location = new Point(15, 230), Width = 90, Height = 28, FlatStyle = FlatStyle.Standard };
        btnLoadPhp.Click += (s, e) => LoadPhpIniValues();

        var btnSavePhp = new Button { Text = "Save php.ini", Location = new Point(115, 230), Width = 110, Height = 28, FlatStyle = FlatStyle.Standard };
        btnSavePhp.Click += (s, e) => SavePhpIniValues();

        var btnOpenPhpNotepad = new Button 
        { 
            Text = "📝 Edit in Notepad", 
            Location = new Point(235, 230), 
            Width = 130, 
            Height = 28, 
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand 
        };
        btnOpenPhpNotepad.Click += (s, e) =>
        {
            try
            {
                string iniPath = PhpConfigManager.GetIniPath();
                if (File.Exists(iniPath))
                {
                    System.Diagnostics.Process.Start("notepad.exe", iniPath);
                }
                else
                {
                    MessageBox.Show($"php.ini not found at {iniPath}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to open Notepad: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        };

        pnlPhp.Controls.Add(lblPath);
        pnlPhp.Controls.Add(l1); pnlPhp.Controls.Add(_phpMemoryLimit);
        pnlPhp.Controls.Add(l2); pnlPhp.Controls.Add(_phpUploadMax);
        pnlPhp.Controls.Add(l3); pnlPhp.Controls.Add(_phpPostMax);
        pnlPhp.Controls.Add(l4); pnlPhp.Controls.Add(_phpMaxExec);
        pnlPhp.Controls.Add(btnLoadPhp); pnlPhp.Controls.Add(btnSavePhp);
        pnlPhp.Controls.Add(btnOpenPhpNotepad);

        // Nginx Manager Group Box
        var pnlNginx = new GroupBox
        {
            Text = "Nginx Web Server Operations",
            Size = new Size(500, 330),
            Location = new Point(530, 15),
            ForeColor = SystemColors.ControlText
        };

        var btnTestNginx = new Button
        {
            Text = "🧪 Validate Syntax (nginx -t)",
            Location = new Point(25, 35),
            Size = new Size(240, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnTestNginx.Click += (s, e) =>
        {
            var res = NginxManager.ValidateConfig();
            MessageBox.Show(res.Output, res.Success ? "Nginx Config Test Passed" : "Nginx Config Test Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
        };

        var btnReloadNginx = new Button
        {
            Text = "⚡ Reload Nginx (nginx -s reload)",
            Location = new Point(25, 75),
            Size = new Size(240, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnReloadNginx.Click += (s, e) =>
        {
            var res = NginxManager.ReloadNginx();
            MessageBox.Show(res.Output, res.Success ? "Nginx Reload Success" : "Nginx Reload Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
        };

        var btnOpenNginxNotepad = new Button
        {
            Text = "📝 Edit nginx.conf in Notepad",
            Location = new Point(25, 120),
            Size = new Size(240, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnOpenNginxNotepad.Click += (s, e) =>
        {
            try
            {
                string confPath = @"C:\tools\nginx\conf\nginx.conf";
                if (File.Exists(confPath))
                {
                    System.Diagnostics.Process.Start("notepad.exe", confPath);
                }
                else
                {
                    MessageBox.Show($"nginx.conf not found at {confPath}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to open Notepad: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        };

        var btnOpenSitesFolder = new Button
        {
            Text = "📁 Open Sites Config Folder",
            Location = new Point(25, 165),
            Size = new Size(240, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnOpenSitesFolder.Click += (s, e) =>
        {
            try
            {
                string sitesFolder = @"C:\tools\nginx\conf\sites\";
                Directory.CreateDirectory(sitesFolder);
                System.Diagnostics.Process.Start("explorer.exe", sitesFolder);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to open sites folder: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        };

        pnlNginx.Controls.Add(btnTestNginx);
        pnlNginx.Controls.Add(btnReloadNginx);
        pnlNginx.Controls.Add(btnOpenNginxNotepad);
        pnlNginx.Controls.Add(btnOpenSitesFolder);

        tab.Controls.Add(pnlPhp);
        tab.Controls.Add(pnlNginx);

        LoadPhpIniValues();

        return tab;
    }

    private void LoadPhpIniValues()
    {
        _phpMemoryLimit.Text = PhpConfigManager.GetSettingValue("memory_limit");
        _phpUploadMax.Text = PhpConfigManager.GetSettingValue("upload_max_filesize");
        _phpPostMax.Text = PhpConfigManager.GetSettingValue("post_max_size");
        _phpMaxExec.Text = PhpConfigManager.GetSettingValue("max_execution_time");
    }

    private void SavePhpIniValues()
    {
        PhpConfigManager.UpdateSetting("memory_limit", _phpMemoryLimit.Text.Trim());
        PhpConfigManager.UpdateSetting("upload_max_filesize", _phpUploadMax.Text.Trim());
        PhpConfigManager.UpdateSetting("post_max_size", _phpPostMax.Text.Trim());
        PhpConfigManager.UpdateSetting("max_execution_time", _phpMaxExec.Text.Trim());
        MessageBox.Show("php.ini settings saved with backup!", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
    }

    // TAB 5: DIAGNOSTICS & DOCKER CHECK
    private TabPage CreateDiagnosticsTab()
    {
        var tab = new TabPage("Port & Docker Pre-Flight") { BackColor = SystemColors.Control };

        var gbPort = new GroupBox
        {
            Text = "Inspect Specific Port Owner",
            Size = new Size(480, 280),
            Location = new Point(15, 15),
            ForeColor = SystemColors.ControlText
        };

        var lblPort = new Label { Text = "Port Number:", Location = new Point(15, 30), AutoSize = true };
        var txtPort = new TextBox { Text = "3306", Location = new Point(110, 28), Width = 90, BorderStyle = BorderStyle.Fixed3D };
        var btnCheckPort = new Button { Text = "Inspect Port", Location = new Point(210, 26), Width = 95, Height = 26, FlatStyle = FlatStyle.Standard };

        var txtPortResult = new RichTextBox
        {
            Location = new Point(15, 65),
            Size = new Size(445, 195),
            BackColor = SystemColors.Window,
            ForeColor = SystemColors.WindowText,
            BorderStyle = BorderStyle.Fixed3D,
            Font = new Font("Consolas", 9F),
            ReadOnly = true
        };

        btnCheckPort.Click += (s, e) =>
        {
            if (int.TryParse(txtPort.Text, out int port))
            {
                var conflict = PortConflictDetector.CheckPort(port);
                if (conflict.IsInUse)
                {
                    txtPortResult.Text = $"⚠️ Port {port} is occupied!\nOwner Process: {conflict.ProcessName}\nPID: {conflict.ProcessId}";
                }
                else
                {
                    txtPortResult.Text = $"✅ Port {port} is currently free / unused.";
                }
            }
        };

        gbPort.Controls.Add(lblPort); gbPort.Controls.Add(txtPort); gbPort.Controls.Add(btnCheckPort);
        gbPort.Controls.Add(txtPortResult);

        // Docker Pre-Flight Box
        var gbDocker = new GroupBox
        {
            Text = "Docker Compose Pre-Flight Conflict Check",
            Size = new Size(520, 280),
            Location = new Point(510, 15),
            ForeColor = SystemColors.ControlText
        };

        var lblDockerPath = new Label { Text = "docker-compose.yml:", Location = new Point(15, 30), AutoSize = true };
        var txtDockerPath = new TextBox { Location = new Point(140, 28), Width = 250, BorderStyle = BorderStyle.Fixed3D };
        var btnBrowseDocker = new Button { Text = "Browse...", Location = new Point(400, 26), Width = 80, Height = 26, FlatStyle = FlatStyle.Standard };
        btnBrowseDocker.Click += (s, e) =>
        {
            using var ofd = new OpenFileDialog { Filter = "YAML Files (*.yml;*.yaml)|*.yml;*.yaml|All Files (*.*)|*.*" };
            if (ofd.ShowDialog() == DialogResult.OK) txtDockerPath.Text = ofd.FileName;
        };

        var btnRunDockerCheck = new Button
        {
            Text = "🔍 Check Docker Ports Before Startup",
            Location = new Point(15, 65),
            Size = new Size(465, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };

        var txtDockerResult = new RichTextBox
        {
            Location = new Point(15, 105),
            Size = new Size(465, 155),
            BackColor = SystemColors.Window,
            ForeColor = SystemColors.WindowText,
            BorderStyle = BorderStyle.Fixed3D,
            Font = new Font("Consolas", 9F),
            ReadOnly = true
        };

        btnRunDockerCheck.Click += (s, e) =>
        {
            if (string.IsNullOrWhiteSpace(txtDockerPath.Text) || !File.Exists(txtDockerPath.Text))
            {
                MessageBox.Show("Select a valid docker-compose.yml file first.", "File Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            var conflicts = DockerPortChecker.CheckComposeFile(txtDockerPath.Text);
            if (conflicts.Count == 0)
            {
                txtDockerResult.Text = "No host port mappings detected in compose file.";
                return;
            }

            var sb = new System.Text.StringBuilder();
            sb.AppendLine("Docker Pre-Flight Inspection Results:");
            sb.AppendLine("------------------------------------");

            foreach (var c in conflicts)
            {
                if (c.IsConflicting)
                {
                    sb.AppendLine($"❌ CONFLICT: Host Port {c.HostPort} -> {c.ConflictOwner}");
                }
                else
                {
                    sb.AppendLine($"✅ Host Port {c.HostPort} is available.");
                }
            }

            txtDockerResult.Text = sb.ToString();
        };

        gbDocker.Controls.Add(lblDockerPath); gbDocker.Controls.Add(txtDockerPath); gbDocker.Controls.Add(btnBrowseDocker);
        gbDocker.Controls.Add(btnRunDockerCheck); gbDocker.Controls.Add(txtDockerResult);

        tab.Controls.Add(gbPort);
        tab.Controls.Add(gbDocker);

        return tab;
    }

    private void SetupSystemTray()
    {
        Icon? appIcon = null;
        string icoPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "app.ico");
        string pngPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dotnet.png");

        if (File.Exists(icoPath))
        {
            try { appIcon = new Icon(icoPath); } catch { }
        }
        
        if (appIcon == null && File.Exists(pngPath))
        {
            try
            {
                using var bmp = new Bitmap(pngPath);
                appIcon = Icon.FromHandle(bmp.GetHicon());
            }
            catch { }
        }

        if (appIcon != null)
        {
            this.Icon = appIcon;
        }

        _trayMenu = new ContextMenuStrip();

        var mnuShow = new ToolStripMenuItem("Tampilkan Dashboard", null, (s, e) => RestoreFromTray());
        mnuShow.Font = new Font(mnuShow.Font, FontStyle.Bold);

        var mnuStartAll = new ToolStripMenuItem("▶ Start All Services", null, async (s, e) => await StartAllGroupServices());
        var mnuStopAll = new ToolStripMenuItem("⏹ Stop All Services", null, async (s, e) => await StopAllGroupServices());

        var mnuAutoStart = new ToolStripMenuItem("⚙ Run on Windows Boot", null, (s, e) =>
        {
            var item = (ToolStripMenuItem)s!;
            bool enable = !item.Checked;
            if (StartupManager.SetRunOnStartup(enable))
            {
                item.Checked = enable;
                if (_chkAutoStart != null) _chkAutoStart.Checked = enable;
            }
        })
        { Checked = StartupManager.IsRunOnStartupEnabled() };

        var mnuShortcut = new ToolStripMenuItem("📌 Register in Start Menu", null, (s, e) =>
        {
            if (StartMenuShortcutManager.CreateStartMenuShortcut())
            {
                MessageBox.Show("Start Menu shortcut created! App is now searchable in Windows Search.", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
        });

        var mnuExit = new ToolStripMenuItem("❌ Keluar / Exit", null, (s, e) =>
        {
            _allowClose = true;
            _notifyIcon.Visible = false;
            Application.Exit();
        });

        _trayMenu.Items.Add(mnuShow);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuStartAll);
        _trayMenu.Items.Add(mnuStopAll);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuAutoStart);
        _trayMenu.Items.Add(mnuShortcut);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuExit);

        _notifyIcon = new NotifyIcon
        {
            Icon = appIcon ?? SystemIcons.Application,
            Text = "Dotnet",
            ContextMenuStrip = _trayMenu,
            Visible = true
        };

        _notifyIcon.MouseClick += (s, e) =>
        {
            if (e.Button == MouseButtons.Left)
            {
                RestoreFromTray();
            }
        };

        _notifyIcon.DoubleClick += (s, e) => RestoreFromTray();
    }

    protected override void WndProc(ref Message m)
    {
        if (m.Msg == Program.WM_RESTORE_APP)
        {
            RestoreFromTray();
        }
        base.WndProc(ref m);
    }

    private void RestoreFromTray()
    {
        this.Show();
        this.WindowState = FormWindowState.Normal;
        this.ShowInTaskbar = true;
        this.BringToFront();
        this.Activate();
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        if (!_allowClose && e.CloseReason == CloseReason.UserClosing)
        {
            e.Cancel = true;
            this.Hide();
            this.ShowInTaskbar = false;
            _notifyIcon.ShowBalloonTip(2000, "Dotnet", "Aplikasi berjalan di system tray. Klik icon tray untuk membuka kembali.", ToolTipIcon.Info);
        }
        else
        {
            _notifyIcon.Visible = false;
            base.OnFormClosing(e);
        }
    }
}
