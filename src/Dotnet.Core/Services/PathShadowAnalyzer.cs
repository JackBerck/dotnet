using Microsoft.Win32;

namespace dotnet.Services;

public class ShadowAnalysisResult
{
    public string BinaryName { get; set; } = string.Empty;
    public string UserPathLocation { get; set; } = string.Empty;
    public string SystemPathLocation { get; set; } = string.Empty;
    public bool IsShadowed { get; set; }
    public string Recommendation { get; set; } = string.Empty;
}

public class PathShadowAnalyzer
{
    public static List<ShadowAnalysisResult> Analyze(IEnumerable<string>? customManagedDirs = null)
    {
        var results = new List<ShadowAnalysisResult>();

        try
        {
            var systemDirs = GetPathDirectories(Registry.LocalMachine, @"SYSTEM\CurrentControlSet\Control\Session Manager\Environment");
            var userDirs = GetPathDirectories(Registry.CurrentUser, @"Environment");

            // Common binary names to inspect if no custom list given
            var commonBinaries = new[] { "php.exe", "node.exe", "composer.bat", "git.exe", "nginx.exe", "bun.exe", "go.exe", "mysqld.exe", "postgres.exe", "redis-server.exe" };

            // Find all directories under Dotnet tools or in User PATH
            var targetUserDirs = new List<string>(userDirs);
            if (customManagedDirs != null)
            {
                foreach (var dir in customManagedDirs)
                {
                    if (!targetUserDirs.Contains(dir, StringComparer.OrdinalIgnoreCase))
                    {
                        targetUserDirs.Add(dir);
                    }
                }
            }

            foreach (var binary in commonBinaries)
            {
                string? foundInUser = null;
                foreach (var uDir in targetUserDirs)
                {
                    try
                    {
                        if (Directory.Exists(uDir))
                        {
                            string candidate = Path.Combine(uDir, binary);
                            if (File.Exists(candidate))
                            {
                                foundInUser = candidate;
                                break;
                            }
                        }
                    }
                    catch { }
                }

                if (string.IsNullOrEmpty(foundInUser))
                {
                    continue;
                }

                string? foundInSystem = null;
                foreach (var sDir in systemDirs)
                {
                    try
                    {
                        if (Directory.Exists(sDir))
                        {
                            string candidate = Path.Combine(sDir, binary);
                            if (File.Exists(candidate))
                            {
                                foundInSystem = candidate;
                                break;
                            }
                        }
                    }
                    catch { }
                }

                bool isShadowed = !string.IsNullOrEmpty(foundInSystem);
                results.Add(new ShadowAnalysisResult
                {
                    BinaryName = binary,
                    UserPathLocation = foundInUser,
                    SystemPathLocation = foundInSystem ?? "None (Clean)",
                    IsShadowed = isShadowed,
                    Recommendation = isShadowed
                        ? $"System PATH precedes User PATH. Remove '{foundInSystem}' from System PATH to use Dotnet version."
                        : "OK - Managed version has precedence."
                });
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"PathShadowAnalyzer: Analysis error: {ex.Message}");
        }

        return results;
    }

    public static List<string> GetPathDirectories(RegistryKey rootKey, string subKeyPath)
    {
        var dirs = new List<string>();
        try
        {
            using var key = rootKey.OpenSubKey(subKeyPath, false);
            string? rawPath = key?.GetValue("Path", null, RegistryValueOptions.DoNotExpandEnvironmentNames) as string;
            if (string.IsNullOrWhiteSpace(rawPath))
            {
                return dirs;
            }

            string expanded = Environment.ExpandEnvironmentVariables(rawPath);
            var parts = expanded.Split(';', StringSplitOptions.RemoveEmptyEntries);
            foreach (var part in parts)
            {
                string trimmed = part.Trim().Trim('"');
                if (!string.IsNullOrEmpty(trimmed) && !dirs.Contains(trimmed, StringComparer.OrdinalIgnoreCase))
                {
                    dirs.Add(trimmed);
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"PathShadowAnalyzer: Error reading registry {subKeyPath}: {ex.Message}");
        }

        return dirs;
    }
}
