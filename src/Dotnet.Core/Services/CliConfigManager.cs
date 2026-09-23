using System.Diagnostics;

namespace dotnet.Services;

public static class CliConfigManager
{
    private static async Task<(bool Success, string Output)> RunCliCommandAsync(string fileName, string arguments, int timeoutMs = 5000)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = fileName,
                Arguments = arguments,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var proc = new Process { StartInfo = psi };
            if (!proc.Start())
            {
                return (false, "Failed to start process.");
            }

            var stdoutTask = proc.StandardOutput.ReadToEndAsync();
            var stderrTask = proc.StandardError.ReadToEndAsync();

            var timeoutTask = Task.Delay(timeoutMs);
            var completed = await Task.WhenAny(Task.WhenAll(stdoutTask, stderrTask), timeoutTask);

            if (completed == timeoutTask)
            {
                try { proc.Kill(); } catch { }
                return (false, "Command timed out.");
            }

            string stdout = await stdoutTask;
            string stderr = await stderrTask;
            await proc.WaitForExitAsync();

            return (proc.ExitCode == 0, stdout.Trim());
        }
        catch (Exception ex)
        {
            return (false, ex.Message);
        }
    }

    public static async Task<string?> GetGitConfigAsync(string key)
    {
        var (ok, output) = await RunCliCommandAsync("git.exe", $"config --global {key}");
        return ok ? output : null;
    }

    public static async Task<bool> SetGitConfigAsync(string key, string value)
    {
        var (ok, _) = await RunCliCommandAsync("git.exe", $"config --global {key} \"{value}\"");
        if (ok)
        {
            AppLogger.Log($"Git config updated: {key} = {value}");
        }
        return ok;
    }

    public static async Task<string?> GetNpmConfigAsync(string key)
    {
        var (ok, output) = await RunCliCommandAsync("npm.cmd", $"config get {key}");
        return ok ? output : null;
    }

    public static async Task<bool> SetNpmConfigAsync(string key, string value)
    {
        var (ok, _) = await RunCliCommandAsync("npm.cmd", $"config set {key} \"{value}\"");
        if (ok)
        {
            AppLogger.Log($"npm config updated: {key} = {value}");
        }
        return ok;
    }
}
