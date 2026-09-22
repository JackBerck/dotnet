using System.Diagnostics;

namespace dotnet.Services;

public class TaskSchedulerManager
{
    private const string TaskName = "DotnetElevatedStartup";

    public static bool IsTaskScheduled()
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "schtasks.exe",
                Arguments = $"/Query /TN \"{TaskName}\"",
                CreateNoWindow = true,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return false;
            proc.WaitForExit(3000);
            return proc.ExitCode == 0;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"TaskScheduler: Query failed: {ex.Message}");
            return false;
        }
    }

    public static bool CreateLogonTask(string exePath, string arguments = "--autostart")
    {
        try
        {
            string taskRun = $"\\\"{exePath}\\\" {arguments}".Trim();
            var psi = new ProcessStartInfo
            {
                FileName = "schtasks.exe",
                Arguments = $"/Create /TN \"{TaskName}\" /TR \"{taskRun}\" /SC ONLOGON /RL HIGHEST /F",
                CreateNoWindow = true,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return false;
            string stdout = proc.StandardOutput.ReadToEnd();
            string stderr = proc.StandardError.ReadToEnd();
            proc.WaitForExit(5000);

            if (proc.ExitCode == 0)
            {
                AppLogger.Log($"TaskScheduler: Scheduled task '{TaskName}' created with highest privileges.");
                return true;
            }
            else
            {
                AppLogger.Log($"TaskScheduler: Failed to create task. Exit code: {proc.ExitCode}. Error: {stderr} {stdout}");
                return false;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"TaskScheduler: Exception creating task: {ex.Message}");
            return false;
        }
    }

    public static bool DeleteLogonTask()
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "schtasks.exe",
                Arguments = $"/Delete /TN \"{TaskName}\" /F",
                CreateNoWindow = true,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return false;
            proc.WaitForExit(5000);

            if (proc.ExitCode == 0)
            {
                AppLogger.Log($"TaskScheduler: Scheduled task '{TaskName}' deleted.");
                return true;
            }
            else
            {
                AppLogger.Log($"TaskScheduler: Task '{TaskName}' delete returned exit code: {proc.ExitCode}");
                return false;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"TaskScheduler: Exception deleting task: {ex.Message}");
            return false;
        }
    }
}
