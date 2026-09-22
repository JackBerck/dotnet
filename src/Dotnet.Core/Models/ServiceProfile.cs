using dotnet.Persistence;
using dotnet.Services;

namespace dotnet.Models;

public class ServiceProfile
{
    public string Id { get; set; } = "standalone";
    public string Name { get; set; } = "Standalone (Default)";
    public List<string> ServiceIds { get; set; } = new();
    public List<string> OptionalServiceIds { get; set; } = new();
}

public class ProfileConfiguration
{
    public string ActiveProfileId { get; set; } = "standalone";
    public List<ServiceProfile> Profiles { get; set; } = new();
}

public static class ProfileStore
{
    private static readonly string FilePath = AppPaths.GetPath(Path.Combine("state", "profiles.json"));
    private static ProfileConfiguration _config = new();

    static ProfileStore()
    {
        Load();
    }

    public static string ActiveProfileId
    {
        get => _config.ActiveProfileId;
        set
        {
            _config.ActiveProfileId = value;
            Save();
        }
    }

    public static IReadOnlyList<ServiceProfile> Profiles => _config.Profiles.AsReadOnly();

    public static ServiceProfile GetActiveProfile()
    {
        return _config.Profiles.FirstOrDefault(p => p.Id.Equals(_config.ActiveProfileId, StringComparison.OrdinalIgnoreCase))
               ?? _config.Profiles.First();
    }

    public static void Load()
    {
        var doc = JsonStore.Load<ProfileConfiguration>(FilePath);
        if (doc?.Data != null && doc.Data.Profiles.Count > 0)
        {
            _config = doc.Data;
        }
        else
        {
            // Setup defaults
            _config = new ProfileConfiguration
            {
                ActiveProfileId = "standalone",
                Profiles = new List<ServiceProfile>
                {
                    new()
                    {
                        Id = "standalone",
                        Name = "Standalone (Local Databases & Web)",
                        ServiceIds = new() { "mysql", "postgres", "redis", "nginx", "php-cgi" },
                        OptionalServiceIds = new()
                    },
                    new()
                    {
                        Id = "docker",
                        Name = "Docker Coexistence (Nginx & PHP only)",
                        ServiceIds = new() { "nginx", "php-cgi" },
                        OptionalServiceIds = new() { "mysql", "postgres", "redis" }
                    }
                }
            };
            Save();
        }
    }

    public static void Save()
    {
        JsonStore.Save(FilePath, _config, schemaVersion: 1);
    }
}
