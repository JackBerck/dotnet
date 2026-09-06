using System;
using System.IO;

namespace dotnet.Services;

public static class AppLogger
{
    public static event Action<string>? OnLog;
    private static readonly string LogFilePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "dev-manager.log");

    public static void Log(string message)
    {
        string formatted = $"[{DateTime.Now:HH:mm:ss}] {message}";
        try
        {
            File.AppendAllText(LogFilePath, formatted + Environment.NewLine);
        }
        catch
        {
            // Ignore file log errors
        }

        OnLog?.Invoke(formatted);
    }
}
