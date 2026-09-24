using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Forms;
using dotnet.Catalog;
using dotnet.Config;
using dotnet.Env;
using dotnet.Installation;
using dotnet.Models;
using dotnet.Persistence;
using dotnet.Services;

namespace dotnet;

public partial class Form1 : Form
{
    private readonly List<DevServiceInfo> _services = new();
    private readonly CatalogLoader _catalogLoader = new();
    private readonly InstalledToolStore _toolStore = new();
    private TabControl _tabControl = null!;
    private RichTextBox _logBox = null!;
    private FlowLayoutPanel _servicesPanel = null!;
    private DataGridView _toolsGrid = null!;
    private Label _lblToolName = null!;
    private Label _lblToolCategory = null!;
    private Label _lblToolLicense = null!;
    private Label _lblToolStatus = null!;
    private Label _lblToolInstallPath = null!;
    private ComboBox _cboToolVersion = null!;
    private Button _btnInstallTool = null!;
    private Button _btnUninstallTool = null!;
    private Button _btnAdoptTool = null!;
    private ProgressBar _toolProgressBar = null!;
    private Label _toolProgressLabel = null!;
    private ToolDefinition? _selectedTool = null;
    private DataGridView _projectsGrid = null!;
    private DataGridView _hostsGrid = null!;
    private TextBox _phpMemoryLimit = null!;
    private TextBox _phpUploadMax = null!;
    private TextBox _phpPostMax = null!;
    private TextBox _phpMaxExec = null!;
    private TextBox _phpTimezone = null!;
    private CheckedListBox _lstPhpExtensions = null!;
    private NumericUpDown _numPhpPoolSize = null!;
    private RichTextBox _txtNginxErrorLog = null!;
    private Label _adminStatusLabel = null!;
    private ComboBox _cboProfile = null!;
    private DataGridView _gridActivePorts = null!;
    private DataGridView _gridShadow = null!;
    private NotifyIcon _notifyIcon = null!;
    private ContextMenuStrip _trayMenu = null!;
    private CheckBox _chkAutoStart = null!;
    private bool _allowClose = false;
    private bool _startMinimized = false;

