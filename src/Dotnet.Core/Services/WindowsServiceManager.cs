using System.ServiceProcess;
using dotnet.Models;

namespace dotnet.Services;

public class WindowsServiceManager
{
    public static ServiceStatus GetStatus(string serviceName)
    {
        if (string.IsNullOrWhiteSpace(serviceName))
            return ServiceStatus.NotInstalled;

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
        catch (InvalidOperationException)
        {
            // Service not found / not installed
            return ServiceStatus.NotInstalled;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error querying service {serviceName}: {ex.Message}");
            return ServiceStatus.Error;
        }
    }

    public static async Task<OperationResult> StartServiceAsync(string serviceName, TimeSpan timeout)
    {
        if (string.IsNullOrWhiteSpace(serviceName))
            return OperationResult.Fail("Service name is empty.");

        try
        {
            using var sc = new ServiceController(serviceName);
            if (sc.Status == ServiceControllerStatus.Running)
                return OperationResult.Ok($"{serviceName} is already running.");

            sc.Start();
            await Task.Run(() => sc.WaitForStatus(ServiceControllerStatus.Running, timeout));
            AppLogger.Log($"Started Windows Service: {serviceName}");
            return OperationResult.Ok($"Started {serviceName}");
        }
        catch (InvalidOperationException)
        {
            string msg = $"Service '{serviceName}' is not installed on this system.";
            AppLogger.Log($"Error starting service: {msg}");
            return OperationResult.Fail(msg);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error starting service {serviceName}: {ex.Message}");
            return OperationResult.Fail($"Failed to start {serviceName}: {ex.Message}", ex);
        }
    }

    public static async Task<OperationResult> StopServiceAsync(string serviceName, TimeSpan timeout)
    {
        if (string.IsNullOrWhiteSpace(serviceName))
            return OperationResult.Fail("Service name is empty.");

        try
        {
            using var sc = new ServiceController(serviceName);
            if (sc.Status == ServiceControllerStatus.Stopped)
                return OperationResult.Ok($"{serviceName} is already stopped.");

            if (!sc.CanStop)
            {
                string msg = $"Service '{serviceName}' cannot be stopped.";
                AppLogger.Log(msg);
                return OperationResult.Fail(msg);
            }

            sc.Stop();
            await Task.Run(() => sc.WaitForStatus(ServiceControllerStatus.Stopped, timeout));
            AppLogger.Log($"Stopped Windows Service: {serviceName}");
            return OperationResult.Ok($"Stopped {serviceName}");
        }
        catch (InvalidOperationException)
        {
            string msg = $"Service '{serviceName}' is not installed.";
            AppLogger.Log($"Error stopping service: {msg}");
            return OperationResult.Fail(msg);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error stopping service {serviceName}: {ex.Message}");
            return OperationResult.Fail($"Failed to stop {serviceName}: {ex.Message}", ex);
        }
    }
}
