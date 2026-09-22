using System.Drawing;
using System.Windows.Forms;
using dotnet.Models;
using dotnet.Services;

namespace dotnet;

public class ExitPolicyDialog : Form
{
    private readonly CheckBox chkRemember;
    public bool StopServicesOnExit { get; private set; } = true;

    public ExitPolicyDialog(int runningCount)
    {
        Text = "Konfirmasi Keluar - Dotnet";
        Size = new Size(420, 200);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        StartPosition = FormStartPosition.CenterScreen;
        Font = new Font("Tahoma", 8.25f, FontStyle.Regular);
        BackColor = SystemColors.Control;
        ForeColor = SystemColors.ControlText;

        var lblMessage = new Label
        {
            Text = $"Terdapat {runningCount} layanan yang sedang berjalan.\n\nApakah Anda ingin menghentikan semua layanan latar belakang sebelum keluar?",
            Location = new Point(20, 20),
            Size = new Size(365, 48),
            TextAlign = ContentAlignment.TopLeft
        };
        Controls.Add(lblMessage);

        chkRemember = new CheckBox
        {
            Text = "Ingat pilihan saya (jangan tanya lagi)",
            Location = new Point(20, 80),
            Size = new Size(360, 20),
            FlatStyle = FlatStyle.Standard
        };
        Controls.Add(chkRemember);

        var btnStopAndExit = new Button
        {
            Text = "Hentikan & Keluar",
            Location = new Point(20, 118),
            Size = new Size(125, 26),
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.Yes
        };
        btnStopAndExit.Click += (s, e) => HandleChoice(true);
        Controls.Add(btnStopAndExit);

        var btnKeepRunning = new Button
        {
            Text = "Biarkan Berjalan",
            Location = new Point(155, 118),
            Size = new Size(125, 26),
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.No
        };
        btnKeepRunning.Click += (s, e) => HandleChoice(false);
        Controls.Add(btnKeepRunning);

        var btnCancel = new Button
        {
            Text = "Batal",
            Location = new Point(290, 118),
            Size = new Size(95, 26),
            FlatStyle = FlatStyle.Standard,
            DialogResult = DialogResult.Cancel
        };
        Controls.Add(btnCancel);

        AcceptButton = btnStopAndExit;
        CancelButton = btnCancel;
    }

    private void HandleChoice(bool stopServices)
    {
        StopServicesOnExit = stopServices;
        if (chkRemember.Checked)
        {
            AppSettingsManager.SetExitPreference(
                stopServices ? ExitActionPreference.StopServices : ExitActionPreference.KeepRunning
            );
        }
    }
}
