using System.Diagnostics;

namespace dotnet.Services;

public class StartMenuShortcutManager
{
    public static bool CreateStartMenuShortcut()
    {
        try
        {
            string startMenuFolder = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                @"Microsoft\Windows\Start Menu\Programs"
            );

            Directory.CreateDirectory(startMenuFolder);
            string shortcutPath = Path.Combine(startMenuFolder, "Dotnet.lnk");

            string exePath = Process.GetCurrentProcess().MainModule?.FileName 
                ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dotnet.exe");
            
            string workDir = AppDomain.CurrentDomain.BaseDirectory;
            string iconPath = Path.Combine(workDir, "app.ico");
            if (!File.Exists(iconPath)) iconPath = exePath;

            // Use WScript.Shell COM object dynamically to create shortcut .lnk
            Type? shellType = Type.GetTypeFromProgID("WScript.Shell");
            if (shellType != null)
            {
                dynamic shell = Activator.CreateInstance(shellType)!;
                dynamic shortcut = shell.CreateShortcut(shortcutPath);
                shortcut.TargetPath = exePath;
                shortcut.WorkingDirectory = workDir;
                shortcut.Description = "Native Windows Standalone Local Development Manager";
                shortcut.IconLocation = $"{iconPath},0";
                shortcut.Save();

                AppLogger.Log($"Start Menu Shortcut created at: {shortcutPath}");
                return true;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error creating Start Menu shortcut: {ex.Message}");
        }

        return false;
    }
}
