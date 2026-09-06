using System.Security.Principal;
using System.Text;
using dotnet.Models;

namespace dotnet.Services;

public class HostsFileManager
{
    private static readonly string HostsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), @"drivers\etc\hosts");

    public static bool IsElevated()
    {
        using var identity = WindowsIdentity.GetCurrent();
        var principal = new WindowsPrincipal(identity);
        return principal.IsInRole(WindowsBuiltInRole.Administrator);
    }

    public static List<HostEntryInfo> ReadEntries()
    {
        var list = new List<HostEntryInfo>();
        if (!File.Exists(HostsPath)) return list;

        string[] lines = File.ReadAllLines(HostsPath);
        for (int i = 0; i < lines.Length; i++)
        {
            string line = lines[i].Trim();
            if (string.IsNullOrWhiteSpace(line)) continue;

            bool isCommented = line.StartsWith("#");
            string cleanLine = isCommented ? line.TrimStart('#').Trim() : line;

            var parts = cleanLine.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length >= 2 && System.Net.IPAddress.TryParse(parts[0], out _))
            {
                list.Add(new HostEntryInfo
                {
                    IpAddress = parts[0],
                    Hostname = parts[1],
                    Comment = parts.Length > 2 ? string.Join(" ", parts.Skip(2)) : "",
                    IsEnabled = !isCommented,
                    LineNumber = i + 1,
                    RawLine = lines[i]
                });
            }
        }

        return list;
    }

    public static bool BackupHostsFile()
    {
        try
        {
            string backupDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "backups");
            Directory.CreateDirectory(backupDir);
            string backupFile = Path.Combine(backupDir, $"hosts_{DateTime.Now:yyyyMMdd_HHmmss}.bak");
            File.Copy(HostsPath, backupFile, true);
            AppLogger.Log($"Hosts file backup created at: {backupFile}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error backing up hosts file: {ex.Message}");
            return false;
        }
    }

    public static bool AddOrUpdateEntry(string ip, string hostname, bool enable = true)
    {
        if (!IsElevated())
        {
            AppLogger.Log("Error: Administrator privileges required to modify hosts file.");
            return false;
        }

        BackupHostsFile();

        try
        {
            var entries = ReadEntries();
            var existing = entries.FirstOrDefault(e => e.Hostname.Equals(hostname, StringComparison.OrdinalIgnoreCase));

            if (existing != null)
            {
                // Update existing line
                string[] lines = File.ReadAllLines(HostsPath);
                string prefix = enable ? "" : "# ";
                lines[existing.LineNumber - 1] = $"{prefix}{ip}\t{hostname}";
                File.WriteAllLines(HostsPath, lines);
                AppLogger.Log($"Updated host entry: {hostname} -> {ip} (Enabled: {enable})");
            }
            else
            {
                // Append new entry
                string prefix = enable ? "" : "# ";
                string entry = $"{Environment.NewLine}{prefix}{ip}\t{hostname}";
                File.AppendAllText(HostsPath, entry);
                AppLogger.Log($"Added new host entry: {hostname} -> {ip}");
            }

            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error updating hosts file: {ex.Message}");
            return false;
        }
    }

    public static bool RemoveEntry(string hostname)
    {
        if (!IsElevated())
        {
            AppLogger.Log("Error: Administrator privileges required to modify hosts file.");
            return false;
        }

        BackupHostsFile();

        try
        {
            string[] lines = File.ReadAllLines(HostsPath);
            var updatedLines = lines.Where(line =>
            {
                string cleanLine = line.TrimStart('#').Trim();
                var parts = cleanLine.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length >= 2 && parts[1].Equals(hostname, StringComparison.OrdinalIgnoreCase))
                {
                    return false; // exclude
                }
                return true;
            }).ToArray();

            File.WriteAllLines(HostsPath, updatedLines);
            AppLogger.Log($"Removed host entry: {hostname}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error removing host entry: {ex.Message}");
            return false;
        }
    }
}
