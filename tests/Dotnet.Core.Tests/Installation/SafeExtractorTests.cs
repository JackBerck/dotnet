using System.IO.Compression;
using dotnet.Installation;

namespace Dotnet.Core.Tests.Installation;

public class SafeExtractorTests
{
    [Fact]
    public async Task ExtractZip_DetectsAndBlocks_ZipSlip()
    {
        string tempZip = Path.Combine(Path.GetTempPath(), $"test_slip_{Guid.NewGuid():N}.zip");
        string destDir = Path.Combine(Path.GetTempPath(), $"dest_slip_{Guid.NewGuid():N}");

        try
        {
            // Create a malicious zip with traversal
            using (var zip = ZipFile.Open(tempZip, ZipArchiveMode.Create))
            {
                var entry = zip.CreateEntry("../evil.exe");
                using var writer = new StreamWriter(entry.Open());
                writer.WriteLine("malicious");
            }

            await Assert.ThrowsAsync<InvalidOperationException>(async () =>
            {
                await SafeExtractor.ExtractZipAsync(tempZip, destDir);
            });
        }
        finally
        {
            if (File.Exists(tempZip)) File.Delete(tempZip);
            if (Directory.Exists(destDir)) Directory.Delete(destDir, true);
        }
    }

    [Fact]
    public async Task ExtractZip_ExtractsNormalArchive_Successfully()
    {
        string tempZip = Path.Combine(Path.GetTempPath(), $"test_ok_{Guid.NewGuid():N}.zip");
        string destDir = Path.Combine(Path.GetTempPath(), $"dest_ok_{Guid.NewGuid():N}");

        try
        {
            using (var zip = ZipFile.Open(tempZip, ZipArchiveMode.Create))
            {
                var entry = zip.CreateEntry("bin/app.exe");
                using var writer = new StreamWriter(entry.Open());
                writer.WriteLine("ok");
            }

            await SafeExtractor.ExtractZipAsync(tempZip, destDir);
            Assert.True(File.Exists(Path.Combine(destDir, "bin", "app.exe")));
        }
        finally
        {
            if (File.Exists(tempZip)) File.Delete(tempZip);
            if (Directory.Exists(destDir)) Directory.Delete(destDir, true);
        }
    }
}
