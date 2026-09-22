using System.Diagnostics;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class ProcessRecord
{
    public string ServiceId { get; set; } = string.Empty;
    public int Pid { get; set; }
    public DateTime StartTime { get; set; }
    public string ExecutablePath { get; set; } = string.Empty;
}

public class ProcessTrackerState
{
    public List<ProcessRecord> Records { get; set; } = new();
}

public static class ProcessTracker
{
    private static readonly string StateFilePath = AppPaths.GetPath(Path.Combine("state", "running-processes.json"));
    private static readonly object LockObj = new();

    public static event Action<string, int>? ProcessCrashed;

    public static void RecordProcess(string serviceId, int pid, string executablePath)
    {
        lock (LockObj)
        {
            try
            {
                var state = LoadState();
                state.Records.RemoveAll(r => r.ServiceId.Equals(serviceId, StringComparison.OrdinalIgnoreCase));

                DateTime startTime = DateTime.UtcNow;
                try
                {
                    using var proc = Process.GetProcessById(pid);
                    startTime = proc.StartTime.ToUniversalTime();
                }
                catch
                {
                    // Ignore if unable to query process start time
                }

                state.Records.Add(new ProcessRecord
                {
                    ServiceId = serviceId,
                    Pid = pid,
                    StartTime = startTime,
                    ExecutablePath = executablePath
                });

                SaveState(state);
            }
            catch (Exception ex)
            {
                AppLogger.Log($"ProcessTracker: Failed to record PID {pid} for {serviceId}: {ex.Message}");
            }
        }
    }

    public static void RemoveProcess(string serviceId)
    {
        lock (LockObj)
        {
            try
            {
                var state = LoadState();
                int removed = state.Records.RemoveAll(r => r.ServiceId.Equals(serviceId, StringComparison.OrdinalIgnoreCase));
                if (removed > 0)
                {
                    SaveState(state);
                }
            }
            catch (Exception ex)
            {
                AppLogger.Log($"ProcessTracker: Failed to remove {serviceId}: {ex.Message}");
            }
        }
    }

    public static List<ProcessRecord> GetTrackedProcesses()
    {
        lock (LockObj)
        {
            return LoadState().Records;
        }
    }

    public static Process? TryAdoptProcess(string serviceId, string expectedExecutablePath)
    {
        lock (LockObj)
        {
            var state = LoadState();
            var record = state.Records.FirstOrDefault(r => r.ServiceId.Equals(serviceId, StringComparison.OrdinalIgnoreCase));
            if (record == null)
            {
                return null;
            }

            try
            {
                var proc = Process.GetProcessById(record.Pid);
                if (proc.HasExited)
                {
                    state.Records.Remove(record);
                    SaveState(state);
                    return null;
                }

                // Verify PID reuse safety check (StartTime within 5 seconds of record, or process path matches)
                bool pathMatches = false;
                try
                {
                    if (!string.IsNullOrEmpty(proc.MainModule?.FileName) && !string.IsNullOrEmpty(expectedExecutablePath))
                    {
                        pathMatches = string.Equals(
                            Path.GetFullPath(proc.MainModule.FileName),
                            Path.GetFullPath(expectedExecutablePath),
                            StringComparison.OrdinalIgnoreCase);
                    }
                }
                catch
                {
                    // Access denied or 32/64 bit mismatch on MainModule
                    // Fallback to matching process name
                    string expectedName = Path.GetFileNameWithoutExtension(expectedExecutablePath);
                    pathMatches = string.Equals(proc.ProcessName, expectedName, StringComparison.OrdinalIgnoreCase);
                }

                if (!pathMatches)
                {
                    // PID was reused by another process
                    AppLogger.Log($"ProcessTracker: PID {record.Pid} exists but does not match executable '{expectedExecutablePath}'. Discarding record.");
                    state.Records.Remove(record);
                    SaveState(state);
                    return null;
                }

                AppLogger.Log($"ProcessTracker: Successfully adopted existing process for {serviceId} (PID: {proc.Id})");
                return proc;
            }
            catch (ArgumentException)
            {
                // Process does not exist
                state.Records.Remove(record);
                SaveState(state);
                return null;
            }
            catch (Exception ex)
            {
                AppLogger.Log($"ProcessTracker: Error adopting process {serviceId}: {ex.Message}");
                return null;
            }
        }
    }

    public static void NotifyCrash(string serviceId, int exitCode)
    {
        AppLogger.Log($"ProcessTracker: Service '{serviceId}' crashed with exit code {exitCode}");
        RemoveProcess(serviceId);
        ProcessCrashed?.Invoke(serviceId, exitCode);
    }

    private static ProcessTrackerState LoadState()
    {
        var doc = JsonStore.Load<ProcessTrackerState>(StateFilePath);
        return doc?.Data ?? new ProcessTrackerState();
    }

    private static void SaveState(ProcessTrackerState state)
    {
        JsonStore.Save(StateFilePath, state, schemaVersion: 1);
    }
}
