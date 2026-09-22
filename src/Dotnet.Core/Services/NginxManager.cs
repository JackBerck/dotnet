using System.Diagnostics;
using System.Text;

namespace dotnet.Services;

public class NginxManager
{
    private static string _nginxDirectory = @"C:\tools\nginx";

    public static string GetNginxDirectory() => _nginxDirectory;

    public static void SetNginxDirectory(string directory)
    {
        if (!string.IsNullOrWhiteSpace(directory))
        {
            _nginxDirectory = directory;
        }
    }

    private static string GetNginxExe() => Path.Combine(_nginxDirectory, "nginx.exe");

    private static async Task<(bool Success, int ExitCode, string Output)> RunNginxCommandAsync(string arguments, int timeoutMs = 8000)
    {
        string exe = GetNginxExe();
        if (!File.Exists(exe))
        {
            return (false, -1, $"nginx.exe not found at {exe}");
        }

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = exe,
                Arguments = arguments,
                WorkingDirectory = _nginxDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var proc = new Process { StartInfo = psi };
            if (!proc.Start())
            {
                return (false, -1, "Failed to start nginx process.");
            }

            // Read streams asynchronously in parallel to prevent deadlock
            var stdoutTask = proc.StandardOutput.ReadToEndAsync();
            var stderrTask = proc.StandardError.ReadToEndAsync();

            var timeoutTask = Task.Delay(timeoutMs);
            var completed = await Task.WhenAny(Task.WhenAll(stdoutTask, stderrTask), timeoutTask);

            if (completed == timeoutTask)
            {
                try { proc.Kill(); } catch { }
                return (false, -1, "Nginx command timed out.");
            }

            string stdout = await stdoutTask;
            string stderr = await stderrTask;
            await proc.WaitForExitAsync();

            string fullOutput = $"{stdout}\n{stderr}".Trim();
            return (proc.ExitCode == 0, proc.ExitCode, fullOutput);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error running nginx command '{arguments}': {ex.Message}");
            return (false, -1, ex.Message);
        }
    }

    public static async Task<(bool Success, string Output)> ValidateConfigAsync()
    {
        var (ok, exitCode, output) = await RunNginxCommandAsync($"-p \"{_nginxDirectory}\" -t");
        bool isOk = ok && (output.Contains("syntax is ok", StringComparison.OrdinalIgnoreCase) ||
                           output.Contains("test is successful", StringComparison.OrdinalIgnoreCase));

        AppLogger.Log($"Nginx Config Test: {(isOk ? "PASSED" : "FAILED")}");
        return (isOk, output);
    }

    public static (bool Success, string Output) ValidateConfig()
    {
        return Task.Run(async () => await ValidateConfigAsync()).GetAwaiter().GetResult();
    }

    public static async Task<(bool Success, string Output)> ReloadNginxAsync()
    {
        var valResult = await ValidateConfigAsync();
        if (!valResult.Success)
        {
            AppLogger.Log("Nginx reload aborted due to configuration validation errors.");
            return valResult;
        }

        var (ok, exitCode, output) = await RunNginxCommandAsync($"-p \"{_nginxDirectory}\" -s reload");
        if (ok)
        {
            AppLogger.Log("Reloaded Nginx configuration successfully.");
            return (true, string.IsNullOrEmpty(output) ? "Nginx reloaded successfully." : output);
        }
        else
        {
            AppLogger.Log($"Failed to reload Nginx (Exit code: {exitCode}): {output}");
            return (false, output);
        }
    }

    public static (bool Success, string Output) ReloadNginx()
    {
        return Task.Run(async () => await ReloadNginxAsync()).GetAwaiter().GetResult();
    }

    public static async Task<(bool Success, string Output)> StopGracefulAsync()
    {
        var (ok, exitCode, output) = await RunNginxCommandAsync($"-p \"{_nginxDirectory}\" -s quit");
        return (ok, output);
    }

    public static List<string> TailErrorLog(int maxLines = 50)
    {
        string logPath = Path.Combine(_nginxDirectory, "logs", "error.log");
        if (!File.Exists(logPath)) return new List<string>();

        try
        {
            // Open with FileShare.ReadWrite to not lock nginx
            using var fs = new FileStream(logPath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            using var sr = new StreamReader(fs, Encoding.UTF8);
            var lines = new List<string>();
            string? line;
            while ((line = sr.ReadLine()) != null)
            {
                lines.Add(line);
            }

            if (lines.Count > maxLines)
            {
                return lines.Skip(lines.Count - maxLines).ToList();
            }
            return lines;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reading nginx error log: {ex.Message}");
            return new List<string> { $"Error reading log: {ex.Message}" };
        }
    }
}
