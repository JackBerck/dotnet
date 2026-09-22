using System.Diagnostics;
using dotnet.Models;

namespace dotnet.Services;

public class ProcessServiceManager
{
    private static readonly Dictionary<string, Process> ManagedProcesses = new();
    private static readonly HashSet<string> IntentionallyStopping = new();
    private static readonly object SyncLock = new();

    public static void TryAdoptAll(IEnumerable<DevServiceInfo> services)
    {
        foreach (var svc in services)
        {
            if (svc.Type == DevServiceType.ManagedProcess)
            {
                GetStatus(svc);
            }
        }
    }

    public static ServiceStatus GetStatus(DevServiceInfo service)
    {
        lock (SyncLock)
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
                    ProcessTracker.RemoveProcess(service.Id);
                    service.ProcessId = null;
                }
            }

            // Attempt to re-adopt previously running process persisted in state
            var adopted = ProcessTracker.TryAdoptProcess(service.Id, service.ExecutablePath);
            if (adopted != null)
            {
                HookExitedEvent(service, adopted);
                ManagedProcesses[service.Id] = adopted;
                service.ProcessId = adopted.Id;
                return ServiceStatus.Running;
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

            HookExitedEvent(service, proc);

            if (proc.Start())
            {
                proc.BeginOutputReadLine();
                proc.BeginErrorReadLine();

                lock (SyncLock)
                {
                    ManagedProcesses[service.Id] = proc;
                    IntentionallyStopping.Remove(service.Id);
                }

                service.ProcessId = proc.Id;
                service.Status = ServiceStatus.Running;
                ProcessTracker.RecordProcess(service.Id, proc.Id, service.ExecutablePath);

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
            Process? proc = null;
            lock (SyncLock)
            {
                IntentionallyStopping.Add(service.Id);
                if (ManagedProcesses.TryGetValue(service.Id, out proc))
                {
                    ManagedProcesses.Remove(service.Id);
                }
            }

            ProcessTracker.RemoveProcess(service.Id);

            if (proc != null)
            {
                if (!proc.HasExited)
                {
                    proc.Kill(true);
                    proc.WaitForExit(3000);
                }
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
        finally
        {
            lock (SyncLock)
            {
                IntentionallyStopping.Remove(service.Id);
            }
        }

        return false;
    }

    private static void HookExitedEvent(DevServiceInfo service, Process proc)
    {
        try
        {
            proc.EnableRaisingEvents = true;
            proc.Exited += (s, e) =>
            {
                bool intentional = false;
                int exitCode = 0;
                try
                {
                    exitCode = proc.ExitCode;
                }
                catch { }

                lock (SyncLock)
                {
                    intentional = IntentionallyStopping.Contains(service.Id);
                    ManagedProcesses.Remove(service.Id);
                }

                ProcessTracker.RemoveProcess(service.Id);
                service.ProcessId = null;
                service.Status = ServiceStatus.Stopped;

                if (!intentional)
                {
                    ProcessTracker.NotifyCrash(service.Id, exitCode);
                }
                else
                {
                    AppLogger.Log($"Process {service.Name} (PID {proc.Id}) exited cleanly.");
                }
            };
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Warning: Could not hook Exited event on {service.Name}: {ex.Message}");
        }
    }
}
