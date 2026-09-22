using dotnet.Catalog;
using dotnet.Env;
using dotnet.Persistence;
using dotnet.Services;

namespace dotnet.Installation;

public class ToolInstaller
{
    private static string GetToolsRoot()
    {
        return AppPaths.GetPath("tools");
    }

    public static async Task<bool> InstallAsync(
        ToolDefinition tool,
        ToolVersion version,
        InstalledToolStore store,
        IProgress<double>? progress = null,
        CancellationToken ct = default)
    {
        string toolsRoot = GetToolsRoot();
        string targetDir = Path.Combine(toolsRoot, tool.Id, version.Version);
        string currentJunction = Path.Combine(toolsRoot, tool.Id, "current");
        string stagingDir = Path.Combine(toolsRoot, ".staging", Guid.NewGuid().ToString("N"));

        try
        {
            Directory.CreateDirectory(Path.Combine(toolsRoot, ".staging"));

            // 1. Download source
            string archiveFile;
            if (version.Source.Type == "adopt")
            {
                archiveFile = "";
            }
            else
            {
                archiveFile = await ToolDownloader.DownloadAsync(
                    version.Source.Url,
                    version.Source.Sha256,
                    tool.AllowedHosts,
                    progress,
                    ct);
            }

            // 2. Extract or install to staging
            if (version.Source.Type == "single-file")
            {
                SafeExtractor.InstallSingleFile(archiveFile, stagingDir, version.Source.FileName);
            }
            else if (version.Source.Type == "zip")
            {
                await SafeExtractor.ExtractZipAsync(archiveFile, stagingDir, version.Source.StripRoot, SafeExtractor.DefaultMaxUncompressedBytes, null, ct);
            }

            // 3. Run Post-Install steps in staging
            PostInstallRunner.Run(tool, stagingDir);

            // 4. Move staging to targetDir
            if (Directory.Exists(targetDir))
            {
                Directory.Delete(targetDir, true);
            }
            string? targetParent = Path.GetDirectoryName(targetDir);
            if (!string.IsNullOrEmpty(targetParent) && !Directory.Exists(targetParent))
            {
                Directory.CreateDirectory(targetParent);
            }

            Directory.Move(stagingDir, targetDir);

            // 5. Create or update 'current' junction
            JunctionManager.CreateOrUpdateJunction(currentJunction, targetDir);

            // 6. Update PATH with binDirs pointing to current junction
            var addedPaths = new List<string>();
            foreach (var binRel in tool.Layout.BinDirs)
            {
                string binPath = Path.GetFullPath(Path.Combine(currentJunction, binRel));
                EnvironmentService.AddToUserPath(binPath, prepend: true);
                addedPaths.Add(binPath);
            }

            // 7. Register tool in store
            store.RegisterTool(new InstalledToolRecord
            {
                ToolId = tool.Id,
                DisplayName = tool.DisplayName,
                Version = version.Version,
                InstallPath = targetDir,
                IsAdopted = false,
                IsActive = true,
                AddedPathEntries = addedPaths,
                InstalledAt = DateTimeOffset.UtcNow
            });

            AppLogger.Log($"Successfully installed {tool.DisplayName} v{version.Version}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Installation failed for {tool.DisplayName}: {ex.Message}");
            if (Directory.Exists(stagingDir))
            {
                try { Directory.Delete(stagingDir, true); } catch { }
            }
            return false;
        }
    }

    public static bool UninstallAsync(InstalledToolRecord record, InstalledToolStore store)
    {
        try
        {
            // 1. Remove PATH entries
            foreach (var path in record.AddedPathEntries)
            {
                EnvironmentService.RemoveFromUserPath(path);
            }

            // 2. Remove junction and directory if not adopted
            if (!record.IsAdopted)
            {
                string toolsRoot = GetToolsRoot();
                string currentJunction = Path.Combine(toolsRoot, record.ToolId, "current");
                JunctionManager.DeleteJunction(currentJunction);

                if (Directory.Exists(record.InstallPath))
                {
                    Directory.Delete(record.InstallPath, true);
                }
            }

            // 3. Unregister
            store.UnregisterTool(record.ToolId, record.Version);
            AppLogger.Log($"Uninstalled {record.DisplayName} v{record.Version}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error uninstalling {record.DisplayName}: {ex.Message}");
            return false;
        }
    }
}
