using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Forms;
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

        // Automatically start services marked for Boot Auto-Start
        Task.Run(async () => await AutoStartBootServices());

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
        this.BackColor = Color.FromArgb(30, 30, 30);
        this.ForeColor = Color.White;
        this.Font = new Font("Segoe UI", 9.5F, FontStyle.Regular, GraphicsUnit.Point);

        // Header Panel
        var headerPanel = new Panel
        {
            Dock = DockStyle.Top,
            Height = 60,
            BackColor = Color.FromArgb(45, 45, 48),
            Padding = new Padding(15)
        };

        var titleLabel = new Label
        {
            Text = "Standalone Local Dev Environment Manager",
            Font = new Font("Segoe UI", 11.5F, FontStyle.Bold),
            ForeColor = Color.White,
            AutoSize = true,
            Location = new Point(15, 18)
        };

        bool isAdmin = HostsFileManager.IsElevated();
        _adminStatusLabel = new Label
        {
            Text = isAdmin ? "[ Administrator ]" : "[ Standard User ]",
            ForeColor = isAdmin ? Color.FromArgb(78, 201, 176) : Color.FromArgb(244, 71, 71),
            AutoSize = true,
            Location = new Point(375, 20),
            Font = new Font("Segoe UI", 8.5F, FontStyle.Bold)
        };

        _chkAutoStart = new CheckBox
        {
            Text = "Run on Boot",
            ForeColor = Color.White,
            AutoSize = true,
            Location = new Point(530, 20),
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
            BackColor = Color.FromArgb(0, 122, 204),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(100, 32),
            Location = new Point(680, 14),
            Cursor = Cursors.Hand
        };
        btnStartAll.FlatAppearance.BorderSize = 0;
        btnStartAll.Click += async (s, e) => await StartAllGroupServices();

        var btnStopAll = new Button
        {
            Text = "⏹ Stop All",
            BackColor = Color.FromArgb(180, 40, 40),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(100, 32),
            Location = new Point(790, 14),
            Cursor = Cursors.Hand
        };
        btnStopAll.FlatAppearance.BorderSize = 0;
        btnStopAll.Click += async (s, e) => await StopAllGroupServices();

        var btnRefresh = new Button
        {
            Text = "🔄 Refresh",
            BackColor = Color.FromArgb(60, 60, 65),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(90, 32),
            Location = new Point(900, 14),
            Cursor = Cursors.Hand
        };
        btnRefresh.FlatAppearance.BorderSize = 0;
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
            BackColor = Color.FromArgb(25, 25, 25),
            Padding = new Padding(5)
        };

        var logTitle = new Label
        {
            Text = "Operational Log Console:",
            Dock = DockStyle.Top,
            Height = 22,
            ForeColor = Color.Gray,
            Font = new Font("Segoe UI", 8.5F, FontStyle.Italic)
        };

        _logBox = new RichTextBox
        {
            Dock = DockStyle.Fill,
            BackColor = Color.FromArgb(15, 15, 15),
            ForeColor = Color.FromArgb(78, 201, 176),
            Font = new Font("Consolas", 9F),
            ReadOnly = true,
            BorderStyle = BorderStyle.None
        };

        logPanel.Controls.Add(_logBox);
        logPanel.Controls.Add(logTitle);

        // Tab Control Main Area
        _tabControl = new TabControl
        {
            Dock = DockStyle.Fill,
            Padding = new Point(12, 6)
        };

        // Tab 1: Dashboard
        var tabDashboard = new TabPage("Services Dashboard") { BackColor = Color.FromArgb(30, 30, 30) };
        _servicesPanel = new FlowLayoutPanel
        {
            Dock = DockStyle.Fill,
            AutoScroll = true,
            Padding = new Padding(15)
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
            Size = new Size(1030, 65),
            BackColor = Color.FromArgb(40, 40, 45),
            Margin = new Padding(0, 0, 0, 10),
            Padding = new Padding(10)
        };

        var lblName = new Label
        {
            Text = svc.Name,
            Font = new Font("Segoe UI", 10.5F, FontStyle.Bold),
            ForeColor = Color.White,
            Location = new Point(15, 20),
            AutoSize = true
        };

        var lblPort = new Label
        {
            Text = $"Port: {svc.Port}",
            ForeColor = Color.Gray,
            Location = new Point(250, 22),
            AutoSize = true
        };

        string statusText = svc.Status switch
        {
            ServiceStatus.Running => "● RUNNING",
            ServiceStatus.Stopped => "○ STOPPED",
            ServiceStatus.Starting => "⏳ STARTING",
            ServiceStatus.Stopping => "⏳ STOPPING",
            _ => "❓ UNKNOWN"
        };

        Color statusColor = svc.Status switch
        {
            ServiceStatus.Running => Color.FromArgb(78, 201, 176),
            ServiceStatus.Stopped => Color.FromArgb(244, 71, 71),
            _ => Color.Orange
        };

        var lblStatus = new Label
        {
            Text = statusText,
            ForeColor = statusColor,
            Font = new Font("Segoe UI", 10F, FontStyle.Bold),
            Location = new Point(400, 20),
            AutoSize = true
        };

        var lblPid = new Label
        {
            Text = svc.ProcessId.HasValue ? $"PID: {svc.ProcessId}" : "",
            ForeColor = Color.DarkGray,
            Location = new Point(530, 22),
            AutoSize = true
        };

        var chkAutoBoot = new CheckBox
        {
            Text = "Auto-Start Boot",
            Checked = svc.AutoStartOnBoot,
            ForeColor = Color.FromArgb(200, 200, 200),
            Font = new Font("Segoe UI", 9F),
            Location = new Point(640, 20),
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
            BackColor = svc.Status == ServiceStatus.Running ? Color.FromArgb(160, 50, 50) : Color.FromArgb(40, 130, 70),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(90, 30),
            Location = new Point(790, 15),
            Cursor = Cursors.Hand
        };
        btnToggle.FlatAppearance.BorderSize = 0;
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
            BackColor = Color.FromArgb(70, 70, 75),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(80, 30),
            Location = new Point(890, 15),
            Cursor = Cursors.Hand
        };
        btnRestart.FlatAppearance.BorderSize = 0;
        btnRestart.Click += async (s, e) =>
        {
            btnRestart.Enabled = false;
            await StopSingleService(svc);
            await Task.Delay(1000);
            await StartSingleService(svc);
            RefreshServicesStatus();
        };

        card.Controls.Add(lblName);
        card.Controls.Add(lblPort);
        card.Controls.Add(lblStatus);
        card.Controls.Add(lblPid);
        card.Controls.Add(chkAutoBoot);
        card.Controls.Add(btnToggle);
        card.Controls.Add(btnRestart);

        return card;
    }

    private async Task StartSingleService(DevServiceInfo svc)
    {
        // 1. Refresh status first
        if (svc.Type == DevServiceType.WindowsService)
        {
            svc.Status = WindowsServiceManager.GetStatus(svc.WindowsServiceName);
        }
        else
        {
            svc.Status = ProcessServiceManager.GetStatus(svc);
        }

        if (svc.Status == ServiceStatus.Running)
        {
            // Already running! Nothing to do.
            return;
        }

        // 2. Inspect port occupation
        var conflict = PortConflictDetector.CheckPort(svc.Port);
        if (conflict.IsInUse)
        {
            // Check if the occupying process is actually THIS service itself
            string procName = conflict.ProcessName.ToLowerInvariant();
            bool isOwnProcess = (procName.Contains("mysql") && svc.Id.Contains("mysql"))
                             || (procName.Contains("postgres") && svc.Id.Contains("postgres"))
                             || ((procName.Contains("memurai") || procName.Contains("redis")) && svc.Id.Contains("redis"))
                             || (procName.Contains("nginx") && svc.Id.Contains("nginx"))
                             || (procName.Contains("php") && svc.Id.Contains("php"));

            if (isOwnProcess)
            {
                svc.ProcessId = conflict.ProcessId;
                svc.Status = ServiceStatus.Running;
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
            await WindowsServiceManager.StartServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
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
            await WindowsServiceManager.StopServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
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
            await StartSingleService(svc);
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
        var tab = new TabPage("Projects") { BackColor = Color.FromArgb(30, 30, 30) };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 45, Padding = new Padding(5) };
        var btnAddProject = new Button
        {
            Text = "+ Add Project",
            BackColor = Color.FromArgb(0, 122, 204),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(120, 30),
            Location = new Point(10, 8),
            Cursor = Cursors.Hand
        };
        btnAddProject.FlatAppearance.BorderSize = 0;
        btnAddProject.Click += (s, e) => ShowAddProjectDialog();

        topBar.Controls.Add(btnAddProject);

        _projectsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = Color.FromArgb(25, 25, 25),
            ForeColor = Color.Black,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            MultiSelect = false,
            AllowUserToAddRows = false
        };

        _projectsGrid.Columns.Add("Id", "ID");
        _projectsGrid.Columns["Id"]!.Visible = false;
        _projectsGrid.Columns.Add("Name", "Project Name");
        _projectsGrid.Columns.Add("Path", "Directory Path");
        _projectsGrid.Columns.Add("DevCommand", "Dev Command");
        _projectsGrid.Columns.Add("NginxHost", "Virtual Host");

        // Action Column
        var btnColRun = new DataGridViewButtonColumn
        {
            HeaderText = "Actions",
            Text = "▶ composer run dev",
            UseColumnTextForButtonValue = true
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
            Size = new Size(450, 320),
            StartPosition = FormStartPosition.CenterParent,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = Color.FromArgb(35, 35, 35),
            ForeColor = Color.White
        };

        var lbl1 = new Label { Text = "Project Name:", Location = new Point(20, 20), AutoSize = true };
        var txtName = new TextBox { Location = new Point(150, 18), Width = 250 };

        var lbl2 = new Label { Text = "Project Directory:", Location = new Point(20, 60), AutoSize = true };
        var txtPath = new TextBox { Location = new Point(150, 58), Width = 170 };
        var btnBrowse = new Button { Text = "Browse...", Location = new Point(330, 56), Width = 70 };
        btnBrowse.Click += (s, e) =>
        {
            using var fbd = new FolderBrowserDialog();
            if (fbd.ShowDialog() == DialogResult.OK) txtPath.Text = fbd.SelectedPath;
        };

        var lbl3 = new Label { Text = "Dev Command:", Location = new Point(20, 100), AutoSize = true };
        var txtCmd = new TextBox { Text = "composer run dev", Location = new Point(150, 98), Width = 250 };

        var lbl4 = new Label { Text = "Nginx Host Domain:", Location = new Point(20, 140), AutoSize = true };
        var txtHost = new TextBox { Text = "project.test", Location = new Point(150, 138), Width = 250 };

        var btnSave = new Button
        {
            Text = "Save Project",
            DialogResult = DialogResult.OK,
            Location = new Point(150, 200),
            Width = 120,
            Height = 35,
            BackColor = Color.FromArgb(0, 122, 204),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat
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
        var tab = new TabPage("Hosts File") { BackColor = Color.FromArgb(30, 30, 30) };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 45, Padding = new Padding(5) };
        var btnAddHost = new Button
        {
            Text = "+ Add Host Entry",
            BackColor = Color.FromArgb(0, 122, 204),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(130, 30),
            Location = new Point(10, 8),
            Cursor = Cursors.Hand
        };
        btnAddHost.FlatAppearance.BorderSize = 0;
        btnAddHost.Click += (s, e) => ShowAddHostDialog();

        var btnReloadHosts = new Button
        {
            Text = "🔄 Reload Hosts",
            BackColor = Color.FromArgb(60, 60, 65),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat,
            Size = new Size(120, 30),
            Location = new Point(150, 8),
            Cursor = Cursors.Hand
        };
        btnReloadHosts.FlatAppearance.BorderSize = 0;
        btnReloadHosts.Click += (s, e) => RefreshHostsGrid();

        topBar.Controls.Add(btnAddHost);
        topBar.Controls.Add(btnReloadHosts);

        _hostsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = Color.FromArgb(25, 25, 25),
            ForeColor = Color.Black,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            AllowUserToAddRows = false
        };

        _hostsGrid.Columns.Add("Ip", "IP Address");
        _hostsGrid.Columns.Add("Hostname", "Domain / Hostname");
        _hostsGrid.Columns.Add("Enabled", "Enabled");

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
            _hostsGrid.Rows.Add(entry.IpAddress, entry.Hostname, entry.IsEnabled ? "Yes" : "No (#)");
        }
    }

    private void ShowAddHostDialog()
    {
        using var dlg = new Form
        {
            Text = "Add Hosts Entry",
            Size = new Size(400, 220),
            StartPosition = FormStartPosition.CenterParent,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = Color.FromArgb(35, 35, 35),
            ForeColor = Color.White
        };

        var lbl1 = new Label { Text = "IP Address:", Location = new Point(20, 20), AutoSize = true };
        var txtIp = new TextBox { Text = "127.0.0.1", Location = new Point(120, 18), Width = 220 };

        var lbl2 = new Label { Text = "Hostname:", Location = new Point(20, 60), AutoSize = true };
        var txtHost = new TextBox { Text = "myproject.test", Location = new Point(120, 58), Width = 220 };

        var btnSave = new Button
        {
            Text = "Add Entry",
            DialogResult = DialogResult.OK,
            Location = new Point(120, 110),
            Width = 110,
            Height = 35,
            BackColor = Color.FromArgb(0, 122, 204),
            ForeColor = Color.White,
            FlatStyle = FlatStyle.Flat
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
        var tab = new TabPage("PHP & Nginx Config") { BackColor = Color.FromArgb(30, 30, 30) };

        var pnlPhp = new GroupBox
        {
            Text = "PHP Configuration (php.ini)",
            Size = new Size(500, 350),
            Location = new Point(20, 20),
            ForeColor = Color.White
        };

        var lblPath = new Label { Text = $"Path: {PhpConfigManager.GetIniPath()}", Location = new Point(20, 30), AutoSize = true, ForeColor = Color.Gray };

        var l1 = new Label { Text = "memory_limit:", Location = new Point(20, 70), AutoSize = true };
        _phpMemoryLimit = new TextBox { Location = new Point(180, 68), Width = 150 };

        var l2 = new Label { Text = "upload_max_filesize:", Location = new Point(20, 110), AutoSize = true };
        _phpUploadMax = new TextBox { Location = new Point(180, 108), Width = 150 };

        var l3 = new Label { Text = "post_max_size:", Location = new Point(20, 150), AutoSize = true };
        _phpPostMax = new TextBox { Location = new Point(180, 148), Width = 150 };

        var l4 = new Label { Text = "max_execution_time:", Location = new Point(20, 190), AutoSize = true };
        _phpMaxExec = new TextBox { Location = new Point(180, 188), Width = 150 };

        var btnLoadPhp = new Button { Text = "Read INI", Location = new Point(20, 250), Width = 100, Height = 32, BackColor = Color.FromArgb(60, 60, 65), FlatStyle = FlatStyle.Flat };
        btnLoadPhp.Click += (s, e) => LoadPhpIniValues();

        var btnSavePhp = new Button { Text = "Save php.ini", Location = new Point(140, 250), Width = 120, Height = 32, BackColor = Color.FromArgb(0, 122, 204), FlatStyle = FlatStyle.Flat };
        btnSavePhp.Click += (s, e) => SavePhpIniValues();

        var btnOpenPhpNotepad = new Button 
        { 
            Text = "📝 Edit in Notepad", 
            Location = new Point(280, 250), 
            Width = 160, 
            Height = 32, 
            BackColor = Color.FromArgb(40, 130, 70), 
            FlatStyle = FlatStyle.Flat,
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
            Size = new Size(500, 350),
            Location = new Point(540, 20),
            ForeColor = Color.White
        };

        var btnTestNginx = new Button
        {
            Text = "🧪 Validate Syntax (nginx -t)",
            Location = new Point(30, 45),
            Size = new Size(220, 38),
            BackColor = Color.FromArgb(60, 60, 65),
            FlatStyle = FlatStyle.Flat,
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
            Location = new Point(30, 95),
            Size = new Size(220, 38),
            BackColor = Color.FromArgb(0, 122, 204),
            FlatStyle = FlatStyle.Flat,
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
            Location = new Point(30, 150),
            Size = new Size(220, 38),
            BackColor = Color.FromArgb(40, 130, 70),
            FlatStyle = FlatStyle.Flat,
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
            Location = new Point(30, 200),
            Size = new Size(220, 38),
            BackColor = Color.FromArgb(60, 60, 65),
            FlatStyle = FlatStyle.Flat,
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
        var tab = new TabPage("Port & Docker Pre-Flight") { BackColor = Color.FromArgb(30, 30, 30) };

        var gbPort = new GroupBox
        {
            Text = "Inspect Specific Port Owner",
            Size = new Size(480, 300),
            Location = new Point(20, 20),
            ForeColor = Color.White
        };

        var lblPort = new Label { Text = "Port Number:", Location = new Point(20, 40), AutoSize = true };
        var txtPort = new TextBox { Text = "3306", Location = new Point(130, 38), Width = 100 };
        var btnCheckPort = new Button { Text = "Inspect Port", Location = new Point(250, 36), Width = 100, Height = 28, BackColor = Color.FromArgb(0, 122, 204), FlatStyle = FlatStyle.Flat };

        var txtPortResult = new RichTextBox
        {
            Location = new Point(20, 80),
            Size = new Size(440, 190),
            BackColor = Color.FromArgb(15, 15, 15),
            ForeColor = Color.FromArgb(78, 201, 176),
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
            Size = new Size(520, 300),
            Location = new Point(520, 20),
            ForeColor = Color.White
        };

        var lblDockerPath = new Label { Text = "docker-compose.yml:", Location = new Point(20, 35), AutoSize = true };
        var txtDockerPath = new TextBox { Location = new Point(160, 33), Width = 230 };
        var btnBrowseDocker = new Button { Text = "Browse...", Location = new Point(400, 31), Width = 90 };
        btnBrowseDocker.Click += (s, e) =>
        {
            using var ofd = new OpenFileDialog { Filter = "YAML Files (*.yml;*.yaml)|*.yml;*.yaml|All Files (*.*)|*.*" };
            if (ofd.ShowDialog() == DialogResult.OK) txtDockerPath.Text = ofd.FileName;
        };

        var btnRunDockerCheck = new Button
        {
            Text = "🔍 Check Docker Ports Before Startup",
            Location = new Point(20, 75),
            Size = new Size(470, 32),
            BackColor = Color.FromArgb(0, 122, 204),
            FlatStyle = FlatStyle.Flat,
            Cursor = Cursors.Hand
        };

        var txtDockerResult = new RichTextBox
        {
            Location = new Point(20, 120),
            Size = new Size(470, 160),
            BackColor = Color.FromArgb(15, 15, 15),
            ForeColor = Color.FromArgb(78, 201, 176),
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
