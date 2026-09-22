using System.Diagnostics;
using dotnet.Services;

namespace dotnet.Installation;

public class JunctionManager
{
    public static bool CreateOrUpdateJunction(string junctionPath, string targetDirectory)
    {
        if (!OperatingSystem.IsWindows()) return false;

        try
        {
            if (Directory.Exists(junctionPath))
            {
                // Remove existing junction or folder
                DeleteJunction(junctionPath);
            }

            string? parent = Path.GetDirectoryName(junctionPath);
            if (!string.IsNullOrEmpty(parent) && !Directory.Exists(parent))
            {
                Directory.CreateDirectory(parent);
            }

            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = $"/c mklink /J \"{junctionPath}\" \"{targetDirectory}\"",
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return false;
            proc.WaitForExit();

            bool success = proc.ExitCode == 0 && Directory.Exists(junctionPath);
            if (success)
            {
                AppLogger.Log($"Directory junction created: {junctionPath} -> {targetDirectory}");
            }
            else
            {
                string err = proc.StandardError.ReadToEnd();
                AppLogger.Log($"Failed to create junction '{junctionPath}': {err}");
            }
            return success;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Exception creating junction: {ex.Message}");
            return false;
        }
    }

    public static bool DeleteJunction(string junctionPath)
    {
        if (!Directory.Exists(junctionPath)) return true;

        try
        {
            // Directory.Delete on a junction removes the junction link, not target contents
            Directory.Delete(junctionPath);
            return true;
        }
        catch
        {
            // Fallback: rmdir command
            try
            {
                var psi = new ProcessStartInfo
                {
                    FileName = "cmd.exe",
                    Arguments = $"/c rmdir \"{junctionPath}\"",
                    UseShellExecute = false,
                    CreateNoWindow = true
                };
                using var proc = Process.Start(psi);
                proc?.WaitForExit();
                return !Directory.Exists(junctionPath);
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Error deleting junction {junctionPath}: {ex.Message}");
                return false;
            }
        }
    }
}
