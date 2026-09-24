using System.Diagnostics;
using System.Reflection;
using System.Security.Principal;

namespace dotnet;

public class AboutDialog : Form
{
    public AboutDialog()
    {
        InitializeCustomUi();
    }

    private void InitializeCustomUi()
    {
        Text = "About Dotnet DevManager";
        Size = new Size(480, 340);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        ShowInTaskbar = false;
        StartPosition = FormStartPosition.CenterParent;
        BackColor = SystemColors.Control;
        Font = new Font("Tahoma", 8.25F, FontStyle.Regular, GraphicsUnit.Point);

        // Header Banner Panel
        var pnlHeader = new Panel
        {
            Dock = DockStyle.Top,
            Height = 65,
            BackColor = SystemColors.Window,
            BorderStyle = BorderStyle.FixedSingle,
            Padding = new Padding(15, 12, 15, 10)
        };

        var lblTitle = new Label
        {
            Text = "Dotnet Standalone DevManager",
            Font = new Font("Tahoma", 11F, FontStyle.Bold),
            ForeColor = SystemColors.WindowText,
            AutoSize = true,
            Location = new Point(15, 12)
        };

        var lblSub = new Label
        {
            Text = "Native Windows developer environment manager (Laragon alternative)",
            Font = new Font("Tahoma", 8.25F, FontStyle.Regular),
            ForeColor = SystemColors.ControlDarkDark,
            AutoSize = true,
            Location = new Point(16, 36)
        };

        pnlHeader.Controls.Add(lblTitle);
        pnlHeader.Controls.Add(lblSub);

        // Details Panel
        var pnlDetails = new Panel
        {
            Dock = DockStyle.Fill,
            Padding = new Padding(15, 15, 15, 10)
        };

        bool isAdmin = new WindowsPrincipal(WindowsIdentity.GetCurrent()).IsInRole(WindowsBuiltInRole.Administrator);
        string elevationText = isAdmin ? "Administrator (Elevated)" : "Standard User (Non-Elevated)";
        string version = Assembly.GetExecutingAssembly().GetName().Version?.ToString(3) ?? "1.0.0";

        var lblVersion = new Label
        {
            Text = $"Version: {version} (win-x64, .NET 10.0)",
            Location = new Point(15, 15),
            AutoSize = true,
            Font = new Font("Tahoma", 8.25F, FontStyle.Bold)
        };

        var lblLicense = new Label
        {
            Text = "License: MIT License — Open Source & Free Software",
            Location = new Point(15, 40),
            AutoSize = true
        };

        var lblAuthor = new Label
        {
            Text = "Authors: JackBerck and contributors",
            Location = new Point(15, 65),
            AutoSize = true
        };

        var lblElevation = new Label
        {
            Text = $"Process Security: {elevationText}",
            Location = new Point(15, 90),
            AutoSize = true,
            ForeColor = isAdmin ? Color.DarkRed : Color.DarkGreen
        };

        var lblOs = new Label
        {
            Text = $"Host OS: {Environment.OSVersion.VersionString} ({(Environment.Is64BitOperatingSystem ? "64-bit" : "32-bit")})",
            Location = new Point(15, 115),
            AutoSize = true,
            ForeColor = SystemColors.ControlDarkDark
        };

        var lnkRepo = new LinkLabel
        {
            Text = "GitHub Repository: https://github.com/JackBerck/dotnet",
            Location = new Point(15, 145),
            AutoSize = true
        };
        lnkRepo.LinkClicked += (s, e) =>
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = "https://github.com/JackBerck/dotnet",
                UseShellExecute = true
            });
        };

        pnlDetails.Controls.Add(lblVersion);
        pnlDetails.Controls.Add(lblLicense);
        pnlDetails.Controls.Add(lblAuthor);
        pnlDetails.Controls.Add(lblElevation);
        pnlDetails.Controls.Add(lblOs);
        pnlDetails.Controls.Add(lnkRepo);

        // Bottom Button Panel
        var pnlBottom = new Panel
        {
            Dock = DockStyle.Bottom,
            Height = 45,
            BackColor = SystemColors.Control,
            Padding = new Padding(15, 8, 15, 8)
        };

        var btnOk = new Button
        {
            Text = "OK",
            Size = new Size(80, 26),
            Location = new Point(365, 8),
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.OK,
            Cursor = Cursors.Hand
        };
        pnlBottom.Controls.Add(btnOk);

        Controls.Add(pnlDetails);
        Controls.Add(pnlBottom);
        Controls.Add(pnlHeader);

        AcceptButton = btnOk;
    }
}
