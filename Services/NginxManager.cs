using System.Diagnostics;

namespace dotnet.Services;

public class NginxManager
{
    private static readonly string NginxDirectory = @"C:\tools\nginx";
    private static readonly string NginxExe = Path.Combine(NginxDirectory, "nginx.exe");

    public static (bool Success, string Output) ValidateConfig()
    {
        if (!File.Exists(NginxExe))
        {
            return (false, $"nginx.exe not found at {NginxExe}");
        }

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = NginxExe,
                Arguments = "-t",
                WorkingDirectory = NginxDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return (false, "Could not start nginx.exe for validation.");

            string stdout = proc.StandardOutput.ReadToEnd();
            string stderr = proc.StandardError.ReadToEnd();
            proc.WaitForExit();

            string fullOutput = $"{stdout}\n{stderr}".Trim();
            bool isOk = proc.ExitCode == 0 && fullOutput.Contains("test is successful");

            AppLogger.Log($"Nginx Config Test: {(isOk ? "PASSED" : "FAILED")}");
            return (isOk, fullOutput);
        }
        catch (Exception ex)
        {
            return (false, ex.Message);
        }
    }

    public static (bool Success, string Output) ReloadNginx()
    {
        var valResult = ValidateConfig();
        if (!valResult.Success)
        {
            AppLogger.Log("Nginx reload aborted due to configuration validation errors.");
            return valResult;
        }

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = NginxExe,
                Arguments = "-s reload",
                WorkingDirectory = NginxDirectory,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return (false, "Could not start nginx process for reload.");

            string stdout = proc.StandardOutput.ReadToEnd();
            string stderr = proc.StandardError.ReadToEnd();
            proc.WaitForExit();

            string result = $"{stdout}\n{stderr}".Trim();
            AppLogger.Log("Reloaded Nginx configuration successfully.");
            return (true, string.IsNullOrEmpty(result) ? "Nginx reloaded successfully." : result);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error reloading Nginx: {ex.Message}");
            return (false, ex.Message);
        }
    }
}
