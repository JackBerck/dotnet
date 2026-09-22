using System.Text;
using System.Text.Json;
using dotnet.Services;

namespace dotnet.Persistence;

public class VersionedDocument<T>
{
    public int SchemaVersion { get; set; } = 1;
    public T Data { get; set; } = default!;
}

public static class JsonStore
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNameCaseInsensitive = true
    };

    public static VersionedDocument<T>? Load<T>(string filePath, int expectedSchemaVersion = 1)
    {
        if (!File.Exists(filePath)) return null;

        try
        {
            string json = File.ReadAllText(filePath);
            if (string.IsNullOrWhiteSpace(json)) return null;

            // 1. Try deserializing as versioned wrapper
            try
            {
                var versioned = JsonSerializer.Deserialize<VersionedDocument<T>>(json, JsonOptions);
                if (versioned != null && versioned.Data != null)
                {
                    return versioned;
                }
            }
            catch
            {
                // Fallback to legacy direct deserialization
            }

            // 2. Legacy fallback: raw T directly
            var directData = JsonSerializer.Deserialize<T>(json, JsonOptions);
            if (directData != null)
            {
                return new VersionedDocument<T>
                {
                    SchemaVersion = 0, // Legacy unversioned
                    Data = directData
                };
            }

            return null;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error loading JSON store '{filePath}': {ex.Message}");
            return null;
        }
    }

    public static bool Save<T>(string filePath, T data, int schemaVersion = 1)
    {
        try
        {
            string dir = Path.GetDirectoryName(filePath) ?? "";
            if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            var doc = new VersionedDocument<T>
            {
                SchemaVersion = schemaVersion,
                Data = data
            };

            string json = JsonSerializer.Serialize(doc, JsonOptions);
            string tempFile = filePath + ".tmp." + Guid.NewGuid().ToString("N");
            File.WriteAllText(tempFile, json, Encoding.UTF8);

            if (File.Exists(filePath))
            {
                File.Move(tempFile, filePath, overwrite: true);
            }
            else
            {
                File.Move(tempFile, filePath);
            }

            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error saving JSON store '{filePath}': {ex.Message}");
            return false;
        }
    }
}
