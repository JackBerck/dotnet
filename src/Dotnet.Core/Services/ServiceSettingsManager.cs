using System.Text.Json;
using dotnet.Models;

namespace dotnet.Services;

public class ServiceSettings
{
    public string ServiceId { get; set; } = string.Empty;
    public bool AutoStartOnBoot { get; set; } = true;
}

public class ServiceSettingsManager
{
    private static readonly string FilePath = dotnet.Persistence.AppPaths.GetPath("service-settings.json");
    private static Dictionary<string, ServiceSettings> _settings = new();

    static ServiceSettingsManager()
    {
        Load();
    }

    public static void Load()
    {
        var doc = dotnet.Persistence.JsonStore.Load<List<ServiceSettings>>(FilePath);
        if (doc?.Data != null)
        {
            _settings = doc.Data.ToDictionary(s => s.ServiceId, s => s);
        }
    }

    public static void Save()
    {
        dotnet.Persistence.JsonStore.Save(FilePath, _settings.Values.ToList(), schemaVersion: 1);
    }

    public static bool GetAutoStartOnBoot(string serviceId, bool defaultValue = true)
    {
        if (_settings.TryGetValue(serviceId, out var s))
        {
            return s.AutoStartOnBoot;
        }
        return defaultValue;
    }

    public static void SetAutoStartOnBoot(string serviceId, bool autoStart)
    {
        if (!_settings.TryGetValue(serviceId, out var s))
        {
            s = new ServiceSettings { ServiceId = serviceId };
            _settings[serviceId] = s;
        }
        s.AutoStartOnBoot = autoStart;
        Save();
        AppLogger.Log($"Service '{serviceId}' Auto-Start on Boot set to: {autoStart}");
    }
}
