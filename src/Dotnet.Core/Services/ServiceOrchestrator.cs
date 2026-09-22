using dotnet.Models;

namespace dotnet.Services;

public class ServiceOrchestrator
{
    public static List<DevServiceInfo> FilterProfileServices(IEnumerable<DevServiceInfo> allServices, ServiceProfile profile)
    {
        return allServices.Where(s => profile.ServiceIds.Contains(s.Id, StringComparer.OrdinalIgnoreCase)).ToList();
    }

    public static async Task StartProfileServicesAsync(
        IEnumerable<DevServiceInfo> allServices,
        ServiceProfile profile,
        Func<DevServiceInfo, Task<bool>> startServiceFunc)
    {
        var targetServices = FilterProfileServices(allServices, profile);

        // Start order: Databases first (mysql, postgres, redis), then web servers/php
        var ordered = targetServices.OrderBy(s => GetStartPriority(s.Id)).ToList();

        AppLogger.Log($"Orchestrator: Starting {ordered.Count} services for profile '{profile.Name}'");

        foreach (var svc in ordered)
        {
            await startServiceFunc(svc);
        }
    }

    public static async Task StopProfileServicesAsync(
        IEnumerable<DevServiceInfo> allServices,
        ServiceProfile profile,
        Func<DevServiceInfo, Task<bool>> stopServiceFunc)
    {
        var targetServices = FilterProfileServices(allServices, profile);

        // Stop order: Reverse (web servers first, then databases)
        var ordered = targetServices.OrderByDescending(s => GetStartPriority(s.Id)).ToList();

        AppLogger.Log($"Orchestrator: Stopping {ordered.Count} services for profile '{profile.Name}'");

        foreach (var svc in ordered)
        {
            await stopServiceFunc(svc);
        }
    }

    private static int GetStartPriority(string serviceId)
    {
        return serviceId.ToLowerInvariant() switch
        {
            "mysql" => 1,
            "postgres" => 2,
            "redis" => 3,
            "php-cgi" => 4,
            "nginx" => 5,
            _ => 10
        };
    }
}
