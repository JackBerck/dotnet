using System.Text;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class PhpWorkerInfo
{
    public int Port { get; set; }
    public int? ProcessId { get; set; }
    public bool IsRunning { get; set; }
}

public static class PhpPoolManager
{
    private static readonly string UpstreamConfPath = AppPaths.GetPath(Path.Combine("nginx", "upstream_php.conf"));
    private static int _poolSize = 2;
    private static int _basePort = 9000;

    public static int PoolSize
    {
        get => _poolSize;
        set => _poolSize = Math.Clamp(value, 1, 8);
    }

    public static int BasePort
    {
        get => _basePort;
        set => _basePort = value;
    }

    public static string GenerateUpstreamBlock(int poolSize, int basePort = 9000)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# >>> Dotnet PHP FastCGI Pool >>>");
        sb.AppendLine("upstream php_pool {");
        for (int i = 0; i < poolSize; i++)
        {
            sb.AppendLine($"    server 127.0.0.1:{basePort + i};");
        }
        sb.AppendLine("}");
        sb.AppendLine("# <<< Dotnet PHP FastCGI Pool <<<");
        return sb.ToString();
    }

    public static void SaveUpstreamConfig(int poolSize, int basePort = 9000)
    {
        string dir = Path.GetDirectoryName(UpstreamConfPath) ?? "";
        if (!Directory.Exists(dir))
        {
            Directory.CreateDirectory(dir);
        }

        string content = GenerateUpstreamBlock(poolSize, basePort);
        File.WriteAllText(UpstreamConfPath, content, Encoding.UTF8);
        AppLogger.Log($"Generated PHP upstream configuration for {poolSize} workers at {UpstreamConfPath}");
    }

    public static async Task<bool> StartPoolAsync(string phpCgiPath, int poolSize = 2, int basePort = 9000)
    {
        PoolSize = poolSize;
        BasePort = basePort;
        SaveUpstreamConfig(poolSize, basePort);

        bool allStarted = true;
        for (int i = 0; i < poolSize; i++)
        {
            int port = basePort + i;
            var svc = new DevServiceInfo
            {
                Id = i == 0 ? "php-cgi" : $"php-cgi-{port}",
                Name = $"PHP FastCGI (Port {port})",
                Type = DevServiceType.ManagedProcess,
                ExecutablePath = phpCgiPath,
                Arguments = $"-b 127.0.0.1:{port}",
                Port = port
            };

            bool started = ProcessServiceManager.StartProcess(svc);
            if (!started)
            {
                allStarted = false;
                AppLogger.Log($"Failed to start PHP pool worker on port {port}");
            }
        }

        return allStarted;
    }

    public static void StopPool(int poolSize = 2, int basePort = 9000)
    {
        for (int i = 0; i < poolSize; i++)
        {
            int port = basePort + i;
            var svc = new DevServiceInfo
            {
                Id = i == 0 ? "php-cgi" : $"php-cgi-{port}",
                Name = $"PHP FastCGI (Port {port})",
                Type = DevServiceType.ManagedProcess,
                Port = port
            };

            ProcessServiceManager.StopProcess(svc);
        }
        AppLogger.Log($"Stopped PHP pool workers ({poolSize} workers).");
    }

    public static List<PhpWorkerInfo> GetPoolStatus(int poolSize = 2, int basePort = 9000)
    {
        var list = new List<PhpWorkerInfo>();
        for (int i = 0; i < poolSize; i++)
        {
            int port = basePort + i;
            int? pid = PortConflictDetector.GetProcessIdByPort(port);
            list.Add(new PhpWorkerInfo
            {
                Port = port,
                ProcessId = pid,
                IsRunning = pid.HasValue
            });
        }
        return list;
    }
}
