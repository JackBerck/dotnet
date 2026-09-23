using Microsoft.Win32;

namespace dotnet.Services;

public static class VcRedistChecker
{
    private const string VcRuntimeKey = @"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64";
    public const string OfficialDownloadUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe";

    public static bool IsVcRedistInstalled()
    {
        try
        {
            using var baseKey = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(VcRuntimeKey);
            if (subKey == null) return false;

            var installed = subKey.GetValue("Installed");
            if (installed is int intVal && intVal == 1) return true;

            var major = subKey.GetValue("Major");
            if (major is int majorVal && majorVal >= 14) return true;

            return false;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error checking VC++ Redistributable: {ex.Message}");
            return false;
        }
    }

    public static string? GetInstalledVersion()
    {
        try
        {
            using var baseKey = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
            using var subKey = baseKey.OpenSubKey(VcRuntimeKey);
            if (subKey == null) return null;

            return subKey.GetValue("Version")?.ToString();
        }
        catch
        {
            return null;
        }
    }
}
