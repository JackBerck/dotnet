using System.IO.Compression;
using dotnet.Services;

namespace dotnet.Installation;

public class SafeExtractor
{
    public const long DefaultMaxUncompressedBytes = 2L * 1024 * 1024 * 1024; // 2 GB

    public static async Task ExtractZipAsync(
        string zipFilePath,
        string destinationDir,
        bool stripRoot = false,
        long maxBytes = DefaultMaxUncompressedBytes,
        IProgress<double>? progress = null,
        CancellationToken ct = default)
    {
        if (!File.Exists(zipFilePath))
        {
            throw new FileNotFoundException($"ZIP file not found: {zipFilePath}");
        }

        string fullDestDir = Path.GetFullPath(destinationDir);
        if (!Directory.Exists(fullDestDir))
        {
            Directory.CreateDirectory(fullDestDir);
        }

        using var archive = ZipFile.OpenRead(zipFilePath);
        long totalUncompressed = 0;

        // Determine root prefix if stripRoot is requested
        string? rootPrefix = null;
        if (stripRoot)
        {
            rootPrefix = DetermineRootPrefix(archive.Entries);
        }

        int totalEntries = archive.Entries.Count;
        int processedEntries = 0;

        foreach (var entry in archive.Entries)
        {
            ct.ThrowIfCancellationRequested();

            totalUncompressed += entry.Length;
            if (totalUncompressed > maxBytes)
            {
                throw new InvalidOperationException($"Zip-bomb protection triggered: total uncompressed size exceeds {maxBytes / (1024 * 1024)} MB.");
            }

            string relativePath = entry.FullName;
            if (!string.IsNullOrEmpty(rootPrefix) && relativePath.StartsWith(rootPrefix, StringComparison.OrdinalIgnoreCase))
            {
                relativePath = relativePath.Substring(rootPrefix.Length);
            }

            if (string.IsNullOrWhiteSpace(relativePath)) continue;

            // Zip-Slip security check
            string targetPath = Path.GetFullPath(Path.Combine(fullDestDir, relativePath));
            if (!targetPath.StartsWith(fullDestDir, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException($"Zip-Slip detected! Malicious entry: {entry.FullName}");
            }

            if (string.IsNullOrEmpty(entry.Name))
            {
                // Directory entry
                Directory.CreateDirectory(targetPath);
            }
            else
            {
                // File entry
                string? parentDir = Path.GetDirectoryName(targetPath);
                if (!string.IsNullOrEmpty(parentDir) && !Directory.Exists(parentDir))
                {
                    Directory.CreateDirectory(parentDir);
                }

                entry.ExtractToFile(targetPath, overwrite: true);
            }

            processedEntries++;
            progress?.Report((double)processedEntries / totalEntries * 100.0);
        }

        AppLogger.Log($"Extracted archive successfully to: {destinationDir}");
    }

    private static string? DetermineRootPrefix(IEnumerable<ZipArchiveEntry> entries)
    {
        string? candidate = null;
        foreach (var entry in entries)
        {
            string clean = entry.FullName.Replace('\\', '/');
            int slashIdx = clean.IndexOf('/');
            if (slashIdx <= 0)
            {
                // Root file found directly; cannot strip single root
                return null;
            }

            string prefix = clean.Substring(0, slashIdx + 1);
            if (candidate == null)
            {
                candidate = prefix;
            }
            else if (!candidate.Equals(prefix, StringComparison.OrdinalIgnoreCase))
            {
                // Multiple distinct root folders
                return null;
            }
        }
        return candidate;
    }

    public static void InstallSingleFile(string sourceFile, string destinationDir, string targetFileName)
    {
        string fullDestDir = Path.GetFullPath(destinationDir);
        if (!Directory.Exists(fullDestDir))
        {
            Directory.CreateDirectory(fullDestDir);
        }

        string targetPath = Path.Combine(fullDestDir, targetFileName);
        File.Copy(sourceFile, targetPath, overwrite: true);
        AppLogger.Log($"Installed single file to: {targetPath}");
    }
}