    public Form1(bool startMinimized = false)
    {
        _startMinimized = startMinimized;
        _catalogLoader.LoadCatalog();
        _toolStore.Load();
        InitializeComponent();
        SetupServicesList();
        ProcessServiceManager.TryAdoptAll(_services);
        ProcessTracker.ProcessCrashed += OnProcessCrashed;
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

    private void OnProcessCrashed(string serviceId, int exitCode)
    {
        if (InvokeRequired)
        {
            Invoke(new Action<string, int>(OnProcessCrashed), serviceId, exitCode);
            return;
        }

        var svc = _services.FirstOrDefault(s => s.Id.Equals(serviceId, StringComparison.OrdinalIgnoreCase));
        string name = svc?.Name ?? serviceId;
        AppLogger.Log($"[ALERT] Service '{name}' exited unexpectedly with code {exitCode}!");
        _notifyIcon.ShowBalloonTip(3000, "Layanan Berhenti / Crash", $"Layanan '{name}' berhenti tidak terduga (Kode: {exitCode})", ToolTipIcon.Warning);
        RefreshServicesStatus();
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

    private void DetectDatabaseExecutionMode(DevServiceInfo svc)
    {
        string toolsDir = AppPaths.GetPath("tools");
        string portableExe = "";

        if (svc.Id == "mysql")
        {
            var tool = _toolStore.GetTool("mysql");
            if (tool != null && !string.IsNullOrEmpty(tool.InstallPath))
            {
                portableExe = Path.Combine(tool.InstallPath, "bin", "mysqld.exe");
            }
            else
            {
                string cand = Path.Combine(toolsDir, "mysql", "current", "bin", "mysqld.exe");
                if (File.Exists(cand)) portableExe = cand;
                else if (File.Exists(@"C:\tools\mysql84\bin\mysqld.exe")) portableExe = @"C:\tools\mysql84\bin\mysqld.exe";
            }
        }
        else if (svc.Id == "postgres")
        {
            var tool = _toolStore.GetTool("postgres");
            if (tool != null && !string.IsNullOrEmpty(tool.InstallPath))
            {
                portableExe = Path.Combine(tool.InstallPath, "bin", "postgres.exe");
            }
            else
            {
                string cand = Path.Combine(toolsDir, "postgres", "current", "bin", "postgres.exe");
                if (File.Exists(cand)) portableExe = cand;
                else if (File.Exists(@"C:\tools\pgsql\bin\postgres.exe")) portableExe = @"C:\tools\pgsql\bin\postgres.exe";
            }
        }
        else if (svc.Id == "redis")
        {
            var tool = _toolStore.GetTool("redis");
            if (tool != null && !string.IsNullOrEmpty(tool.InstallPath))
            {
                portableExe = Path.Combine(tool.InstallPath, "redis-server.exe");
            }
            else
            {
                string cand = Path.Combine(toolsDir, "redis", "current", "redis-server.exe");
                if (File.Exists(cand)) portableExe = cand;
                else if (File.Exists(@"C:\tools\redis\redis-server.exe")) portableExe = @"C:\tools\redis\redis-server.exe";
            }
        }

        if (!string.IsNullOrEmpty(portableExe) && File.Exists(portableExe))
        {
            svc.Type = DevServiceType.ManagedProcess;
            svc.ExecutablePath = portableExe;
            svc.IsPortable = true;
            svc.DataDirectory = DatabaseInitializer.GetDataDirectory(svc.Id);
            svc.Arguments = string.Join(" ", DatabaseInitializer.BuildRuntimeArguments(svc.Id, svc.Port));
            svc.WorkingDirectory = Path.GetDirectoryName(portableExe) ?? "";
        }
        else
        {
            svc.Type = DevServiceType.WindowsService;
            svc.IsPortable = false;
        }
    }

    private void SetupServicesList()
    {
        string toolsDir = AppPaths.GetPath("tools");

        var mysqlSvc = new DevServiceInfo
        {
            Id = "mysql",
            Name = "MySQL 8.4",
            WindowsServiceName = "MySQL84",
            Port = 3306,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("mysql", true)
        };
        DetectDatabaseExecutionMode(mysqlSvc);
        _services.Add(mysqlSvc);

        var postgresSvc = new DevServiceInfo
        {
            Id = "postgres",
            Name = "PostgreSQL 17",
            WindowsServiceName = "postgresql-x64-17",
            Port = 5432,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("postgres", true)
        };
        DetectDatabaseExecutionMode(postgresSvc);
        _services.Add(postgresSvc);

        var redisSvc = new DevServiceInfo
        {
            Id = "redis",
            Name = "Redis (Memurai)",
            WindowsServiceName = "Memurai",
            Port = 6379,
            AutoStartWithGroup = true,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("redis", true)
        };
        DetectDatabaseExecutionMode(redisSvc);
        _services.Add(redisSvc);

        string nginxExe = @"C:\tools\nginx\nginx.exe";
        var nginxTool = _toolStore.GetTool("nginx");
        if (nginxTool != null && File.Exists(Path.Combine(nginxTool.InstallPath, "nginx.exe")))
        {
            nginxExe = Path.Combine(nginxTool.InstallPath, "nginx.exe");
        }
        else if (File.Exists(Path.Combine(toolsDir, "nginx", "current", "nginx.exe")))
        {
            nginxExe = Path.Combine(toolsDir, "nginx", "current", "nginx.exe");
        }

        _services.Add(new DevServiceInfo
        {
            Id = "nginx",
            Name = "Nginx Web Server",
            Type = DevServiceType.ManagedProcess,
            ExecutablePath = nginxExe,
            Port = 80,
            AutoStartWithGroup = false,
            AutoStartOnBoot = ServiceSettingsManager.GetAutoStartOnBoot("nginx", false)
        });

        string phpCgiExe = @"C:\tools\php85\php-cgi.exe";
        var phpTool = _toolStore.GetTool("php");
        if (phpTool != null && File.Exists(Path.Combine(phpTool.InstallPath, "php-cgi.exe")))
        {
            phpCgiExe = Path.Combine(phpTool.InstallPath, "php-cgi.exe");
        }
        else if (File.Exists(Path.Combine(toolsDir, "php", "current", "php-cgi.exe")))
        {
            phpCgiExe = Path.Combine(toolsDir, "php", "current", "php-cgi.exe");
        }

        _services.Add(new DevServiceInfo
        {
            Id = "php-cgi",
            Name = "PHP 8.5 FastCGI",
            Type = DevServiceType.ManagedProcess,
            ExecutablePath = phpCgiExe,
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
            Location = new Point(315, 18),
            Font = new Font("Tahoma", 8.25F, FontStyle.Bold)
        };

        var lblProfile = new Label
        {
            Text = "Profile:",
            AutoSize = true,
            Location = new Point(445, 18),
            Font = new Font("Tahoma", 8.25F, FontStyle.Bold)
        };

        _cboProfile = new ComboBox
        {
            Location = new Point(495, 15),
            Width = 145,
            DropDownStyle = ComboBoxStyle.DropDownList,
            FlatStyle = FlatStyle.Standard
        };
        foreach (var prof in ProfileStore.Profiles)
        {
            _cboProfile.Items.Add(prof.Name);
        }
        var activeProf = ProfileStore.GetActiveProfile();
        _cboProfile.SelectedItem = activeProf.Name;
        _cboProfile.SelectedIndexChanged += (s, e) =>
        {
            var selected = ProfileStore.Profiles.FirstOrDefault(p => p.Name == _cboProfile.SelectedItem?.ToString());
            if (selected != null && selected.Id != ProfileStore.ActiveProfileId)
            {
                ProfileStore.ActiveProfileId = selected.Id;
                AppLogger.Log($"Switched active service profile to: {selected.Name}");
                RefreshServicesStatus();
                UpdateTrayMenu();
            }
        };

        _chkAutoStart = new CheckBox
        {
            Text = "Run on Boot",
            ForeColor = SystemColors.ControlText,
            AutoSize = true,
            Location = new Point(650, 18),
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
            Size = new Size(80, 28),
            Location = new Point(755, 13),
            Cursor = Cursors.Hand
        };
        btnStartAll.Click += async (s, e) => await StartAllGroupServices();

        var btnStopAll = new Button
        {
            Text = "⏹ Stop All",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(80, 28),
            Location = new Point(840, 13),
            Cursor = Cursors.Hand
        };
        btnStopAll.Click += async (s, e) => await StopAllGroupServices();

        var btnRefresh = new Button
        {
            Text = "🔄 Refresh",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(80, 28),
            Location = new Point(925, 13),
            Cursor = Cursors.Hand
        };
        btnRefresh.Click += (s, e) => RefreshServicesStatus();

        var btnAbout = new Button
        {
            Text = "ℹ️ About",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(68, 28),
            Location = new Point(1010, 13),
            Cursor = Cursors.Hand
        };
        btnAbout.Click += (s, e) =>
        {
            using var dlg = new AboutDialog();
            dlg.ShowDialog(this);
        };

        headerPanel.Controls.Add(titleLabel);
        headerPanel.Controls.Add(_adminStatusLabel);
        headerPanel.Controls.Add(lblProfile);
        headerPanel.Controls.Add(_cboProfile);
        headerPanel.Controls.Add(_chkAutoStart);
        headerPanel.Controls.Add(btnStartAll);
        headerPanel.Controls.Add(btnStopAll);
        headerPanel.Controls.Add(btnRefresh);
        headerPanel.Controls.Add(btnAbout);

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

        // Tab 2: Tools & Packages
        var tabTools = CreateToolsTab();

        // Tab 3: Projects
        var tabProjects = CreateProjectsTab();

        // Tab 4: Hosts File Manager
        var tabHosts = CreateHostsTab();

        // Tab 5: PHP & Nginx Configuration
        var tabConfig = CreateConfigTab();

        // Tab 6: Port Diagnostics & Docker Check
        var tabDiagnostics = CreateDiagnosticsTab();

        _tabControl.TabPages.Add(tabDashboard);
        _tabControl.TabPages.Add(tabTools);
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
            if (svc.Id == "mysql" || svc.Id == "postgres" || svc.Id == "redis")
            {
                DetectDatabaseExecutionMode(svc);
            }

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

        UpdateTrayMenu();
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

        var lblMode = new Label
        {
            Text = svc.IsPortable ? "[PORTABLE]" : "[WIN-SERVICE]",
            ForeColor = svc.IsPortable ? Color.DarkBlue : Color.DarkSlateGray,
            Font = new Font("Tahoma", 7.5F, FontStyle.Bold),
            Location = new Point(175, 20),
            AutoSize = true
        };

        var lblPort = new Label
        {
            Text = $"Port: {svc.Port}",
            ForeColor = SystemColors.ControlDarkDark,
            Location = new Point(275, 20),
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
            Location = new Point(365, 19),
            AutoSize = true
        };

        var lblPid = new Label
        {
            Text = svc.ProcessId.HasValue ? $"PID: {svc.ProcessId}" : "",
            ForeColor = SystemColors.ControlDarkDark,
            Location = new Point(480, 20),
            AutoSize = true
        };

        var activeProfile = ProfileStore.GetActiveProfile();
        bool isProfileTarget = activeProfile.ServiceIds.Contains(svc.Id, StringComparer.OrdinalIgnoreCase);
        var lblProfileTag = new Label
        {
            Text = isProfileTarget ? "[Target]" : "[Optional]",
            ForeColor = isProfileTarget ? Color.Navy : Color.Gray,
            Font = new Font("Tahoma", 7.5F, FontStyle.Regular),
            Location = new Point(555, 21),
            AutoSize = true
        };

        bool needsInit = svc.IsPortable && (svc.Id == "mysql" || svc.Id == "postgres") && !DatabaseInitializer.IsInitialized(svc.Id);
        if (needsInit)
        {
            var btnInitDb = new Button
            {
                Text = "⚡ Init DB",
                FlatStyle = FlatStyle.Standard,
                Size = new Size(68, 26),
                Location = new Point(625, 14),
                Cursor = Cursors.Hand
            };
            btnInitDb.Click += async (s, e) =>
            {
                btnInitDb.Enabled = false;
                var res = await DatabaseInitializer.InitializeAsync(svc.Id, svc.ExecutablePath);
                MessageBox.Show(res.Message, res.Success ? "Database Initialized" : "Init Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
                RefreshServicesStatus();
            };
            card.Controls.Add(btnInitDb);
        }

        var chkAutoBoot = new CheckBox
        {
            Text = "Auto-Boot",
            Checked = svc.AutoStartOnBoot,
            ForeColor = SystemColors.ControlText,
            Font = new Font("Tahoma", 8.25F),
            Location = new Point(700, 19),
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
            Size = new Size(75, 26),
            Location = new Point(790, 14),
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
        card.Controls.Add(lblMode);
        card.Controls.Add(lblPort);
        card.Controls.Add(lblStatus);
        card.Controls.Add(lblPid);
        card.Controls.Add(lblProfileTag);
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
            string localIni = Path.Combine(svc.DataDirectory ?? "", "my.ini");
            path = File.Exists(localIni) ? localIni : @"C:\tools\mysql84\my.ini";
        }
        else if (svc.Id == "postgres")
        {
            path = Path.Combine(svc.DataDirectory ?? "", "postgresql.conf");
        }
        else if (svc.Id == "redis")
        {
            path = Path.Combine(AppPaths.GetPath("tools/redis/current"), "redis.conf");
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

    private async Task<bool> StartSingleService(DevServiceInfo svc)
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
                return true;
            }

            MessageBox.Show(
                $"Port conflict detected!\nPort {svc.Port} is already occupied by external process: {conflict.ProcessName} (PID {conflict.ProcessId}).\nPlease stop the conflicting process first.",
                "Port Conflict Warning",
                MessageBoxButtons.OK,
                MessageBoxIcon.Warning
            );
            return false;
        }

        if (svc.Type == DevServiceType.WindowsService)
        {
            var res = await WindowsServiceManager.StartServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
            if (!res.Success)
            {
                MessageBox.Show(res.Message, "Service Start Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return false;
            }
            return true;
        }
        else
        {
            return ProcessServiceManager.StartProcess(svc);
        }
    }

    private async Task<bool> StopSingleService(DevServiceInfo svc)
    {
        if (svc.Type == DevServiceType.WindowsService)
        {
            var res = await WindowsServiceManager.StopServiceAsync(svc.WindowsServiceName, TimeSpan.FromSeconds(10));
            if (!res.Success)
            {
                MessageBox.Show(res.Message, "Service Stop Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return false;
            }
            return true;
        }
        else
        {
            return ProcessServiceManager.StopProcess(svc);
        }
    }

    private async Task StartAllGroupServices()
    {
        var profile = ProfileStore.GetActiveProfile();
        AppLogger.Log($"Executing Start All Services for profile '{profile.Name}'...");
        await ServiceOrchestrator.StartProfileServicesAsync(_services, profile, StartSingleService);
        RefreshServicesStatus();
    }

    private async Task StopAllGroupServices()
    {
        var profile = ProfileStore.GetActiveProfile();
        AppLogger.Log($"Executing Stop All Services for profile '{profile.Name}'...");
        await ServiceOrchestrator.StopProfileServicesAsync(_services, profile, StopSingleService);
        RefreshServicesStatus();
    }

    // TAB 2: TOOLS & PACKAGES
    private TabPage CreateToolsTab()
    {
        var tab = new TabPage("Tools & Packages") { BackColor = SystemColors.Control };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 40, Padding = new Padding(5) };
        var btnScanAdopt = new Button
        {
            Text = "🔍 Scan & Adopt Local Tools",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(185, 26),
            Location = new Point(8, 6),
            Cursor = Cursors.Hand
        };
        btnScanAdopt.Click += (s, e) => ScanAndAdoptLocalTools();

        var btnRefreshTools = new Button
        {
            Text = "🔄 Refresh Catalog",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(130, 26),
            Location = new Point(200, 6),
            Cursor = Cursors.Hand
        };
        btnRefreshTools.Click += (s, e) => RefreshToolsGrid();

        topBar.Controls.Add(btnScanAdopt);
        topBar.Controls.Add(btnRefreshTools);

        // Bottom Progress Panel
        var bottomPanel = new Panel { Dock = DockStyle.Bottom, Height = 36, Padding = new Padding(8, 6, 8, 6) };
        _toolProgressBar = new ProgressBar { Location = new Point(8, 8), Size = new Size(350, 18), Visible = false };
        _toolProgressLabel = new Label { Location = new Point(365, 9), AutoSize = true, Text = "Ready", ForeColor = SystemColors.ControlDarkDark };
        bottomPanel.Controls.Add(_toolProgressBar);
        bottomPanel.Controls.Add(_toolProgressLabel);

        // Right details panel
        var detailsPanel = new Panel { Dock = DockStyle.Right, Width = 450, Padding = new Padding(10) };
        var grp = new GroupBox
        {
            Text = "Tool Information & Operations",
            Dock = DockStyle.Fill,
            ForeColor = SystemColors.ControlText,
            Padding = new Padding(15)
        };

        var l1 = new Label { Text = "Tool Name:", Location = new Point(15, 30), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };
        _lblToolName = new Label { Text = "-", Location = new Point(110, 30), AutoSize = true };

        var l2 = new Label { Text = "Category:", Location = new Point(15, 60), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };
        _lblToolCategory = new Label { Text = "-", Location = new Point(110, 60), AutoSize = true };

        var l3 = new Label { Text = "License:", Location = new Point(15, 90), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };
        _lblToolLicense = new Label { Text = "-", Location = new Point(110, 90), AutoSize = true };

        var l4 = new Label { Text = "Status:", Location = new Point(15, 120), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };
        _lblToolStatus = new Label { Text = "-", Location = new Point(110, 120), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };

        var l5 = new Label { Text = "Install Path:", Location = new Point(15, 150), AutoSize = true, Font = new Font("Tahoma", 8.25F, FontStyle.Bold) };
        _lblToolInstallPath = new Label { Text = "-", Location = new Point(110, 150), AutoSize = true, MaximumSize = new Size(310, 40) };

        var l6 = new Label { Text = "Select Version:", Location = new Point(15, 195), AutoSize = true };
        _cboToolVersion = new ComboBox { Location = new Point(110, 193), Width = 180, DropDownStyle = ComboBoxStyle.DropDownList };

        _btnInstallTool = new Button
        {
            Text = "⬇ Download & Install",
            Location = new Point(15, 235),
            Size = new Size(160, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        _btnInstallTool.Click += async (s, e) => await InstallSelectedToolAsync();

        _btnUninstallTool = new Button
        {
            Text = "🗑 Uninstall / Remove",
            Location = new Point(185, 235),
            Size = new Size(150, 30),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand,
            Enabled = false
        };
        _btnUninstallTool.Click += (s, e) => UninstallSelectedTool();

        _btnAdoptTool = new Button
        {
            Text = "📂 Adopt Custom Local Folder...",
            Location = new Point(15, 275),
            Size = new Size(220, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        _btnAdoptTool.Click += (s, e) => AdoptCustomLocalFolder();

        grp.Controls.Add(l1); grp.Controls.Add(_lblToolName);
        grp.Controls.Add(l2); grp.Controls.Add(_lblToolCategory);
        grp.Controls.Add(l3); grp.Controls.Add(_lblToolLicense);
        grp.Controls.Add(l4); grp.Controls.Add(_lblToolStatus);
        grp.Controls.Add(l5); grp.Controls.Add(_lblToolInstallPath);
        grp.Controls.Add(l6); grp.Controls.Add(_cboToolVersion);
        grp.Controls.Add(_btnInstallTool);
        grp.Controls.Add(_btnUninstallTool);
        grp.Controls.Add(_btnAdoptTool);
        detailsPanel.Controls.Add(grp);

        // Center / Left Grid
        _toolsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = SystemColors.Window,
            ForeColor = SystemColors.ControlText,
            BorderStyle = BorderStyle.Fixed3D,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            MultiSelect = false,
            AllowUserToAddRows = false
        };

        _toolsGrid.Columns.Add("Id", "ID");
        if (_toolsGrid.Columns["Id"] != null) _toolsGrid.Columns["Id"]!.Visible = false;
        _toolsGrid.Columns.Add("Name", "Tool Name");
        _toolsGrid.Columns.Add("Category", "Category");
        _toolsGrid.Columns.Add("Version", "Active Version");
        _toolsGrid.Columns.Add("Status", "Status");

        _toolsGrid.SelectionChanged += (s, e) => OnToolSelected();

        RefreshToolsGrid();

        tab.Controls.Add(_toolsGrid);
        tab.Controls.Add(detailsPanel);
        tab.Controls.Add(bottomPanel);
        tab.Controls.Add(topBar);

        return tab;
    }

    private void RefreshToolsGrid()
    {
        _toolStore.Load();
        _catalogLoader.LoadCatalog();
        _toolsGrid.Rows.Clear();

        foreach (var tool in _catalogLoader.Tools)
        {
            var installed = _toolStore.GetTool(tool.Id);
            string versionStr = installed != null ? installed.Version : "-";
            string statusStr = "Not Installed";
            if (installed != null)
            {
                statusStr = installed.IsAdopted ? "Adopted (Local)" : "Installed";
            }

            int rowIdx = _toolsGrid.Rows.Add(tool.Id, tool.DisplayName, tool.Category, versionStr, statusStr);
            if (installed != null)
            {
                _toolsGrid.Rows[rowIdx].DefaultCellStyle.ForeColor = installed.IsAdopted ? Color.DarkBlue : Color.DarkGreen;
            }
            else
            {
                _toolsGrid.Rows[rowIdx].DefaultCellStyle.ForeColor = Color.Gray;
            }
        }

        if (_toolsGrid.Rows.Count > 0)
        {
            _toolsGrid.Rows[0].Selected = true;
            OnToolSelected();
        }
    }

    private void OnToolSelected()
    {
        if (_toolsGrid.SelectedRows.Count == 0) return;
        string toolId = _toolsGrid.SelectedRows[0].Cells["Id"].Value?.ToString() ?? "";
        _selectedTool = _catalogLoader.GetTool(toolId);
        if (_selectedTool == null) return;

        var installed = _toolStore.GetTool(_selectedTool.Id);

        _lblToolName.Text = _selectedTool.DisplayName;
        _lblToolCategory.Text = _selectedTool.Category;
        _lblToolLicense.Text = _selectedTool.License;

        if (installed != null)
        {
            _lblToolStatus.Text = installed.IsAdopted ? "Adopted (Local)" : "Installed";
            _lblToolStatus.ForeColor = installed.IsAdopted ? Color.DarkBlue : Color.DarkGreen;
            _lblToolInstallPath.Text = installed.InstallPath;
            _btnUninstallTool.Enabled = true;
        }
        else
        {
            _lblToolStatus.Text = "Not Installed";
            _lblToolStatus.ForeColor = Color.Gray;
            _lblToolInstallPath.Text = "-";
            _btnUninstallTool.Enabled = false;
        }

        _cboToolVersion.Items.Clear();
        foreach (var v in _selectedTool.Versions)
        {
            _cboToolVersion.Items.Add(v.Version);
        }
        if (_cboToolVersion.Items.Count > 0)
        {
            _cboToolVersion.SelectedIndex = 0;
        }
    }

    private async Task InstallSelectedToolAsync()
    {
        if (_selectedTool == null || _cboToolVersion.SelectedItem == null)
        {
            MessageBox.Show("Please select a tool and version.", "Info", MessageBoxButtons.OK, MessageBoxIcon.Information);
            return;
        }

        string versionStr = _cboToolVersion.SelectedItem.ToString() ?? "";
        var ver = _selectedTool.Versions.FirstOrDefault(v => v.Version == versionStr);
        if (ver == null) return;

        var confirm = MessageBox.Show(
            $"Install {_selectedTool.DisplayName} version {versionStr}?\n\nThis will download the package, verify checksum, extract to %LOCALAPPDATA%\\Dotnet\\tools, and configure User PATH.",
            "Confirm Installation",
            MessageBoxButtons.YesNo,
            MessageBoxIcon.Question);

        if (confirm != DialogResult.Yes) return;

        _btnInstallTool.Enabled = false;
        _toolProgressBar.Value = 0;
        _toolProgressBar.Visible = true;
        _toolProgressLabel.Text = $"Starting download for {_selectedTool.DisplayName}...";

        var progress = new Progress<double>(percent =>
        {
            if (InvokeRequired)
            {
                Invoke(new Action(() =>
                {
                    _toolProgressBar.Value = Math.Min(100, Math.Max(0, (int)percent));
                    _toolProgressLabel.Text = $"Downloading/Extracting: {(int)percent}%";
                }));
            }
            else
            {
                _toolProgressBar.Value = Math.Min(100, Math.Max(0, (int)percent));
                _toolProgressLabel.Text = $"Downloading/Extracting: {(int)percent}%";
            }
        });

        bool success = await Task.Run(async () =>
        {
            return await ToolInstaller.InstallAsync(_selectedTool, ver, _toolStore, progress);
        });

        _btnInstallTool.Enabled = true;
        _toolProgressBar.Visible = false;

        if (success)
        {
            _toolProgressLabel.Text = $"Installation of {_selectedTool.DisplayName} succeeded!";
            RefreshToolsGrid();
            MessageBox.Show($"{_selectedTool.DisplayName} installed successfully!\nAdded to User PATH. Open a new terminal to use it.", "Installation Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
        else
        {
            _toolProgressLabel.Text = "Installation failed. Check operational log.";
            MessageBox.Show($"Failed to install {_selectedTool.DisplayName}. See log for details.", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }

    private void UninstallSelectedTool()
    {
        if (_selectedTool == null) return;
        var installed = _toolStore.GetTool(_selectedTool.Id);
        if (installed == null) return;

        var confirm = MessageBox.Show(
            $"Uninstall / Remove {installed.DisplayName} v{installed.Version}?\nThis will remove PATH entries and unregister the tool.",
            "Confirm Uninstall",
            MessageBoxButtons.YesNo,
            MessageBoxIcon.Warning);

        if (confirm != DialogResult.Yes) return;

        bool ok = ToolInstaller.UninstallAsync(installed, _toolStore);
        if (ok)
        {
            RefreshToolsGrid();
            MessageBox.Show($"{installed.DisplayName} uninstalled.", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
    }

    private void ScanAndAdoptLocalTools()
    {
        var discovered = AdoptExistingScanner.ScanAll();
        if (discovered.Count == 0)
        {
            MessageBox.Show("No existing local tools detected in C:\\tools, NVM, or PATH.", "Scan Results", MessageBoxButtons.OK, MessageBoxIcon.Information);
            return;
        }

        var sb = new System.Text.StringBuilder();
        sb.AppendLine($"Discovered {discovered.Count} local tools:");
        foreach (var d in discovered)
        {
            sb.AppendLine($"• {d.DisplayName} (v{d.DetectedVersion}) at {d.DirectoryPath}");
        }
        sb.AppendLine("\nDo you want to adopt all detected tools now?");

        var res = MessageBox.Show(sb.ToString(), "Adopt Detected Local Tools", MessageBoxButtons.YesNo, MessageBoxIcon.Question);
        if (res == DialogResult.Yes)
        {
            foreach (var d in discovered)
            {
                AdoptExistingScanner.Adopt(d, _toolStore);
            }
            RefreshToolsGrid();
            MessageBox.Show($"Adopted {discovered.Count} tools successfully! Registered in Dotnet.", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
    }

    private void AdoptCustomLocalFolder()
    {
        if (_selectedTool == null) return;
        using var fbd = new FolderBrowserDialog
        {
            Description = $"Select directory where {_selectedTool.DisplayName} is installed"
        };
        if (fbd.ShowDialog() == DialogResult.OK && Directory.Exists(fbd.SelectedPath))
        {
            var discovered = new DiscoveredTool
            {
                ToolId = _selectedTool.Id,
                DisplayName = _selectedTool.DisplayName,
                DetectedVersion = "Custom",
                DirectoryPath = fbd.SelectedPath,
                ExePath = fbd.SelectedPath
            };
            AdoptExistingScanner.Adopt(discovered, _toolStore);
            RefreshToolsGrid();
            MessageBox.Show($"Adopted {_selectedTool.DisplayName} from {fbd.SelectedPath}.", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }
    }

    // TAB 3: PROJECTS (V2)
    private TabPage CreateProjectsTab()
    {
        var tab = new TabPage("Projects") { BackColor = SystemColors.Control };

        var topBar = new Panel { Dock = DockStyle.Top, Height = 42, Padding = new Padding(4) };

        var btnAddProject = new Button
        {
            Text = "➕ Add Project",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(95, 26),
            Location = new Point(6, 6),
            Cursor = Cursors.Hand
        };
        btnAddProject.Click += (s, e) => ShowAddProjectDialog();

        var btnDetectFolder = new Button
        {
            Text = "🔍 Auto-Detect...",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(110, 26),
            Location = new Point(106, 6),
            Cursor = Cursors.Hand
        };
        btnDetectFolder.Click += (s, e) =>
        {
            using var fbd = new FolderBrowserDialog { Description = "Select Project Root Folder" };
            if (fbd.ShowDialog() == DialogResult.OK)
            {
                ShowAddProjectDialog(fbd.SelectedPath);
            }
        };

        var btnRunCmd = new Button
        {
            Text = "▶ Run Dev",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(85, 26),
            Location = new Point(221, 6),
            Cursor = Cursors.Hand
        };
        btnRunCmd.Click += (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null) ProjectManager.LaunchDevCommand(p);
        };

        var btnSetupDomain = new Button
        {
            Text = "🌐 Setup Domain",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(115, 26),
            Location = new Point(311, 6),
            Cursor = Cursors.Hand
        };
        btnSetupDomain.Click += async (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null)
            {
                var res = await NginxSiteGenerator.CreateOrUpdateSiteAsync(p);
                MessageBox.Show(res.Message, res.Success ? "Domain Setup Complete" : "Domain Setup Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
                RefreshProjectsGrid();
                RefreshHostsGrid();
            }
        };

        var btnOpenTerminal = new Button
        {
            Text = "💻 Terminal",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(85, 26),
            Location = new Point(431, 6),
            Cursor = Cursors.Hand
        };
        btnOpenTerminal.Click += (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null) ProjectManager.OpenTerminal(p.Path);
        };

        var btnOpenCode = new Button
        {
            Text = "📝 VS Code",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(85, 26),
            Location = new Point(521, 6),
            Cursor = Cursors.Hand
        };
        btnOpenCode.Click += (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null) ProjectManager.OpenInVSCode(p.Path);
        };

        var btnOpenBrowser = new Button
        {
            Text = "🌐 Browser",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(80, 26),
            Location = new Point(611, 6),
            Cursor = Cursors.Hand
        };
        btnOpenBrowser.Click += (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null) ProjectManager.OpenBrowser(p.NginxHost);
        };

        var btnEnableHttps = new Button
        {
            Text = "🔒 HTTPS (.test)",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(115, 26),
            Location = new Point(696, 6),
            Cursor = Cursors.Hand
        };
        btnEnableHttps.Click += async (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null)
            {
                var res = await NginxSiteGenerator.CreateOrUpdateSiteWithSslAsync(p);
                MessageBox.Show(res.Message, res.Success ? "HTTPS Configured" : "HTTPS Setup Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
                RefreshProjectsGrid();
                RefreshHostsGrid();
            }
        };

        var btnDelete = new Button
        {
            Text = "❌ Delete",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(75, 26),
            Location = new Point(816, 6),
            Cursor = Cursors.Hand
        };
        btnDelete.Click += async (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null)
            {
                var confirm = MessageBox.Show($"Remove project '{p.Name}'?\nThis will also remove any configured Nginx site & hosts entries.", "Confirm Delete", MessageBoxButtons.YesNo, MessageBoxIcon.Question);
                if (confirm == DialogResult.Yes)
                {
                    await NginxSiteGenerator.RemoveSiteAsync(p);
                    ProjectManager.DeleteProject(p.Id);
                    RefreshProjectsGrid();
                    RefreshHostsGrid();
                }
            }
        };

        var btnReloadProjects = new Button
        {
            Text = "🔄 Refresh",
            FlatStyle = FlatStyle.Standard,
            Size = new Size(75, 26),
            Location = new Point(896, 6),
            Cursor = Cursors.Hand
        };
        btnReloadProjects.Click += (s, e) => RefreshProjectsGrid();

        topBar.Controls.Add(btnAddProject);
        topBar.Controls.Add(btnDetectFolder);
        topBar.Controls.Add(btnRunCmd);
        topBar.Controls.Add(btnSetupDomain);
        topBar.Controls.Add(btnOpenTerminal);
        topBar.Controls.Add(btnOpenCode);
        topBar.Controls.Add(btnOpenBrowser);
        topBar.Controls.Add(btnEnableHttps);
        topBar.Controls.Add(btnDelete);
        topBar.Controls.Add(btnReloadProjects);

        _projectsGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            BackgroundColor = SystemColors.Window,
            ForeColor = SystemColors.ControlText,
            BorderStyle = BorderStyle.Fixed3D,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            MultiSelect = false,
            AllowUserToAddRows = false,
            RowHeadersVisible = false,
            ReadOnly = true
        };

        _projectsGrid.Columns.Add("Id", "ID");
        if (_projectsGrid.Columns["Id"] != null)
        {
            _projectsGrid.Columns["Id"]!.Visible = false;
        }
        _projectsGrid.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Project Name", Name = "Name", FillWeight = 20 });
        _projectsGrid.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Framework", Name = "Framework", FillWeight = 12 });
        _projectsGrid.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Local Domain", Name = "Host", FillWeight = 18 });
        _projectsGrid.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Dev Command", Name = "DevCmd", FillWeight = 20 });
        _projectsGrid.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Directory Path", Name = "Path", FillWeight = 30 });

        _projectsGrid.DoubleClick += (s, e) =>
        {
            var p = GetSelectedProject();
            if (p != null) ProjectManager.LaunchDevCommand(p);
        };

        RefreshProjectsGrid();

        tab.Controls.Add(_projectsGrid);
        tab.Controls.Add(topBar);

        return tab;
    }

    private ProjectInfo? GetSelectedProject()
    {
        if (_projectsGrid.SelectedRows.Count > 0)
        {
            string id = _projectsGrid.SelectedRows[0].Cells["Id"].Value?.ToString() ?? "";
            return ProjectManager.GetProjects().FirstOrDefault(p => p.Id == id);
        }
        MessageBox.Show("Please select a project from the list first.", "No Selection", MessageBoxButtons.OK, MessageBoxIcon.Information);
        return null;
    }

    private void RefreshProjectsGrid()
    {
        _projectsGrid.Rows.Clear();
        foreach (var p in ProjectManager.GetProjects())
        {
            _projectsGrid.Rows.Add(p.Id, p.Name, p.Framework, p.NginxHost, p.DevCommand, p.Path);
        }
    }

    private void ShowAddProjectDialog(string? initialPath = null)
    {
        using var dlg = new Form
        {
            Text = "Add / Auto-Detect Project - Dotnet",
            Size = new Size(500, 390),
            StartPosition = FormStartPosition.CenterParent,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = SystemColors.Control,
            ForeColor = SystemColors.ControlText,
            Font = new Font("Tahoma", 8.25F)
        };

        var lblPath = new Label { Text = "Project Directory:", Location = new Point(20, 20), AutoSize = true };
        var txtPath = new TextBox { Location = new Point(140, 18), Width = 230, BorderStyle = BorderStyle.Fixed3D, Text = initialPath ?? "" };
        var btnBrowse = new Button { Text = "Browse...", Location = new Point(380, 16), Width = 80, Height = 25, FlatStyle = FlatStyle.Standard };

        var btnDetect = new Button
        {
            Text = "🔍 Auto-Detect Framework & Config",
            Location = new Point(140, 50),
            Width = 320,
            Height = 26,
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };

        var lblName = new Label { Text = "Project Name:", Location = new Point(20, 90), AutoSize = true };
        var txtName = new TextBox { Location = new Point(140, 88), Width = 320, BorderStyle = BorderStyle.Fixed3D };

        var lblFw = new Label { Text = "Framework:", Location = new Point(20, 125), AutoSize = true };
        var cboFw = new ComboBox
        {
            Location = new Point(140, 123),
            Width = 150,
            DropDownStyle = ComboBoxStyle.DropDownList,
            FlatStyle = FlatStyle.Standard
        };
        cboFw.Items.AddRange(new object[] { "laravel", "nextjs", "vite", "node", "php", "static", "custom" });
        cboFw.SelectedItem = "custom";

        var lblPublic = new Label { Text = "Public Dir:", Location = new Point(310, 125), AutoSize = true };
        var txtPublic = new TextBox { Location = new Point(375, 123), Width = 85, BorderStyle = BorderStyle.Fixed3D, Text = "public" };

        var lblCmd = new Label { Text = "Dev Command:", Location = new Point(20, 160), AutoSize = true };
        var txtCmd = new TextBox { Text = "npm run dev", Location = new Point(140, 158), Width = 220, BorderStyle = BorderStyle.Fixed3D };

        var lblPort = new Label { Text = "Port:", Location = new Point(375, 160), AutoSize = true };
        var txtPort = new TextBox { Location = new Point(410, 158), Width = 50, BorderStyle = BorderStyle.Fixed3D };

        var lblHost = new Label { Text = "Local Domain:", Location = new Point(20, 195), AutoSize = true };
        var txtHost = new TextBox { Text = "myproject.test", Location = new Point(140, 193), Width = 320, BorderStyle = BorderStyle.Fixed3D };

        var chkSetupDomain = new CheckBox
        {
            Text = "🌐 Setup Nginx site & local hosts (.test) immediately",
            Location = new Point(140, 230),
            Size = new Size(330, 22),
            Checked = true,
            FlatStyle = FlatStyle.Standard
        };

        void RunDetection()
        {
            if (Directory.Exists(txtPath.Text.Trim()))
            {
                var res = FrameworkDetector.Detect(txtPath.Text.Trim());
                txtName.Text = res.SuggestedName;
                cboFw.SelectedItem = res.Framework;
                txtCmd.Text = res.DevCommand;
                txtPort.Text = res.SuggestedPort?.ToString() ?? "";
                txtPublic.Text = res.PublicDirectory;
                txtHost.Text = res.SuggestedHost;
            }
        }

        btnBrowse.Click += (s, e) =>
        {
            using var fbd = new FolderBrowserDialog();
            if (fbd.ShowDialog() == DialogResult.OK)
            {
                txtPath.Text = fbd.SelectedPath;
                RunDetection();
            }
        };

        btnDetect.Click += (s, e) => RunDetection();

        if (!string.IsNullOrEmpty(initialPath))
        {
            RunDetection();
        }

        var btnSave = new Button
        {
            Text = "Save Project",
            Location = new Point(140, 280),
            Width = 140,
            Height = 32,
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.OK
        };

        btnSave.Click += async (s, e) =>
        {
            if (string.IsNullOrWhiteSpace(txtName.Text) || string.IsNullOrWhiteSpace(txtPath.Text))
            {
                MessageBox.Show("Please fill Name and Directory Path.", "Validation Error", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                dlg.DialogResult = DialogResult.None;
                return;
            }

            int? devPort = int.TryParse(txtPort.Text.Trim(), out int parsedPort) ? parsedPort : null;

            var project = new ProjectInfo
            {
                Name = txtName.Text.Trim(),
                Path = txtPath.Text.Trim(),
                Framework = cboFw.SelectedItem?.ToString() ?? "custom",
                PublicDirectory = txtPublic.Text.Trim(),
                DevCommand = txtCmd.Text.Trim(),
                DevPort = devPort,
                NginxHost = txtHost.Text.Trim()
            };

            ProjectManager.AddOrUpdateProject(project);

            if (chkSetupDomain.Checked && !string.IsNullOrWhiteSpace(project.NginxHost))
            {
                var siteResult = await NginxSiteGenerator.CreateOrUpdateSiteAsync(project);
                if (!siteResult.Success)
                {
                    MessageBox.Show(siteResult.Message, "Nginx / Domain Warning", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                }
            }

            RefreshProjectsGrid();
            RefreshHostsGrid();
        };

        var btnCancel = new Button
        {
            Text = "Cancel",
            Location = new Point(290, 280),
            Width = 90,
            Height = 32,
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.Cancel
        };

        dlg.Controls.Add(lblPath); dlg.Controls.Add(txtPath); dlg.Controls.Add(btnBrowse);
        dlg.Controls.Add(btnDetect);
        dlg.Controls.Add(lblName); dlg.Controls.Add(txtName);
        dlg.Controls.Add(lblFw); dlg.Controls.Add(cboFw); dlg.Controls.Add(lblPublic); dlg.Controls.Add(txtPublic);
        dlg.Controls.Add(lblCmd); dlg.Controls.Add(txtCmd); dlg.Controls.Add(lblPort); dlg.Controls.Add(txtPort);
        dlg.Controls.Add(lblHost); dlg.Controls.Add(txtHost);
        dlg.Controls.Add(chkSetupDomain);
        dlg.Controls.Add(btnSave); dlg.Controls.Add(btnCancel);

        dlg.AcceptButton = btnSave;
        dlg.CancelButton = btnCancel;

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

    // TAB 4: PHP & NGINX CONFIG (V2)
    private TabPage CreateConfigTab()
    {
        var tab = new TabPage("PHP & Nginx Config") { BackColor = SystemColors.Control, AutoScroll = true };

        // 1. PHP Core Settings
        var pnlPhp = new GroupBox
        {
            Text = "PHP Core Configuration (php.ini)",
            Size = new Size(500, 270),
            Location = new Point(15, 15),
            ForeColor = SystemColors.ControlText
        };

        var lblPath = new Label { Text = $"Path: {PhpConfigManager.GetIniPath()}", Location = new Point(15, 22), AutoSize = true, ForeColor = SystemColors.ControlDarkDark };

        var l1 = new Label { Text = "memory_limit:", Location = new Point(15, 50), AutoSize = true };
        _phpMemoryLimit = new TextBox { Location = new Point(150, 48), Width = 140, BorderStyle = BorderStyle.Fixed3D };

        var l2 = new Label { Text = "upload_max_filesize:", Location = new Point(15, 80), AutoSize = true };
        _phpUploadMax = new TextBox { Location = new Point(150, 78), Width = 140, BorderStyle = BorderStyle.Fixed3D };

        var l3 = new Label { Text = "post_max_size:", Location = new Point(15, 110), AutoSize = true };
        _phpPostMax = new TextBox { Location = new Point(150, 108), Width = 140, BorderStyle = BorderStyle.Fixed3D };

        var l4 = new Label { Text = "max_execution_time:", Location = new Point(15, 140), AutoSize = true };
        _phpMaxExec = new TextBox { Location = new Point(150, 138), Width = 140, BorderStyle = BorderStyle.Fixed3D };

        var l5 = new Label { Text = "date.timezone:", Location = new Point(15, 170), AutoSize = true };
        _phpTimezone = new TextBox { Location = new Point(150, 168), Width = 140, BorderStyle = BorderStyle.Fixed3D };

        var btnSavePhp = new Button { Text = "Save php.ini", Location = new Point(15, 215), Width = 100, Height = 28, FlatStyle = FlatStyle.Standard, Cursor = Cursors.Hand };
        btnSavePhp.Click += (s, e) => SavePhpIniValues();

        var btnDevPreset = new Button { Text = "Dev Preset", Location = new Point(125, 215), Width = 90, Height = 28, FlatStyle = FlatStyle.Standard, Cursor = Cursors.Hand };
        btnDevPreset.Click += (s, e) =>
        {
            var res = PhpConfigManager.ApplyPreset("development");
            if (res.Success)
            {
                MessageBox.Show($"Development preset applied!\n\nDiff preview:\n{res.Diff}", "Preset Applied", MessageBoxButtons.OK, MessageBoxIcon.Information);
                LoadPhpIniValues();
            }
            else
            {
                MessageBox.Show(res.Message, "Error Applying Preset", MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
        };

        var btnProdPreset = new Button { Text = "Prod Preset", Location = new Point(225, 215), Width = 90, Height = 28, FlatStyle = FlatStyle.Standard, Cursor = Cursors.Hand };
        btnProdPreset.Click += (s, e) =>
        {
            var res = PhpConfigManager.ApplyPreset("production");
            if (res.Success)
            {
                MessageBox.Show($"Production preset applied!\n\nDiff preview:\n{res.Diff}", "Preset Applied", MessageBoxButtons.OK, MessageBoxIcon.Information);
                LoadPhpIniValues();
            }
            else
            {
                MessageBox.Show(res.Message, "Error Applying Preset", MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
        };

        var btnOpenPhpNotepad = new Button { Text = "📝 Notepad", Location = new Point(325, 215), Width = 90, Height = 28, FlatStyle = FlatStyle.Standard, Cursor = Cursors.Hand };
        btnOpenPhpNotepad.Click += (s, e) =>
        {
            try
            {
                string iniPath = PhpConfigManager.GetIniPath();
                if (File.Exists(iniPath)) System.Diagnostics.Process.Start("notepad.exe", iniPath);
            }
            catch (Exception ex) { MessageBox.Show(ex.Message); }
        };

        pnlPhp.Controls.Add(lblPath);
        pnlPhp.Controls.Add(l1); pnlPhp.Controls.Add(_phpMemoryLimit);
        pnlPhp.Controls.Add(l2); pnlPhp.Controls.Add(_phpUploadMax);
        pnlPhp.Controls.Add(l3); pnlPhp.Controls.Add(_phpPostMax);
        pnlPhp.Controls.Add(l4); pnlPhp.Controls.Add(_phpMaxExec);
        pnlPhp.Controls.Add(l5); pnlPhp.Controls.Add(_phpTimezone);
        pnlPhp.Controls.Add(btnSavePhp); pnlPhp.Controls.Add(btnDevPreset);
        pnlPhp.Controls.Add(btnProdPreset); pnlPhp.Controls.Add(btnOpenPhpNotepad);

        // 2. PHP FastCGI Pool Settings
        var pnlPhpPool = new GroupBox
        {
            Text = "PHP FastCGI Pool & Upstream",
            Size = new Size(500, 270),
            Location = new Point(530, 15),
            ForeColor = SystemColors.ControlText
        };

        var lblPoolDesc = new Label
        {
            Text = "Windows php-cgi is single-threaded. Spawning multiple pool workers\neliminates concurrent request blocking in local web development.",
            Location = new Point(15, 25),
            Size = new Size(465, 35)
        };

        var lblPoolSize = new Label { Text = "Pool Size (Workers):", Location = new Point(15, 75), AutoSize = true };
        _numPhpPoolSize = new NumericUpDown
        {
            Location = new Point(160, 73),
            Width = 70,
            Minimum = 1,
            Maximum = 8,
            Value = PhpPoolManager.PoolSize,
            BorderStyle = BorderStyle.Fixed3D
        };

        var lblBasePort = new Label { Text = "Base Port: 9000 (workers listen on ports 9000..900N)", Location = new Point(15, 110), AutoSize = true, ForeColor = SystemColors.ControlDarkDark };
        var lblUpstreamTarget = new Label { Text = "Nginx Upstream Target: 'php_pool' (conf/upstream_php.conf)", Location = new Point(15, 135), AutoSize = true, ForeColor = SystemColors.ControlDarkDark };

        var btnSavePool = new Button
        {
            Text = "Save Upstream Config",
            Location = new Point(15, 185),
            Size = new Size(160, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnSavePool.Click += (s, e) =>
        {
            int size = (int)_numPhpPoolSize.Value;
            PhpPoolManager.PoolSize = size;
            PhpPoolManager.SaveUpstreamConfig(size);
            MessageBox.Show($"Upstream block 'php_pool' generated with {size} worker servers.", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
        };

        pnlPhpPool.Controls.Add(lblPoolDesc);
        pnlPhpPool.Controls.Add(lblPoolSize);
        pnlPhpPool.Controls.Add(_numPhpPoolSize);
        pnlPhpPool.Controls.Add(lblBasePort);
        pnlPhpPool.Controls.Add(lblUpstreamTarget);
        pnlPhpPool.Controls.Add(btnSavePool);

        // 3. PHP Dynamic Extensions Manager
        var pnlExt = new GroupBox
        {
            Text = "PHP Dynamic Extensions (ext/*.dll)",
            Size = new Size(500, 310),
            Location = new Point(15, 300),
            ForeColor = SystemColors.ControlText
        };

        _lstPhpExtensions = new CheckedListBox
        {
            Location = new Point(15, 25),
            Size = new Size(470, 220),
            BorderStyle = BorderStyle.Fixed3D,
            CheckOnClick = true,
            MultiColumn = true,
            ColumnWidth = 145
        };

        var btnApplyExts = new Button
        {
            Text = "Apply Extensions",
            Location = new Point(15, 260),
            Size = new Size(130, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnApplyExts.Click += (s, e) => SavePhpExtensions();

        var btnCommonExts = new Button
        {
            Text = "Enable Common",
            Location = new Point(155, 260),
            Size = new Size(120, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnCommonExts.Click += (s, e) =>
        {
            var common = new HashSet<string>(StringComparer.OrdinalIgnoreCase) { "curl", "fileinfo", "gd", "intl", "mbstring", "openssl", "pdo_mysql", "zip" };
            for (int i = 0; i < _lstPhpExtensions.Items.Count; i++)
            {
                string name = _lstPhpExtensions.Items[i]?.ToString() ?? "";
                if (common.Contains(name))
                {
                    _lstPhpExtensions.SetItemChecked(i, true);
                }
            }
        };

        var btnRefreshExts = new Button
        {
            Text = "🔄 Refresh Exts",
            Location = new Point(285, 260),
            Size = new Size(110, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnRefreshExts.Click += (s, e) => LoadPhpExtensions();

        pnlExt.Controls.Add(_lstPhpExtensions);
        pnlExt.Controls.Add(btnApplyExts);
        pnlExt.Controls.Add(btnCommonExts);
        pnlExt.Controls.Add(btnRefreshExts);

        // 4. Nginx Server & Error Log
        var pnlNginx = new GroupBox
        {
            Text = "Nginx Server & Error Log",
            Size = new Size(500, 310),
            Location = new Point(530, 300),
            ForeColor = SystemColors.ControlText
        };

        var btnTestNginx = new Button
        {
            Text = "🧪 Validate (nginx -t)",
            Location = new Point(15, 25),
            Size = new Size(150, 26),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnTestNginx.Click += (s, e) =>
        {
            var res = NginxManager.ValidateConfig();
            MessageBox.Show(res.Output, res.Success ? "Config Test Passed" : "Config Test Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
        };

        var btnReloadNginx = new Button
        {
            Text = "⚡ Reload Nginx",
            Location = new Point(175, 25),
            Size = new Size(140, 26),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnReloadNginx.Click += (s, e) =>
        {
            var res = NginxManager.ReloadNginx();
            MessageBox.Show(res.Output, res.Success ? "Nginx Reload Success" : "Nginx Reload Failed", MessageBoxButtons.OK, res.Success ? MessageBoxIcon.Information : MessageBoxIcon.Error);
        };

        var btnOpenSitesFolder = new Button
        {
            Text = "📁 Sites Folder",
            Location = new Point(325, 25),
            Size = new Size(150, 26),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnOpenSitesFolder.Click += (s, e) =>
        {
            string sitesFolder = NginxSiteGenerator.GetSitesDirectory();
            System.Diagnostics.Process.Start("explorer.exe", sitesFolder);
        };

        var lblLogTitle = new Label { Text = "Error Log (Tail 30 lines):", Location = new Point(15, 65), AutoSize = true, ForeColor = SystemColors.ControlDarkDark };

        _txtNginxErrorLog = new RichTextBox
        {
            Location = new Point(15, 88),
            Size = new Size(470, 160),
            BackColor = SystemColors.Window,
            ForeColor = SystemColors.WindowText,
            BorderStyle = BorderStyle.Fixed3D,
            Font = new Font("Consolas", 8.25F),
            ReadOnly = true
        };

        var btnRefreshLog = new Button
        {
            Text = "🔄 Refresh Log",
            Location = new Point(15, 260),
            Size = new Size(120, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnRefreshLog.Click += (s, e) => RefreshNginxLog();

        var btnTrustRootCa = new Button
        {
            Text = "🔒 Trust Root CA",
            Location = new Point(145, 260),
            Size = new Size(140, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnTrustRootCa.Click += (s, e) =>
        {
            var op = LocalCertificateManager.TrustRootCertificate();
            MessageBox.Show(op.Message, op.Success ? "Root CA Trusted" : "Trust Error", MessageBoxButtons.OK, op.Success ? MessageBoxIcon.Information : MessageBoxIcon.Warning);
        };

        var btnOpenSslFolder = new Button
        {
            Text = "📁 SSL Folder",
            Location = new Point(295, 260),
            Size = new Size(130, 28),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };
        btnOpenSslFolder.Click += (s, e) =>
        {
            string sslDir = LocalCertificateManager.GetSslDirectory();
            if (!Directory.Exists(sslDir)) Directory.CreateDirectory(sslDir);
            System.Diagnostics.Process.Start("explorer.exe", sslDir);
        };

        pnlNginx.Controls.Add(btnTestNginx);
        pnlNginx.Controls.Add(btnReloadNginx);
        pnlNginx.Controls.Add(btnOpenSitesFolder);
        pnlNginx.Controls.Add(lblLogTitle);
        pnlNginx.Controls.Add(_txtNginxErrorLog);
        pnlNginx.Controls.Add(btnRefreshLog);
        pnlNginx.Controls.Add(btnTrustRootCa);
        pnlNginx.Controls.Add(btnOpenSslFolder);

        tab.Controls.Add(pnlPhp);
        tab.Controls.Add(pnlPhpPool);
        tab.Controls.Add(pnlExt);
        tab.Controls.Add(pnlNginx);

        LoadPhpIniValues();
        RefreshNginxLog();

        return tab;
    }

    private void LoadPhpIniValues()
    {
        _phpMemoryLimit.Text = PhpConfigManager.GetSettingValue("memory_limit");
        _phpUploadMax.Text = PhpConfigManager.GetSettingValue("upload_max_filesize");
        _phpPostMax.Text = PhpConfigManager.GetSettingValue("post_max_size");
        _phpMaxExec.Text = PhpConfigManager.GetSettingValue("max_execution_time");
        _phpTimezone.Text = PhpConfigManager.GetSettingValue("date.timezone");
        LoadPhpExtensions();
    }

    private void SavePhpIniValues()
    {
        PhpConfigManager.UpdateSetting("memory_limit", _phpMemoryLimit.Text.Trim());
        PhpConfigManager.UpdateSetting("upload_max_filesize", _phpUploadMax.Text.Trim());
        PhpConfigManager.UpdateSetting("post_max_size", _phpPostMax.Text.Trim());
        PhpConfigManager.UpdateSetting("max_execution_time", _phpMaxExec.Text.Trim());
        if (!string.IsNullOrWhiteSpace(_phpTimezone.Text))
        {
            PhpConfigManager.UpdateSetting("date.timezone", _phpTimezone.Text.Trim());
        }
        MessageBox.Show("php.ini settings saved with backup!", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
    }

    private void LoadPhpExtensions()
    {
        if (_lstPhpExtensions == null) return;
        _lstPhpExtensions.Items.Clear();

        var exts = PhpConfigManager.GetAvailableExtensions();
        foreach (var ext in exts)
        {
            _lstPhpExtensions.Items.Add(ext.Name, ext.IsEnabled);
        }
    }

    private void SavePhpExtensions()
    {
        if (_lstPhpExtensions == null) return;
        for (int i = 0; i < _lstPhpExtensions.Items.Count; i++)
        {
            string name = _lstPhpExtensions.Items[i]?.ToString() ?? "";
            bool isChecked = _lstPhpExtensions.GetItemChecked(i);
            PhpConfigManager.ToggleExtension(name, isChecked);
        }
        MessageBox.Show("PHP extensions configuration updated successfully!", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);
    }

    private void RefreshNginxLog()
    {
        if (_txtNginxErrorLog == null) return;
        var lines = NginxManager.TailErrorLog(30);
        _txtNginxErrorLog.Text = lines.Count > 0
            ? string.Join(Environment.NewLine, lines)
            : "[No error logs recorded or file is empty]";
        _txtNginxErrorLog.SelectionStart = _txtNginxErrorLog.Text.Length;
        _txtNginxErrorLog.ScrollToCaret();
    }

    // TAB 6: DIAGNOSTICS & SYSTEM AUDIT
    private TabPage CreateDiagnosticsTab()
    {
        var tab = new TabPage("Diagnostics & Port Monitor") { BackColor = SystemColors.Control, AutoScroll = true };

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

        btnRunDockerCheck.Click += async (s, e) =>
        {
            if (string.IsNullOrWhiteSpace(txtDockerPath.Text) || !File.Exists(txtDockerPath.Text))
            {
                MessageBox.Show("Select a valid docker-compose.yml file first.", "File Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            btnRunDockerCheck.Enabled = false;
            txtDockerResult.Text = "Inspecting compose file ports...";

            var report = await DockerPortChecker.CheckComposeFileAsync(txtDockerPath.Text);
            btnRunDockerCheck.Enabled = true;

            var sb = new System.Text.StringBuilder();
            sb.AppendLine("Docker Compose Pre-Flight Port Report:");
            sb.AppendLine("--------------------------------------------------");
            sb.AppendLine($"Parser Mode: {(report.IsDockerAvailable ? "Docker CLI Config" : "YAML Fallback")}");
            sb.AppendLine($"Status: {report.Message}");
            sb.AppendLine();

            if (report.Conflicts.Count == 0)
            {
                sb.AppendLine("No published host port bindings found in compose file.");
            }
            else
            {
                foreach (var c in report.Conflicts)
                {
                    if (c.IsSameProjectContainer)
                    {
                        sb.AppendLine($"ℹ️ Port {c.HostPort} -> Running container of this project (safe).");
                    }
                    else if (c.IsConflicting)
                    {
                        sb.AppendLine($"❌ CONFLICT: Port {c.HostPort} ({c.ServiceName}) -> {c.ConflictOwner}");
                        sb.AppendLine($"   👉 Suggested Override: {c.SuggestedOverridePort}");
                    }
                    else
                    {
                        sb.AppendLine($"✅ Port {c.HostPort} ({c.ServiceName}:{c.ContainerPort}/{c.Protocol}) -> Free.");
                    }
                }
            }

            txtDockerResult.Text = sb.ToString();
        };

        gbDocker.Controls.Add(lblDockerPath); gbDocker.Controls.Add(txtDockerPath); gbDocker.Controls.Add(btnBrowseDocker);
        gbDocker.Controls.Add(btnRunDockerCheck); gbDocker.Controls.Add(txtDockerResult);

        // Active TCP Ports Monitor
        var gbActivePorts = new GroupBox
        {
            Text = "Active TCP Listeners Monitor (Real-time Port Snapshot)",
            Size = new Size(1015, 230),
            Location = new Point(15, 305),
            ForeColor = SystemColors.ControlText
        };

        var btnRefreshPorts = new Button
        {
            Text = "🔄 Refresh Active Ports",
            Location = new Point(15, 22),
            Size = new Size(160, 26),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };

        _gridActivePorts = new DataGridView
        {
            Location = new Point(15, 54),
            Size = new Size(985, 160),
            BackgroundColor = SystemColors.Window,
            BorderStyle = BorderStyle.Fixed3D,
            ReadOnly = true,
            AllowUserToAddRows = false,
            AllowUserToDeleteRows = false,
            AllowUserToResizeRows = false,
            RowHeadersVisible = false,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            MultiSelect = false,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill
        };

        _gridActivePorts.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Port", Width = 70, FillWeight = 10 });
        _gridActivePorts.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "PID", Width = 70, FillWeight = 10 });
        _gridActivePorts.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Process Name", Width = 150, FillWeight = 25 });
        _gridActivePorts.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Managed Service", Width = 180, FillWeight = 30 });
        _gridActivePorts.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Type", Width = 100, FillWeight = 15 });

        void PopulateActivePorts()
        {
            _gridActivePorts.Rows.Clear();
            var listeners = PortConflictDetector.GetAllActiveTcpListeners(_services);
            foreach (var l in listeners)
            {
                int rowIndex = _gridActivePorts.Rows.Add(
                    l.Port,
                    l.ProcessId,
                    l.ProcessName,
                    string.IsNullOrEmpty(l.ServiceName) ? "-" : l.ServiceName,
                    l.IsDotnetManaged ? "Managed" : "External"
                );
                if (l.IsDotnetManaged)
                {
                    _gridActivePorts.Rows[rowIndex].DefaultCellStyle.ForeColor = Color.DarkGreen;
                }
            }
        }

        btnRefreshPorts.Click += (s, e) => PopulateActivePorts();
        gbActivePorts.Controls.Add(btnRefreshPorts);
        gbActivePorts.Controls.Add(_gridActivePorts);

        // PATH Shadow Analyzer
        var gbShadow = new GroupBox
        {
            Text = "PATH Shadow Analyzer (User PATH vs System PATH Precedence Check)",
            Size = new Size(1015, 230),
            Location = new Point(15, 545),
            ForeColor = SystemColors.ControlText
        };

        var btnAnalyzeShadow = new Button
        {
            Text = "🔍 Analyze PATH Shadowing",
            Location = new Point(15, 22),
            Size = new Size(180, 26),
            FlatStyle = FlatStyle.Standard,
            Cursor = Cursors.Hand
        };

        _gridShadow = new DataGridView
        {
            Location = new Point(15, 54),
            Size = new Size(985, 160),
            BackgroundColor = SystemColors.Window,
            BorderStyle = BorderStyle.Fixed3D,
            ReadOnly = true,
            AllowUserToAddRows = false,
            AllowUserToDeleteRows = false,
            AllowUserToResizeRows = false,
            RowHeadersVisible = false,
            SelectionMode = DataGridViewSelectionMode.FullRowSelect,
            MultiSelect = false,
            AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill
        };

        _gridShadow.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Binary", Width = 90, FillWeight = 12 });
        _gridShadow.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Status", Width = 90, FillWeight = 12 });
        _gridShadow.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Managed User Path", Width = 220, FillWeight = 28 });
        _gridShadow.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "System Path Location", Width = 220, FillWeight = 28 });
        _gridShadow.Columns.Add(new DataGridViewTextBoxColumn { HeaderText = "Recommendation", Width = 200, FillWeight = 20 });

        void RunShadowAnalysis()
        {
            _gridShadow.Rows.Clear();
            var results = PathShadowAnalyzer.Analyze();
            foreach (var r in results)
            {
                int rowIndex = _gridShadow.Rows.Add(
                    r.BinaryName,
                    r.IsShadowed ? "⚠️ SHADOWED" : "✅ OK",
                    r.UserPathLocation,
                    r.SystemPathLocation,
                    r.Recommendation
                );
                if (r.IsShadowed)
                {
                    _gridShadow.Rows[rowIndex].DefaultCellStyle.ForeColor = Color.DarkRed;
                    _gridShadow.Rows[rowIndex].DefaultCellStyle.BackColor = Color.FromArgb(255, 240, 240);
                }
                else
                {
                    _gridShadow.Rows[rowIndex].DefaultCellStyle.ForeColor = Color.DarkGreen;
                }
            }
        }

        btnAnalyzeShadow.Click += (s, e) => RunShadowAnalysis();
        gbShadow.Controls.Add(btnAnalyzeShadow);
        gbShadow.Controls.Add(_gridShadow);

        tab.Controls.Add(gbPort);
        tab.Controls.Add(gbDocker);
        tab.Controls.Add(gbActivePorts);
        tab.Controls.Add(gbShadow);

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
        UpdateTrayMenu();

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

    private void UpdateTrayMenu()
    {
        if (_trayMenu == null) return;
        _trayMenu.Items.Clear();

        var mnuShow = new ToolStripMenuItem("Tampilkan Dashboard", null, (s, e) => RestoreFromTray());
        mnuShow.Font = new Font(mnuShow.Font, FontStyle.Bold);

        // Profiles Submenu
        var activeProf = ProfileStore.GetActiveProfile();
        var mnuProfile = new ToolStripMenuItem($"Profil: {activeProf.Name}");
        foreach (var prof in ProfileStore.Profiles)
        {
            var pItem = new ToolStripMenuItem(prof.Name, null, (s, e) =>
            {
                ProfileStore.ActiveProfileId = prof.Id;
                if (_cboProfile != null)
                {
                    _cboProfile.SelectedItem = prof.Name;
                }
                AppLogger.Log($"Tray: Switched active profile to: {prof.Name}");
                RefreshServicesStatus();
            })
            {
                Checked = prof.Id.Equals(activeProf.Id, StringComparison.OrdinalIgnoreCase)
            };
            mnuProfile.DropDownItems.Add(pItem);
        }

        // Service Statuses Submenu
        int runningCount = _services.Count(s => s.Status == ServiceStatus.Running);
        var mnuStatus = new ToolStripMenuItem($"Status Layanan ({runningCount} aktif)");
        foreach (var svc in _services)
        {
            string icon = svc.Status == ServiceStatus.Running ? "●" : "○";
            var item = new ToolStripMenuItem($"{icon} {svc.Name} ({svc.Status})");
            item.Enabled = false;
            mnuStatus.DropDownItems.Add(item);
        }

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

        var mnuExit = new ToolStripMenuItem("❌ Keluar / Exit", null, (s, e) => HandleAppExit());

        _trayMenu.Items.Add(mnuShow);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuProfile);
        _trayMenu.Items.Add(mnuStatus);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuStartAll);
        _trayMenu.Items.Add(mnuStopAll);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuAutoStart);
        _trayMenu.Items.Add(mnuShortcut);
        _trayMenu.Items.Add(new ToolStripSeparator());
        _trayMenu.Items.Add(mnuExit);
    }

    private void HandleAppExit()
    {
        int runningCount = _services.Count(s => s.Status == ServiceStatus.Running);
        if (runningCount > 0)
        {
            var pref = AppSettingsManager.Settings.ExitPreference;
            if (pref == ExitActionPreference.Prompt)
            {
                using var dlg = new ExitPolicyDialog(runningCount);
                var result = dlg.ShowDialog(this);
                if (result == DialogResult.Cancel)
                {
                    return;
                }

                if (dlg.StopServicesOnExit)
                {
                    StopAllGroupServicesSync();
                }
            }
            else if (pref == ExitActionPreference.StopServices)
            {
                StopAllGroupServicesSync();
            }
        }

        _allowClose = true;
        _notifyIcon.Visible = false;
        Application.Exit();
    }

    private void StopAllGroupServicesSync()
    {
        AppLogger.Log("Exit Policy: Stopping all active services...");
        foreach (var svc in _services)
        {
            try
            {
                if (svc.Status == ServiceStatus.Running)
                {
                    if (svc.Type == DevServiceType.WindowsService)
                    {
                        WindowsServiceManager.StopService(svc.WindowsServiceName);
                    }
                    else
                    {
                        ProcessServiceManager.StopProcess(svc);
                    }
                }
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Exit Policy: Error stopping {svc.Name}: {ex.Message}");
            }
        }
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
            if (!_allowClose)
            {
                int runningCount = _services.Count(s => s.Status == ServiceStatus.Running);
                if (runningCount > 0 && AppSettingsManager.Settings.ExitPreference == ExitActionPreference.StopServices)
                {
                    StopAllGroupServicesSync();
                }
            }
            _notifyIcon.Visible = false;
            base.OnFormClosing(e);
        }
    }
}
