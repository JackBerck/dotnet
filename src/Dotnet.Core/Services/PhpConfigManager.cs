using dotnet.Config;
using dotnet.Persistence;

namespace dotnet.Services;

public class PhpSettingInfo
{
    public string Key { get; set; } = string.Empty;
    public string Value { get; set; } = string.Empty;
}

public class PhpConfigManager
{
    private static string _iniPath = @"C:\tools\php85\php.ini";

    public static string GetIniPath() => _iniPath;

    public static void SetIniPath(string path)
    {
        if (!string.IsNullOrWhiteSpace(path))
        {
            _iniPath = path;
        }
    }

    public static bool BackupIniFile(string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? _iniPath : iniPath;
        if (!File.Exists(iniPath)) return false;

        try
        {
            string backupDir = AppPaths.GetPath("backups");
            Directory.CreateDirectory(backupDir);
            string backupFile = Path.Combine(backupDir, $"php_ini_{DateTime.Now:yyyyMMdd_HHmmss}.bak");
            File.Copy(iniPath, backupFile, true);
            AppLogger.Log($"php.ini backup created at: {backupFile}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error backing up php.ini: {ex.Message}");
            return false;
        }
    }

    public static string? GetSetting(string key, string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? _iniPath : iniPath;
        if (!File.Exists(iniPath)) return null;

        try
        {
            var doc = IniDocument.Load(iniPath);
            return doc.Get(key);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reading php.ini setting {key}: {ex.Message}");
            return null;
        }
    }

    public static string GetSettingValue(string key, string iniPath = "")
    {
        return GetSetting(key, iniPath) ?? string.Empty;
    }

    public static bool UpdateSetting(string key, string value, string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? _iniPath : iniPath;
        if (!File.Exists(iniPath))
        {
            AppLogger.Log($"php.ini file not found at {iniPath}");
            return false;
        }

        BackupIniFile(iniPath);

        try
        {
            var doc = IniDocument.Load(iniPath);
            doc.Set(key, value);
            doc.Save(iniPath);
            AppLogger.Log($"Updated php.ini: {key} = {value}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error updating php.ini setting {key}: {ex.Message}");
            return false;
        }
    }
}
