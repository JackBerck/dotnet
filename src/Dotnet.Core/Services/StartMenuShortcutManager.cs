using System.Diagnostics;
using System.Runtime.InteropServices;

namespace dotnet.Services;

public class StartMenuShortcutManager
{
    public static bool CreateStartMenuShortcut()
    {
        object? shell = null;
        object? shortcut = null;

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

            // Use WScript.Shell COM object safely with proper RCW release
            Type? shellType = Type.GetTypeFromProgID("WScript.Shell");
            if (shellType != null)
            {
                shell = Activator.CreateInstance(shellType);
                if (shell != null)
                {
                    dynamic dynamicShell = shell;
                    shortcut = dynamicShell.CreateShortcut(shortcutPath);
                    if (shortcut != null)
                    {
                        dynamic dynamicShortcut = shortcut;
                        dynamicShortcut.TargetPath = exePath;
                        dynamicShortcut.WorkingDirectory = workDir;
                        dynamicShortcut.Description = "Native Windows Standalone Local Development Manager";
                        dynamicShortcut.IconLocation = $"{iconPath},0";
                        dynamicShortcut.Save();

                        AppLogger.Log($"Start Menu Shortcut created safely at: {shortcutPath}");
                        return true;
                    }
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error creating Start Menu shortcut: {ex.Message}");
        }
        finally
        {
            if (shortcut != null && Marshal.IsComObject(shortcut))
            {
                Marshal.FinalReleaseComObject(shortcut);
            }
            if (shell != null && Marshal.IsComObject(shell))
            {
                Marshal.FinalReleaseComObject(shell);
            }
        }

        return false;
    }
}
