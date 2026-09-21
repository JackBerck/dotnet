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
        try
        {
            if (File.Exists(FilePath))
            {
                string json = File.ReadAllText(FilePath);
                var list = JsonSerializer.Deserialize<List<ServiceSettings>>(json);
                if (list != null)
                {
                    _settings = list.ToDictionary(s => s.ServiceId, s => s);
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error loading service settings: {ex.Message}");
        }
    }

    public static void Save()
    {
        try
        {
            string json = JsonSerializer.Serialize(_settings.Values.ToList(), new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(FilePath, json);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error saving service settings: {ex.Message}");
        }
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
