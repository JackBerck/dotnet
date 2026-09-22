using System.Security.Cryptography;
using dotnet.Persistence;
using dotnet.Services;

namespace dotnet.Installation;

public class ToolDownloader
{
    private static readonly HttpClient HttpClient = new(new HttpClientHandler
    {
        AllowAutoRedirect = true,
        AutomaticDecompression = System.Net.DecompressionMethods.All
    })
    {
        Timeout = TimeSpan.FromMinutes(10)
    };

    public static async Task<string> DownloadAsync(
        string url,
        string expectedSha256,
        IEnumerable<string> allowedHosts,
        IProgress<double>? progress = null,
        CancellationToken ct = default)
    {
        if (!Uri.TryCreate(url, UriKind.Absolute, out var uri) || uri.Scheme != Uri.UriSchemeHttps)
        {
            throw new ArgumentException("Only HTTPS URLs are allowed.", nameof(url));
        }

        // Whitelist validation
        bool hostAllowed = allowedHosts.Any(h => uri.Host.Equals(h, StringComparison.OrdinalIgnoreCase) ||
                                                 uri.Host.EndsWith("." + h, StringComparison.OrdinalIgnoreCase));
        if (!hostAllowed)
        {
            throw new InvalidOperationException($"Host '{uri.Host}' is not in the allowed hosts list.");
        }

        string downloadsDir = AppPaths.GetPath("downloads");
        Directory.CreateDirectory(downloadsDir);

        string finalPath = Path.Combine(downloadsDir, $"{expectedSha256.ToLowerInvariant()}.bin");
        if (File.Exists(finalPath))
        {
            // Verify cached file
            if (await VerifySha256Async(finalPath, expectedSha256))
            {
                AppLogger.Log($"Using cached verified download: {finalPath}");
                progress?.Report(100.0);
                return finalPath;
            }
            File.Delete(finalPath);
        }

        string partPath = Path.Combine(downloadsDir, $"{expectedSha256.ToLowerInvariant()}.part.{Guid.NewGuid():N}");

        try
        {
            AppLogger.Log($"Starting download: {url}");
            using var response = await HttpClient.GetAsync(uri, HttpCompletionOption.ResponseHeadersRead, ct);
            response.EnsureSuccessStatusCode();

            long? totalBytes = response.Content.Headers.ContentLength;

            using var contentStream = await response.Content.ReadAsStreamAsync(ct);
            using var fileStream = new FileStream(partPath, FileMode.Create, FileAccess.Write, FileShare.None);
            using var sha256 = SHA256.Create();

            byte[] buffer = new byte[81920]; // 80KB buffer
            long totalRead = 0;
            int bytesRead;

            while ((bytesRead = await contentStream.ReadAsync(buffer, 0, buffer.Length, ct)) > 0)
            {
                await fileStream.WriteAsync(buffer, 0, bytesRead, ct);
                sha256.TransformBlock(buffer, 0, bytesRead, null, 0);

                totalRead += bytesRead;
                if (totalBytes.HasValue && totalBytes.Value > 0)
                {
                    progress?.Report((double)totalRead / totalBytes.Value * 100.0);
                }
            }

            sha256.TransformFinalBlock(Array.Empty<byte>(), 0, 0);
            string computedHash = Convert.ToHexString(sha256.Hash!).ToLowerInvariant();

            fileStream.Close();

            if (!string.Equals(computedHash, expectedSha256.Trim(), StringComparison.OrdinalIgnoreCase))
            {
                File.Delete(partPath);
                throw new InvalidOperationException($"SHA-256 verification failed! Expected: {expectedSha256}, Computed: {computedHash}");
            }

            if (File.Exists(finalPath)) File.Delete(finalPath);
            File.Move(partPath, finalPath);

            AppLogger.Log($"Download and SHA256 verification complete: {finalPath}");
            progress?.Report(100.0);
            return finalPath;
        }
        catch (Exception ex)
        {
            if (File.Exists(partPath))
            {
                try { File.Delete(partPath); } catch { }
            }
            AppLogger.Log($"Download failed for {url}: {ex.Message}");
            throw;
        }
    }

    public static async Task<bool> VerifySha256Async(string filePath, string expectedSha256)
    {
        if (!File.Exists(filePath)) return false;

        try
        {
            using var fs = File.OpenRead(filePath);
            using var sha256 = SHA256.Create();
            byte[] hashBytes = await sha256.ComputeHashAsync(fs);
            string hash = Convert.ToHexString(hashBytes);
            return string.Equals(hash, expectedSha256.Trim(), StringComparison.OrdinalIgnoreCase);
        }
        catch
        {
            return false;
        }
    }
}
