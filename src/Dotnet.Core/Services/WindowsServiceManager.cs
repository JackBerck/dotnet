using System.ServiceProcess;
using dotnet.Models;

namespace dotnet.Services;

public class WindowsServiceManager
{
    public static ServiceStatus GetStatus(string serviceName)
    {
        try
        {
            using var sc = new ServiceController(serviceName);
            return sc.Status switch
            {
                ServiceControllerStatus.Running => ServiceStatus.Running,
                ServiceControllerStatus.Stopped => ServiceStatus.Stopped,
                ServiceControllerStatus.StartPending => ServiceStatus.Starting,
                ServiceControllerStatus.StopPending => ServiceStatus.Stopping,
                _ => ServiceStatus.Unknown
            };
        }
        catch
        {
            return ServiceStatus.Stopped;
        }
    }

    public static async Task<bool> StartServiceAsync(string serviceName, TimeSpan timeout)
    {
        try
        {
            using var sc = new ServiceController(serviceName);
            if (sc.Status == ServiceControllerStatus.Running) return true;

            sc.Start();
            await Task.Run(() => sc.WaitForStatus(ServiceControllerStatus.Running, timeout));
            AppLogger.Log($"Started Windows Service: {serviceName}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error starting service {serviceName}: {ex.Message}");
            return false;
        }
    }

    public static async Task<bool> StopServiceAsync(string serviceName, TimeSpan timeout)
    {
        try
        {
            using var sc = new ServiceController(serviceName);
            if (sc.Status == ServiceControllerStatus.Stopped) return true;

            sc.Stop();
            await Task.Run(() => sc.WaitForStatus(ServiceControllerStatus.Stopped, timeout));
            AppLogger.Log($"Stopped Windows Service: {serviceName}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error stopping service {serviceName}: {ex.Message}");
            return false;
        }
    }
}
