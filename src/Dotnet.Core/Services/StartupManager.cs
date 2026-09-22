using Microsoft.Win32;
using System.Diagnostics;

namespace dotnet.Services;

public class StartupManager
{
    private const string RunKeyPath = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    private const string AppName = "Dotnet";

    public static bool IsRunOnStartupEnabled()
    {
        // Primary: Check Windows Task Scheduler (supports elevated startup F-10)
        if (TaskSchedulerManager.IsTaskScheduled())
        {
            return true;
        }

        // Secondary / Fallback: Check HKCU Run key
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
        string exePath = Process.GetCurrentProcess().MainModule?.FileName 
            ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dotnet.exe");

        if (enable)
        {
            // Try Task Scheduler first for elevated execution without UAC block at logon
            bool taskSuccess = TaskSchedulerManager.CreateLogonTask(exePath);
            
            // Clean up registry entry if task scheduler succeeded, or use as fallback
            if (taskSuccess)
            {
                CleanRegistryRunKey();
                return true;
            }

            // Fallback to registry if schtasks failed
            return SetRegistryRunKey(exePath);
        }
        else
        {
            bool taskDeleted = TaskSchedulerManager.DeleteLogonTask();
            bool regDeleted = CleanRegistryRunKey();
            return taskDeleted || regDeleted;
        }
    }

    private static bool SetRegistryRunKey(string exePath)
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true);
            if (key == null) return false;

            string value = $"\"{exePath}\" --autostart";
            key.SetValue(AppName, value);
            AppLogger.Log($"Enabled Windows Startup via Registry: {value}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error updating startup registry key: {ex.Message}");
            return false;
        }
    }

    private static bool CleanRegistryRunKey()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true);
            if (key != null && key.GetValue(AppName) != null)
            {
                key.DeleteValue(AppName, false);
                AppLogger.Log("Cleaned legacy Windows Startup registry key.");
                return true;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error removing startup registry key: {ex.Message}");
        }
        return false;
    }
}
