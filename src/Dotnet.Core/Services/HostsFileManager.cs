using System.Net;
using System.Security.Principal;
using System.Text;
using System.Text.RegularExpressions;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class HostsFileManager
{
    public const string BlockStart = "# >>> Dotnet (managed) >>>";
    public const string BlockEnd = "# <<< Dotnet (managed) <<<";

    private static readonly string DefaultHostsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), @"drivers\etc\hosts");

    public static string GetHostsPath() => DefaultHostsPath;

    public static bool IsElevated()
    {
        if (!OperatingSystem.IsWindows()) return false;
        try
        {
            using var identity = WindowsIdentity.GetCurrent();
            var principal = new WindowsPrincipal(identity);
            return principal.IsInRole(WindowsBuiltInRole.Administrator);
        }
        catch
        {
            return false;
        }
    }

    public static bool IsValidHostname(string hostname, out string error)
    {
        error = string.Empty;
        if (string.IsNullOrWhiteSpace(hostname))
        {
            error = "Hostname cannot be empty.";
            return false;
        }

        hostname = hostname.Trim();
        if (hostname.Length > 255)
        {
            error = "Hostname exceeds 255 characters.";
            return false;
        }

        // Standard RFC 1123 hostname check
        var regex = new Regex(@"^(?=.{1,255}$)[0-9A-Za-z](?:(?:[0-9A-Za-z]|-){0,61}[0-9A-Za-z])?(?:\.[0-9A-Za-z](?:(?:[0-9A-Za-z]|-){0,61}[0-9A-Za-z])?)*$");
        if (!regex.IsMatch(hostname))
        {
            error = "Invalid hostname format.";
            return false;
        }

        return true;
    }

    public static List<HostEntryInfo> ParseContent(string content)
    {
        var list = new List<HostEntryInfo>();
        using var reader = new StringReader(content);
        string? line;
        int lineNum = 0;
        bool inManagedBlock = false;

        while ((line = reader.ReadLine()) != null)
        {
            lineNum++;
            string trimmed = line.Trim();

            if (trimmed.Equals(BlockStart, StringComparison.OrdinalIgnoreCase))
            {
                inManagedBlock = true;
                continue;
            }
            if (trimmed.Equals(BlockEnd, StringComparison.OrdinalIgnoreCase))
            {
                inManagedBlock = false;
                continue;
            }

            if (string.IsNullOrWhiteSpace(trimmed)) continue;

            bool isCommented = trimmed.StartsWith("#");
            string cleanLine = isCommented ? trimmed.TrimStart('#').Trim() : trimmed;

            // Split into IP and hostnames/comments
            var parts = cleanLine.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length >= 2 && IPAddress.TryParse(parts[0], out _))
            {
                string ip = parts[0];
                for (int i = 1; i < parts.Length; i++)
                {
                    string candidate = parts[i];
                    if (candidate.StartsWith("#"))
                    {
                        break; // Remaining parts are comments
                    }

                    list.Add(new HostEntryInfo
                    {
                        IpAddress = ip,
                        Hostname = candidate,
                        Comment = parts.Length > i + 1 ? string.Join(" ", parts.Skip(i + 1)) : "",
                        IsEnabled = !isCommented,
                        LineNumber = lineNum,
                        RawLine = line,
                        IsManaged = inManagedBlock
                    });
                }
            }
            else if (!isCommented && !inManagedBlock)
            {
                // Other non-comment lines
            }
        }

        return list;
    }

    public static string BuildContent(string originalContent, IEnumerable<HostEntryInfo> managedEntries)
    {
        var beforeBlock = new List<string>();
        var afterBlock = new List<string>();

        using (var reader = new StringReader(originalContent))
        {
            string? line;
            bool inBlock = false;
            bool passedBlock = false;

            while ((line = reader.ReadLine()) != null)
            {
                string trimmed = line.Trim();
                if (trimmed.Equals(BlockStart, StringComparison.OrdinalIgnoreCase))
                {
                    inBlock = true;
                    continue;
                }
                if (trimmed.Equals(BlockEnd, StringComparison.OrdinalIgnoreCase))
                {
                    inBlock = false;
                    passedBlock = true;
                    continue;
                }

                if (inBlock)
                {
                    // Skip old managed block content
                    continue;
                }

                if (!passedBlock)
                {
                    beforeBlock.Add(line);
                }
                else
                {
                    afterBlock.Add(line);
                }
            }
        }

        var sb = new StringBuilder();
        foreach (var l in beforeBlock)
        {
            sb.AppendLine(l);
        }

        // Add Managed Block
        sb.AppendLine(BlockStart);
        foreach (var entry in managedEntries)
        {
            string prefix = entry.IsEnabled ? "" : "# ";
            string comment = string.IsNullOrWhiteSpace(entry.Comment) ? "" : $"\t# {entry.Comment}";
            sb.AppendLine($"{prefix}{entry.IpAddress}\t{entry.Hostname}{comment}");
        }
        sb.AppendLine(BlockEnd);

        foreach (var l in afterBlock)
        {
            sb.AppendLine(l);
        }

        return sb.ToString().TrimEnd() + Environment.NewLine;
    }

    public static List<HostEntryInfo> ReadEntries(string? path = null)
    {
        string targetPath = path ?? DefaultHostsPath;
        if (!File.Exists(targetPath)) return new List<HostEntryInfo>();

        try
        {
            string content = File.ReadAllText(targetPath);
            return ParseContent(content);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reading hosts file: {ex.Message}");
            return new List<HostEntryInfo>();
        }
    }

    public static bool BackupHostsFile(string? path = null)
    {
        string targetPath = path ?? DefaultHostsPath;
        if (!File.Exists(targetPath)) return false;

        try
        {
            string backupDir = AppPaths.GetPath("backups");
            Directory.CreateDirectory(backupDir);
            string backupFile = Path.Combine(backupDir, $"hosts_{DateTime.Now:yyyyMMdd_HHmmss}.bak");
            File.Copy(targetPath, backupFile, true);
            AppLogger.Log($"Hosts backup created at: {backupFile}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error backing up hosts file: {ex.Message}");
            return false;
        }
    }

    public static bool SaveManagedEntries(IEnumerable<HostEntryInfo> managedEntries, string? path = null)
    {
        string targetPath = path ?? DefaultHostsPath;
        if (path == null && !IsElevated())
        {
            AppLogger.Log("Administrator privileges required to modify hosts file.");
            return false;
        }

        BackupHostsFile(targetPath);

        string existingContent = File.Exists(targetPath) ? File.ReadAllText(targetPath) : "";
        string updatedContent = BuildContent(existingContent, managedEntries);

        // Atomic write with retry
        for (int retry = 0; retry < 3; retry++)
        {
            try
            {
                string tempFile = targetPath + ".tmp." + Guid.NewGuid().ToString("N");
                File.WriteAllText(tempFile, updatedContent, Encoding.UTF8);
                if (File.Exists(targetPath))
                {
                    File.Move(tempFile, targetPath, overwrite: true);
                }
                else
                {
                    File.Move(tempFile, targetPath);
                }
                AppLogger.Log("Hosts file updated successfully.");
                return true;
            }
            catch (Exception ex)
            {
                if (retry == 2)
                {
                    AppLogger.Log($"Failed to write hosts file: {ex.Message}");
                    return false;
                }
                Thread.Sleep(100);
            }
        }

        return false;
    }

    public static bool AddOrUpdateEntry(string ip, string hostname, bool enable = true, string? path = null)
    {
        if (!IsValidHostname(hostname, out string error))
        {
            AppLogger.Log($"Invalid hostname: {error}");
            return false;
        }

        var entries = ReadEntries(path);
        var managed = entries.Where(e => e.IsManaged).ToList();

        var existing = managed.FirstOrDefault(e => e.Hostname.Equals(hostname, StringComparison.OrdinalIgnoreCase));
        if (existing != null)
        {
            existing.IpAddress = ip;
            existing.IsEnabled = enable;
        }
        else
        {
            managed.Add(new HostEntryInfo
            {
                IpAddress = ip,
                Hostname = hostname,
                IsEnabled = enable,
                IsManaged = true
            });
        }

        return SaveManagedEntries(managed, path);
    }

    public static bool RemoveEntry(string hostname, string? path = null)
    {
        var entries = ReadEntries(path);
        var managed = entries.Where(e => e.IsManaged).ToList();
        int countBefore = managed.Count;
        managed.RemoveAll(e => e.Hostname.Equals(hostname, StringComparison.OrdinalIgnoreCase));

        if (managed.Count == countBefore)
        {
            return false; // Not found in managed block
        }

        return SaveManagedEntries(managed, path);
    }

    public static bool ToggleEntry(string hostname, bool enable, string? path = null)
    {
        var entries = ReadEntries(path);
        var managed = entries.Where(e => e.IsManaged).ToList();
        var existing = managed.FirstOrDefault(e => e.Hostname.Equals(hostname, StringComparison.OrdinalIgnoreCase));
        if (existing == null) return false;

        existing.IsEnabled = enable;
        return SaveManagedEntries(managed, path);
    }
}
