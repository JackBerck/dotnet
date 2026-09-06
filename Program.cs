using System.Runtime.InteropServices;

namespace dotnet;

static class Program
{
    private const string MutexName = "Global\\StandaloneDevManager_SingleInstance_Mutex";

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern int RegisterWindowMessage(string lpString);

    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern bool PostMessage(IntPtr hWnd, int Msg, IntPtr wParam, IntPtr lParam);

    public static readonly int HWND_BROADCAST = 0xffff;
    public static readonly int WM_RESTORE_APP = RegisterWindowMessage("WM_RESTORE_STANDALONE_DEV_MANAGER");

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main(string[] args)
    {
        using var mutex = new Mutex(true, MutexName, out bool createdNew);

        if (!createdNew)
        {
            // Another instance is ALREADY running! Broadcast restore message to existing window and exit.
            PostMessage((IntPtr)HWND_BROADCAST, WM_RESTORE_APP, IntPtr.Zero, IntPtr.Zero);
            return;
        }

        ApplicationConfiguration.Initialize();

        // Ensure Start Menu shortcut exists so app is searchable in Windows
        Services.StartMenuShortcutManager.CreateStartMenuShortcut();

        bool startMinimized = args.Contains("--autostart") || args.Contains("--minimized");
        Application.Run(new Form1(startMinimized));
    }
}