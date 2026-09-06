using Microsoft.Win32;
using System.Diagnostics;

namespace dotnet.Services;

public class StartupManager
{
    private const string RunKeyPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string AppName = "StandaloneDevManager";

    public static bool IsRunOnStartupEnabled()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, false);
            return key?.GetValue(AppName) != null;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error checking startup registry key: {ex.Message}");
            return false;
        }
    }

    public static bool SetRunOnStartup(bool enable)
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true);
            if (key == null) return false;

            if (enable)
            {
                string exePath = Process.GetCurrentProcess().MainModule?.FileName 
                    ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dotnet.exe");
                
                string value = $"\"{exePath}\" --autostart";
                key.SetValue(AppName, value);
                AppLogger.Log($"Enabled Windows Startup: {value}");
            }
            else
            {
                key.DeleteValue(AppName, false);
                AppLogger.Log("Disabled Windows Startup.");
            }

            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error updating startup registry key: {ex.Message}");
            return false;
        }
    }
}
