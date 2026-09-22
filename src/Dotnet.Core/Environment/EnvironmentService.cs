using System.Collections;
using System.Runtime.InteropServices;
using dotnet.Persistence;
using dotnet.Services;
using Microsoft.Win32;

namespace dotnet.Env;

public class EnvironmentService
{
    private const int HWND_BROADCAST = 0xffff;
    private const uint WM_SETTINGCHANGE = 0x001A;
    private const uint SMTO_ABORTIFHUNG = 0x0002;

    [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    private static extern IntPtr SendMessageTimeout(
        IntPtr hWnd,
        uint Msg,
        UIntPtr wParam,
        string lParam,
        uint fuFlags,
        uint uTimeout,
        out UIntPtr lpdwResult);

    public static string GetUserPath()
    {
        if (!OperatingSystem.IsWindows()) return string.Empty;
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey("Environment", false);
            var val = key?.GetValue("Path", "", RegistryValueOptions.DoNotExpandEnvironmentNames);
            return val?.ToString() ?? string.Empty;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reading User PATH from registry: {ex.Message}");
            return string.Empty;
        }
    }

    public static bool BackupPath()
    {
        try
        {
            string backupDir = AppPaths.GetPath("backups");
            Directory.CreateDirectory(backupDir);
            string backupFile = Path.Combine(backupDir, $"path_{DateTime.Now:yyyyMMdd_HHmmss}.bak");
            string currentPath = GetUserPath();
            File.WriteAllText(backupFile, currentPath);
            AppLogger.Log($"User PATH backup created at: {backupFile}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error creating PATH backup: {ex.Message}");
            return false;
        }
    }

    public static bool SetUserPath(string newPath)
    {
        if (!OperatingSystem.IsWindows()) return false;
        try
        {
            BackupPath();
            using (var key = Registry.CurrentUser.OpenSubKey("Environment", true))
            {
                if (key == null) return false;
                key.SetValue("Path", newPath, RegistryValueKind.ExpandString);
            }

            BroadcastEnvironmentChange();
            AppLogger.Log("User PATH updated successfully.");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error writing User PATH: {ex.Message}");
            return false;
        }
    }

    public static bool AddToUserPath(string directory, bool prepend = true)
    {
        string current = GetUserPath();
        var entries = PathEditor.Parse(current);

        if (PathEditor.ContainsEntry(entries, directory))
        {
            return true; // already present
        }

        var updated = prepend ? PathEditor.Prepend(entries, directory) : PathEditor.Append(entries, directory);
        return SetUserPath(PathEditor.Join(updated));
    }

    public static bool RemoveFromUserPath(string directory)
    {
        string current = GetUserPath();
        var entries = PathEditor.Parse(current);

        if (!PathEditor.ContainsEntry(entries, directory))
        {
            return true;
        }

        var updated = PathEditor.Remove(entries, directory);
        return SetUserPath(PathEditor.Join(updated));
    }

    public static void BroadcastEnvironmentChange()
    {
        if (!OperatingSystem.IsWindows()) return;
        try
        {
            SendMessageTimeout(
                new IntPtr(HWND_BROADCAST),
                WM_SETTINGCHANGE,
                UIntPtr.Zero,
                "Environment",
                SMTO_ABORTIFHUNG,
                5000,
                out _);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error broadcasting WM_SETTINGCHANGE: {ex.Message}");
        }
    }

    public static Dictionary<string, string> BuildProcessEnvironment()
    {
        var env = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

        // 1. Inherit current process env
        foreach (DictionaryEntry de in System.Environment.GetEnvironmentVariables())
        {
            if (de.Key is string k && de.Value is string v)
            {
                env[k] = v;
            }
        }

        // 2. Fetch fresh System + User PATH from registry
        if (OperatingSystem.IsWindows())
        {
            try
            {
                string systemPath = "";
                using (var sysKey = Registry.LocalMachine.OpenSubKey(@"SYSTEM\CurrentControlSet\Control\Session Manager\Environment", false))
                {
                    systemPath = sysKey?.GetValue("Path", "", RegistryValueOptions.None)?.ToString() ?? "";
                }

                string userPath = "";
                using (var usrKey = Registry.CurrentUser.OpenSubKey("Environment", false))
                {
                    userPath = usrKey?.GetValue("Path", "", RegistryValueOptions.None)?.ToString() ?? "";
                }

                string combinedPath = $"{userPath};{systemPath}".Trim(';');
                env["Path"] = combinedPath;
                env["PATH"] = combinedPath;
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Error building fresh environment block: {ex.Message}");
            }
        }

        return env;
    }
}
