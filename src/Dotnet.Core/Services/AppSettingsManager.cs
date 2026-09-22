using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public static class AppSettingsManager
{
    private static readonly string FilePath = AppPaths.GetPath(Path.Combine("config", "appsettings.json"));
    private static AppSettings _settings = new();

    static AppSettingsManager()
    {
        Load();
    }

    public static AppSettings Settings => _settings;

    public static void Load()
    {
        var doc = JsonStore.Load<AppSettings>(FilePath);
        if (doc?.Data != null)
        {
            _settings = doc.Data;
        }
        else
        {
            _settings = new AppSettings();
            Save();
        }
    }

    public static void Save()
    {
        JsonStore.Save(FilePath, _settings, schemaVersion: 1);
    }

    public static void SetExitPreference(ExitActionPreference preference)
    {
        _settings.ExitPreference = preference;
        Save();
    }
}
