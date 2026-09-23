using dotnet.Config;
using dotnet.Persistence;

namespace dotnet.Services;

public class PhpExtensionItem
{
    public string Name { get; set; } = string.Empty;
    public string FileName { get; set; } = string.Empty;
    public bool IsEnabled { get; set; }
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

    public static List<PhpExtensionItem> GetAvailableExtensions(string? phpDir = null)
    {
        var result = new List<PhpExtensionItem>();
        string targetDir = phpDir ?? Path.GetDirectoryName(_iniPath) ?? @"C:\tools\php85";
        string extDir = Path.Combine(targetDir, "ext");

        string currentIni = File.Exists(_iniPath) ? File.ReadAllText(_iniPath) : "";
        var doc = IniDocument.Parse(currentIni);
        var activeExts = doc.ListExtensions().Where(e => e.IsActive).Select(e => e.Name.ToLowerInvariant()).ToHashSet();

        if (Directory.Exists(extDir))
        {
            foreach (var file in Directory.GetFiles(extDir, "*.dll"))
            {
                string fileName = Path.GetFileName(file);
                string norm = IniDocument.NormalizeExtensionName(fileName);
                result.Add(new PhpExtensionItem
                {
                    Name = norm,
                    FileName = fileName,
                    IsEnabled = activeExts.Contains(norm.ToLowerInvariant())
                });
            }
        }
        else
        {
            // If physical ext directory not found, fallback to parsed from php.ini
            foreach (var ext in doc.ListExtensions())
            {
                result.Add(new PhpExtensionItem
                {
                    Name = ext.Name,
                    FileName = $"php_{ext.Name}.dll",
                    IsEnabled = ext.IsActive
                });
            }
        }

        return result.OrderBy(e => e.Name).ToList();
    }

    public static bool ToggleExtension(string extName, bool enable, string? iniPath = null)
    {
        string path = iniPath ?? _iniPath;
        if (!File.Exists(path)) return false;

        BackupIniFile(path);
        try
        {
            var doc = IniDocument.Load(path);
            if (enable)
            {
                doc.EnableExtension(extName);
            }
            else
            {
                doc.DisableExtension(extName);
            }
            doc.Save(path);
            AppLogger.Log($"Toggled PHP extension '{extName}' -> {(enable ? "Enabled" : "Disabled")}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error toggling extension {extName}: {ex.Message}");
            return false;
        }
    }

    public static (bool Success, string Diff, string Message) ApplyPreset(string presetName, string? phpDir = null)
    {
        string dir = phpDir ?? Path.GetDirectoryName(_iniPath) ?? @"C:\tools\php85";
        string presetFileName = presetName.Equals("production", StringComparison.OrdinalIgnoreCase)
            ? "php.ini-production"
            : "php.ini-development";

        string presetPath = Path.Combine(dir, presetFileName);
        if (!File.Exists(presetPath))
        {
            return (false, "", $"Preset file not found: {presetPath}");
        }

        if (!File.Exists(_iniPath))
        {
            return (false, "", $"Current php.ini not found: {_iniPath}");
        }

        try
        {
            string oldContent = File.ReadAllText(_iniPath);
            string newContent = File.ReadAllText(presetPath);

            var oldDoc = IniDocument.Parse(oldContent);
            string diff = oldDoc.ToDiff(newContent);

            BackupIniFile(_iniPath);
            File.WriteAllText(_iniPath, newContent);
            AppLogger.Log($"Applied PHP preset '{presetName}' from {presetFileName}");
            return (true, diff, $"Successfully applied preset '{presetName}'.");
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error applying PHP preset: {ex.Message}");
            return (false, "", ex.Message);
        }
    }
}
