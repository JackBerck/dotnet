using dotnet.Persistence;
using dotnet.Services;

namespace dotnet.Installation;

public class InstalledToolRecord
{
    public string ToolId { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public string Version { get; set; } = string.Empty;
    public string InstallPath { get; set; } = string.Empty;
    public bool IsAdopted { get; set; } = false;
    public bool IsActive { get; set; } = true;
    public List<string> AddedPathEntries { get; set; } = new();
    public DateTimeOffset InstalledAt { get; set; } = DateTimeOffset.UtcNow;
}

public class InstalledToolStore
{
    private static readonly string StateFilePath = AppPaths.GetPath(Path.Combine("state", "installed-tools.json"));
    private readonly List<InstalledToolRecord> _tools = new();

    public IReadOnlyList<InstalledToolRecord> Tools => _tools.AsReadOnly();

    public void Load()
    {
        _tools.Clear();
        var doc = JsonStore.Load<List<InstalledToolRecord>>(StateFilePath);
        if (doc?.Data != null)
        {
            _tools.AddRange(doc.Data);
        }
    }

    public void Save()
    {
        string dir = Path.GetDirectoryName(StateFilePath) ?? "";
        if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
        {
            Directory.CreateDirectory(dir);
        }

        JsonStore.Save(StateFilePath, _tools, schemaVersion: 1);
    }

    public InstalledToolRecord? GetTool(string toolId)
    {
        return _tools.FirstOrDefault(t => t.ToolId.Equals(toolId, StringComparison.OrdinalIgnoreCase) && t.IsActive);
    }

    public List<InstalledToolRecord> GetVersions(string toolId)
    {
        return _tools.Where(t => t.ToolId.Equals(toolId, StringComparison.OrdinalIgnoreCase)).ToList();
    }

    public void RegisterTool(InstalledToolRecord record)
    {
        // Deactivate other versions if this is active
        if (record.IsActive)
        {
            foreach (var t in _tools.Where(t => t.ToolId.Equals(record.ToolId, StringComparison.OrdinalIgnoreCase)))
            {
                t.IsActive = false;
            }
        }

        var existing = _tools.FirstOrDefault(t => t.ToolId.Equals(record.ToolId, StringComparison.OrdinalIgnoreCase) &&
                                                  t.Version.Equals(record.Version, StringComparison.OrdinalIgnoreCase));
        if (existing != null)
        {
            int idx = _tools.IndexOf(existing);
            _tools[idx] = record;
        }
        else
        {
            _tools.Add(record);
        }

        Save();
        AppLogger.Log($"Registered tool state: {record.ToolId} v{record.Version} (Adopted: {record.IsAdopted})");
    }

    public bool UnregisterTool(string toolId, string version)
    {
        int removed = _tools.RemoveAll(t => t.ToolId.Equals(toolId, StringComparison.OrdinalIgnoreCase) &&
                                            t.Version.Equals(version, StringComparison.OrdinalIgnoreCase));
        if (removed > 0)
        {
            Save();
            AppLogger.Log($"Unregistered tool: {toolId} v{version}");
            return true;
        }
        return false;
    }
}
