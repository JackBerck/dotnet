using System.Text.RegularExpressions;

namespace dotnet.Services;

public class PhpSettingInfo
{
    public string Key { get; set; } = string.Empty;
    public string Value { get; set; } = string.Empty;
}

public class PhpConfigManager
{
    private static readonly string DefaultIniPath = @"C:\tools\php85\php.ini";

    public static string GetIniPath() => DefaultIniPath;

    public static bool BackupIniFile(string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? DefaultIniPath : iniPath;
        if (!File.Exists(iniPath)) return false;

        try
        {
            string backupDir = dotnet.Persistence.AppPaths.GetPath("backups");
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

    public static string GetSettingValue(string key, string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? DefaultIniPath : iniPath;
        if (!File.Exists(iniPath)) return "File not found";

        try
        {
            var regex = new Regex($@"^\s*{Regex.Escape(key)}\s*=\s*(.*)$", RegexOptions.IgnoreCase | RegexOptions.Multiline);
            string content = File.ReadAllText(iniPath);
            var match = regex.Match(content);
            if (match.Success)
            {
                return match.Groups[1].Value.Trim();
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reading php.ini setting {key}: {ex.Message}");
        }

        return "Not Set";
    }

    public static bool UpdateSetting(string key, string value, string iniPath = "")
    {
        iniPath = string.IsNullOrEmpty(iniPath) ? DefaultIniPath : iniPath;
        if (!File.Exists(iniPath))
        {
            AppLogger.Log($"php.ini file not found at {iniPath}");
            return false;
        }

        BackupIniFile(iniPath);

        try
        {
            string[] lines = File.ReadAllLines(iniPath);
            bool updated = false;
            var keyRegex = new Regex($@"^\s*;?\s*{Regex.Escape(key)}\s*=", RegexOptions.IgnoreCase);

            for (int i = 0; i < lines.Length; i++)
            {
                if (keyRegex.IsMatch(lines[i]))
                {
                    lines[i] = $"{key} = {value}";
                    updated = true;
                    break;
                }
            }

            if (updated)
            {
                File.WriteAllLines(iniPath, lines);
                AppLogger.Log($"Updated php.ini: {key} = {value}");
                return true;
            }
            else
            {
                // Append if not found
                File.AppendAllText(iniPath, $"{Environment.NewLine}{key} = {value}");
                AppLogger.Log($"Added to php.ini: {key} = {value}");
                return true;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error modifying php.ini: {ex.Message}");
            return false;
        }
    }
}
