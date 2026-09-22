using System.Reflection;
using System.Text.Json;
using dotnet.Persistence;
using dotnet.Services;

namespace dotnet.Catalog;

public class CatalogLoader
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        AllowTrailingCommas = true,
        ReadCommentHandling = JsonCommentHandling.Skip
    };

    private readonly List<ToolDefinition> _tools = new();

    public IReadOnlyList<ToolDefinition> Tools => _tools.AsReadOnly();

    public void LoadCatalog()
    {
        _tools.Clear();
        var toolMap = new Dictionary<string, ToolDefinition>(StringComparer.OrdinalIgnoreCase);

        // 1. Load embedded manifests
        var assembly = Assembly.GetExecutingAssembly();
        var resourceNames = assembly.GetManifestResourceNames()
            .Where(r => r.EndsWith(".json", StringComparison.OrdinalIgnoreCase));

        foreach (var resName in resourceNames)
        {
            try
            {
                using var stream = assembly.GetManifestResourceStream(resName);
                if (stream == null) continue;

                using var reader = new StreamReader(stream);
                string json = reader.ReadToEnd();
                var tool = JsonSerializer.Deserialize<ToolDefinition>(json, JsonOptions);
                if (tool != null && !string.IsNullOrWhiteSpace(tool.Id))
                {
                    toolMap[tool.Id] = tool;
                }
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Error loading embedded manifest '{resName}': {ex.Message}");
            }
        }

        // 2. Load from local catalog-cache directory if exists
        string cacheDir = AppPaths.GetPath("catalog-cache");
        if (Directory.Exists(cacheDir))
        {
            foreach (var file in Directory.GetFiles(cacheDir, "*.json"))
            {
                try
                {
                    string json = File.ReadAllText(file);
                    var tool = JsonSerializer.Deserialize<ToolDefinition>(json, JsonOptions);
                    if (tool != null && !string.IsNullOrWhiteSpace(tool.Id))
                    {
                        toolMap[tool.Id] = tool; // override with cache
                    }
                }
                catch (Exception ex)
                {
                    AppLogger.Log($"Error loading cached manifest '{file}': {ex.Message}");
                }
            }
        }

        _tools.AddRange(toolMap.Values.OrderBy(t => t.Category).ThenBy(t => t.DisplayName));
    }

    public ToolDefinition? GetTool(string id)
    {
        return _tools.FirstOrDefault(t => t.Id.Equals(id, StringComparison.OrdinalIgnoreCase));
    }
}
