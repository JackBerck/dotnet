using System.Diagnostics;
using System.Net.NetworkInformation;

namespace dotnet.Services;

public class PortConflictInfo
{
    public int Port { get; set; }
    public bool IsInUse { get; set; }
    public int ProcessId { get; set; }
    public string ProcessName { get; set; } = string.Empty;
}

public class PortConflictDetector
{
    public static int? GetProcessIdByPort(int port)
    {
        try
        {
            var properties = IPGlobalProperties.GetIPGlobalProperties();
            var tcpListeners = properties.GetActiveTcpListeners();

            if (tcpListeners.Any(endpoint => endpoint.Port == port))
            {
                // Retrieve PID using netstat via Process execution
                return GetPidFromNetstat(port);
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Port check exception for port {port}: {ex.Message}");
        }

        return null;
    }

    public static PortConflictInfo CheckPort(int port)
    {
        int? pid = GetProcessIdByPort(port);
        if (pid.HasValue)
        {
            string name = "Unknown Process";
            try
            {
                var proc = Process.GetProcessById(pid.Value);
                name = proc.ProcessName;
            }
            catch
            {
                // Process might have ended or permission restricted
            }

            return new PortConflictInfo
            {
                Port = port,
                IsInUse = true,
                ProcessId = pid.Value,
                ProcessName = name
            };
        }

        return new PortConflictInfo
        {
            Port = port,
            IsInUse = false,
            ProcessId = 0,
            ProcessName = "None"
        };
    }

    private static int? GetPidFromNetstat(int port)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = $"/c netstat -ano | findstr :{port}",
                UseShellExecute = false,
                RedirectStandardOutput = true,
                CreateNoWindow = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return null;

            string output = proc.StandardOutput.ReadToEnd();
            proc.WaitForExit();

            var lines = output.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
            foreach (var line in lines)
            {
                if (line.Contains("LISTENING"))
                {
                    var parts = line.Split(new[] { ' ' }, StringSplitOptions.RemoveEmptyEntries);
                    if (parts.Length > 0 && int.TryParse(parts[^1], out int pid))
                    {
                        return pid;
                    }
                }
            }
        }
        catch
        {
            // Netstat fallback failure ignored
        }

        return null;
    }
}
