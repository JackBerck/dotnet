using System.Diagnostics;
using dotnet.Models;

namespace dotnet.Services;

public class ProcessServiceManager
{
    private static readonly Dictionary<string, Process> ManagedProcesses = new();

    public static ServiceStatus GetStatus(DevServiceInfo service)
    {
        if (ManagedProcesses.TryGetValue(service.Id, out var proc))
        {
            if (!proc.HasExited)
            {
                service.ProcessId = proc.Id;
                return ServiceStatus.Running;
            }
            else
            {
                ManagedProcesses.Remove(service.Id);
                service.ProcessId = null;
            }
        }

        if (service.Port > 0)
        {
            int? pid = PortConflictDetector.GetProcessIdByPort(service.Port);
            if (pid.HasValue)
            {
                service.ProcessId = pid.Value;
                return ServiceStatus.PortConflict;
            }
        }

        service.ProcessId = null;
        return ServiceStatus.Stopped;
    }

    public static bool StartProcess(DevServiceInfo service)
    {
        try
        {
            if (!File.Exists(service.ExecutablePath))
            {
                AppLogger.Log($"Error: Executable not found at {service.ExecutablePath}");
                return false;
            }

            var startInfo = new ProcessStartInfo
            {
                FileName = service.ExecutablePath,
                Arguments = service.Arguments,
                WorkingDirectory = string.IsNullOrEmpty(service.WorkingDirectory) 
                    ? Path.GetDirectoryName(service.ExecutablePath) ?? "" 
                    : service.WorkingDirectory,
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            var proc = new Process { StartInfo = startInfo };
            proc.EnableRaisingEvents = true;
            
            proc.OutputDataReceived += (s, e) => {
                if (!string.IsNullOrEmpty(e.Data))
                {
                    // Drain stdout to prevent hang
                }
            };
            
            proc.ErrorDataReceived += (s, e) => {
                if (!string.IsNullOrEmpty(e.Data))
                {
                    // Drain stderr to prevent hang
                }
            };

            proc.Exited += (s, e) =>
            {
                AppLogger.Log($"Process {service.Name} (PID {proc.Id}) exited.");
                ManagedProcesses.Remove(service.Id);
            };

            if (proc.Start())
            {
                proc.BeginOutputReadLine();
                proc.BeginErrorReadLine();
                
                ManagedProcesses[service.Id] = proc;
                service.ProcessId = proc.Id;
                service.Status = ServiceStatus.Running;
                AppLogger.Log($"Started {service.Name} (PID: {proc.Id}) on port {service.Port}");
                return true;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to start {service.Name}: {ex.Message}");
        }

        return false;
    }

    public static bool StopProcess(DevServiceInfo service)
    {
        try
        {
            if (ManagedProcesses.TryGetValue(service.Id, out var proc))
            {
                if (!proc.HasExited)
                {
                    proc.Kill(true);
                    proc.WaitForExit(3000);
                }
                ManagedProcesses.Remove(service.Id);
                service.ProcessId = null;
                service.Status = ServiceStatus.Stopped;
                AppLogger.Log($"Stopped process {service.Name}");
                return true;
            }

            if (service.Port > 0)
            {
                int? pid = PortConflictDetector.GetProcessIdByPort(service.Port);
                if (pid.HasValue)
                {
                    AppLogger.Log($"Warning: Port {service.Port} is in use by PID {pid.Value}, but it's not managed by us. Will not kill.");
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error stopping process {service.Name}: {ex.Message}");
        }

        return false;
    }
}
