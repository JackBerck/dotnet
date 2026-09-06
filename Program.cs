namespace dotnet;

static class Program
{
    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main(string[] args)
    {
        ApplicationConfiguration.Initialize();

        // Ensure Start Menu shortcut exists so app is searchable in Windows
        Services.StartMenuShortcutManager.CreateStartMenuShortcut();

        bool startMinimized = args.Contains("--autostart") || args.Contains("--minimized");
        Application.Run(new Form1(startMinimized));
    }    
}