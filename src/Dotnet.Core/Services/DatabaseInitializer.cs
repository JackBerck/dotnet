using System.Diagnostics;
using dotnet.Config;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public static class DatabaseInitializer
{
    public static string GetDataDirectory(string serviceId)
    {
        return AppPaths.GetPath($"data/{serviceId.ToLowerInvariant()}");
    }

    public static bool IsInitialized(string serviceId)
    {
        string dataDir = GetDataDirectory(serviceId);
        if (!Directory.Exists(dataDir)) return false;

        string id = serviceId.ToLowerInvariant();
        if (id == "mysql" || id == "mariadb")
        {
            return Directory.Exists(Path.Combine(dataDir, "mysql")) ||
                   File.Exists(Path.Combine(dataDir, "ibdata1"));
        }
        if (id == "postgres" || id == "postgresql")
        {
            return File.Exists(Path.Combine(dataDir, "PG_VERSION"));
        }

        return Directory.GetFileSystemEntries(dataDir).Length > 0;
    }

    public static async Task<OperationResult> InitializeAsync(string serviceId, string binaryPath, CancellationToken ct = default)
    {
        string id = serviceId.ToLowerInvariant();
        string dataDir = GetDataDirectory(id);

        if (IsInitialized(id))
        {
            return new OperationResult(true, $"Database '{id}' is already initialized in {dataDir}");
        }

        Directory.CreateDirectory(dataDir);

        if (id == "mysql" || id == "mariadb")
        {
            return await InitializeMysqlAsync(binaryPath, dataDir, ct);
        }
        else if (id == "postgres" || id == "postgresql")
        {
            return await InitializePostgresAsync(binaryPath, dataDir, ct);
        }

        return new OperationResult(false, $"Unknown database service type '{serviceId}'");
    }

    private static async Task<OperationResult> InitializeMysqlAsync(string mysqldPath, string dataDir, CancellationToken ct)
    {
        if (!File.Exists(mysqldPath))
        {
            return new OperationResult(false, $"mysqld binary not found at: {mysqldPath}");
        }

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = mysqldPath,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };
            psi.ArgumentList.Add("--initialize-insecure");
            psi.ArgumentList.Add($"--datadir={dataDir}");
            psi.ArgumentList.Add("--console");

            AppLogger.Log($"Initializing MySQL in: {dataDir} via {mysqldPath}");

            using var process = Process.Start(psi);
            if (process == null)
            {
                return new OperationResult(false, "Failed to launch mysqld process for initialization");
            }

            var stderrTask = process.StandardError.ReadToEndAsync(ct);
            var stdoutTask = process.StandardOutput.ReadToEndAsync(ct);

            await process.WaitForExitAsync(ct);
            string stderr = await stderrTask;
            string stdout = await stdoutTask;

            if (process.ExitCode != 0)
            {
                AppLogger.Log($"MySQL initialization failed (code {process.ExitCode}): {stderr}");
                return new OperationResult(false, $"MySQL initialization failed with code {process.ExitCode}: {stderr}");
            }

            AppLogger.Log($"MySQL successfully initialized in {dataDir}");
            return new OperationResult(true, $"MySQL successfully initialized in {dataDir}");
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Exception during MySQL init: {ex.Message}");
            return new OperationResult(false, ex.Message, ex);
        }
    }

    private static async Task<OperationResult> InitializePostgresAsync(string postgresExePath, string dataDir, CancellationToken ct)
    {
        // initdb.exe is typically in the same folder as postgres.exe
        string binDir = Path.GetDirectoryName(postgresExePath) ?? "";
        string initDbPath = Path.Combine(binDir, "initdb.exe");

        if (!File.Exists(initDbPath))
        {
            return new OperationResult(false, $"initdb.exe binary not found at: {initDbPath}");
        }

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = initDbPath,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };
            psi.ArgumentList.Add("-D");
            psi.ArgumentList.Add(dataDir);
            psi.ArgumentList.Add("-U");
            psi.ArgumentList.Add("postgres");
            psi.ArgumentList.Add("-A");
            psi.ArgumentList.Add("trust");
            psi.ArgumentList.Add("--encoding=UTF8");

            AppLogger.Log($"Initializing PostgreSQL in: {dataDir} via {initDbPath}");

            using var process = Process.Start(psi);
            if (process == null)
            {
                return new OperationResult(false, "Failed to launch initdb process");
            }

            var stderrTask = process.StandardError.ReadToEndAsync(ct);
            var stdoutTask = process.StandardOutput.ReadToEndAsync(ct);

            await process.WaitForExitAsync(ct);
            string stderr = await stderrTask;
            string stdout = await stdoutTask;

            if (process.ExitCode != 0)
            {
                AppLogger.Log($"PostgreSQL initdb failed (code {process.ExitCode}): {stderr}");
                return new OperationResult(false, $"PostgreSQL initdb failed with code {process.ExitCode}: {stderr}");
            }

            AppLogger.Log($"PostgreSQL successfully initialized in {dataDir}");
            return new OperationResult(true, $"PostgreSQL successfully initialized in {dataDir}");
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Exception during PostgreSQL init: {ex.Message}");
            return new OperationResult(false, ex.Message, ex);
        }
    }

    public static List<string> BuildRuntimeArguments(string serviceId, int port)
    {
        string id = serviceId.ToLowerInvariant();
        string dataDir = GetDataDirectory(id);
        var args = new List<string>();

        if (id == "mysql" || id == "mariadb")
        {
            args.Add($"--datadir={dataDir}");
            args.Add($"--port={port}");
            args.Add("--console");
        }
        else if (id == "postgres" || id == "postgresql")
        {
            args.Add("-D");
            args.Add(dataDir);
            args.Add("-p");
            args.Add(port.ToString());
        }
        else if (id == "redis" || id == "memurai")
        {
            args.Add("--port");
            args.Add(port.ToString());
        }

        return args;
    }
}
