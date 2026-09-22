using System.Diagnostics;
using System.Text.RegularExpressions;
using dotnet.Services;

namespace dotnet.Installation;

public class DiscoveredTool
{
    public string ToolId { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public string DetectedVersion { get; set; } = string.Empty;
    public string DirectoryPath { get; set; } = string.Empty;
    public string ExePath { get; set; } = string.Empty;
}

public class AdoptExistingScanner
{
    public static List<DiscoveredTool> ScanAll()
    {
        var discovered = new List<DiscoveredTool>();

        // 1. Scan C:\tools
        ScanToolsDirectory(discovered);

        // 2. Scan NVM / Node in PATH
        ScanNode(discovered);

        // 3. Scan Git in PATH
        ScanGit(discovered);

        // 4. Scan Bun
        ScanBun(discovered);

        // 5. Scan Go
        ScanGo(discovered);

        return discovered;
    }

    private static void ScanToolsDirectory(List<DiscoveredTool> list)
    {
        string toolsRoot = @"C:\tools";
        if (!Directory.Exists(toolsRoot)) return;

        try
        {
            foreach (var dir in Directory.GetDirectories(toolsRoot))
            {
                string dirName = Path.GetFileName(dir).ToLowerInvariant();

                // PHP check
                if (dirName.StartsWith("php") && File.Exists(Path.Combine(dir, "php.exe")))
                {
                    string ver = DetectExeVersion(Path.Combine(dir, "php.exe"), @"PHP (\d+\.\d+\.\d+)") ?? dirName.Replace("php", "");
                    list.Add(new DiscoveredTool
                    {
                        ToolId = "php",
                        DisplayName = $"PHP ({dirName})",
                        DetectedVersion = string.IsNullOrWhiteSpace(ver) ? "Local" : ver,
                        DirectoryPath = dir,
                        ExePath = Path.Combine(dir, "php.exe")
                    });
                }

                // Nginx check
                if (dirName.Contains("nginx") && File.Exists(Path.Combine(dir, "nginx.exe")))
                {
                    string ver = DetectExeVersion(Path.Combine(dir, "nginx.exe"), @"nginx/(\d+\.\d+\.\d+)") ?? "1.x";
                    list.Add(new DiscoveredTool
                    {
                        ToolId = "nginx",
                        DisplayName = "Nginx Web Server",
                        DetectedVersion = ver,
                        DirectoryPath = dir,
                        ExePath = Path.Combine(dir, "nginx.exe")
                    });
                }

                // MySQL check
                if (dirName.Contains("mysql") && File.Exists(Path.Combine(dir, "bin", "mysqld.exe")))
                {
                    list.Add(new DiscoveredTool
                    {
                        ToolId = "mysql",
                        DisplayName = $"MySQL ({dirName})",
                        DetectedVersion = dirName.Replace("mysql", ""),
                        DirectoryPath = dir,
                        ExePath = Path.Combine(dir, "bin", "mysqld.exe")
                    });
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error scanning C:\\tools: {ex.Message}");
        }
    }

    private static void ScanNode(List<DiscoveredTool> list)
    {
        string? nvmSymlink = System.Environment.GetEnvironmentVariable("NVM_SYMLINK");
        if (!string.IsNullOrEmpty(nvmSymlink) && File.Exists(Path.Combine(nvmSymlink, "node.exe")))
        {
            string ver = DetectExeVersion(Path.Combine(nvmSymlink, "node.exe"), @"v?(\d+\.\d+\.\d+)") ?? "NVM";
            list.Add(new DiscoveredTool
            {
                ToolId = "node",
                DisplayName = "Node.js (via NVM)",
                DetectedVersion = ver,
                DirectoryPath = nvmSymlink,
                ExePath = Path.Combine(nvmSymlink, "node.exe")
            });
            return;
        }

        string? nodeExe = WhereExe("node.exe");
        if (!string.IsNullOrEmpty(nodeExe))
        {
            string dir = Path.GetDirectoryName(nodeExe) ?? "";
            string ver = DetectExeVersion(nodeExe, @"v?(\d+\.\d+\.\d+)") ?? "System";
            list.Add(new DiscoveredTool
            {
                ToolId = "node",
                DisplayName = "Node.js",
                DetectedVersion = ver,
                DirectoryPath = dir,
                ExePath = nodeExe
            });
        }
    }

    private static void ScanGit(List<DiscoveredTool> list)
    {
        string? gitExe = WhereExe("git.exe");
        if (!string.IsNullOrEmpty(gitExe))
        {
            string dir = Path.GetDirectoryName(gitExe) ?? "";
            string ver = DetectExeVersion(gitExe, @"git version (\d+\.\d+\.\d+)") ?? "System";
            list.Add(new DiscoveredTool
            {
                ToolId = "git",
                DisplayName = "Git for Windows",
                DetectedVersion = ver,
                DirectoryPath = dir,
                ExePath = gitExe
            });
        }
    }

    private static void ScanBun(List<DiscoveredTool> list)
    {
        string userProfile = System.Environment.GetFolderPath(System.Environment.SpecialFolder.UserProfile);
        string bunExe = Path.Combine(userProfile, ".bun", "bin", "bun.exe");
        if (File.Exists(bunExe))
        {
            string ver = DetectExeVersion(bunExe, @"(\d+\.\d+\.\d+)") ?? "1.x";
            list.Add(new DiscoveredTool
            {
                ToolId = "bun",
                DisplayName = "Bun Runtime",
                DetectedVersion = ver,
                DirectoryPath = Path.GetDirectoryName(bunExe)!,
                ExePath = bunExe
            });
        }
    }

    private static void ScanGo(List<DiscoveredTool> list)
    {
        string? goExe = WhereExe("go.exe");
        if (string.IsNullOrEmpty(goExe))
        {
            string defaultPath = @"C:\Program Files\Go\bin\go.exe";
            if (File.Exists(defaultPath)) goExe = defaultPath;
        }

        if (!string.IsNullOrEmpty(goExe))
        {
            string dir = Path.GetDirectoryName(goExe) ?? "";
            string ver = DetectExeVersion(goExe, @"go(\d+\.\d+(\.\d+)?)") ?? "1.x";
            list.Add(new DiscoveredTool
            {
                ToolId = "go",
                DisplayName = "Go Language",
                DetectedVersion = ver,
                DirectoryPath = dir,
                ExePath = goExe
            });
        }
    }

    private static string? WhereExe(string exeName)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "where.exe",
                Arguments = exeName,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                CreateNoWindow = true
            };
            using var proc = Process.Start(psi);
            if (proc == null) return null;
            string output = proc.StandardOutput.ReadToEnd();
            proc.WaitForExit();

            if (proc.ExitCode == 0)
            {
                var lines = output.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
                if (lines.Length > 0 && File.Exists(lines[0].Trim()))
                {
                    return lines[0].Trim();
                }
            }
        }
        catch { }
        return null;
    }

    private static string? DetectExeVersion(string exePath, string regexPattern)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = exePath,
                Arguments = "-v",
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };
            using var proc = Process.Start(psi);
            if (proc == null) return null;
            string output = $"{proc.StandardOutput.ReadToEnd()}\n{proc.StandardError.ReadToEnd()}";
            proc.WaitForExit(3000);

            var match = Regex.Match(output, regexPattern, RegexOptions.IgnoreCase);
            if (match.Success)
            {
                return match.Groups[1].Value;
            }
        }
        catch { }
        return null;
    }

    public static void Adopt(DiscoveredTool tool, InstalledToolStore store)
    {
        store.RegisterTool(new InstalledToolRecord
        {
            ToolId = tool.ToolId,
            DisplayName = tool.DisplayName,
            Version = tool.DetectedVersion,
            InstallPath = tool.DirectoryPath,
            IsAdopted = true,
            IsActive = true,
            InstalledAt = DateTimeOffset.UtcNow
        });
        AppLogger.Log($"Adopted local tool: {tool.DisplayName} at {tool.DirectoryPath}");
    }
}
