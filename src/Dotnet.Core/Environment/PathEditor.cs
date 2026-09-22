namespace dotnet.Env;

public static class PathEditor
{
    public static List<string> Parse(string pathValue)
    {
        if (string.IsNullOrWhiteSpace(pathValue))
        {
            return new List<string>();
        }

        return pathValue
            .Split(';', StringSplitOptions.RemoveEmptyEntries)
            .Select(p => p.Trim())
            .Where(p => !string.IsNullOrWhiteSpace(p))
            .ToList();
    }

    public static string NormalizeEntry(string entry)
    {
        if (string.IsNullOrWhiteSpace(entry)) return string.Empty;
        string normalized = entry.Trim().Replace('/', '\\');
        // Strip trailing slash unless it is root drive like C:\
        if (normalized.Length > 3 && normalized.EndsWith('\\'))
        {
            normalized = normalized.TrimEnd('\\');
        }
        return normalized;
    }

    public static bool ContainsEntry(IEnumerable<string> entries, string entryToFind)
    {
        string target = NormalizeEntry(entryToFind);
        return entries.Any(e => string.Equals(NormalizeEntry(e), target, StringComparison.OrdinalIgnoreCase));
    }

    public static List<string> Prepend(List<string> entries, string newEntry)
    {
        var result = new List<string>(entries);
        string normalized = NormalizeEntry(newEntry);
        if (string.IsNullOrWhiteSpace(normalized)) return result;

        // Remove if already exists so it moves to front
        result.RemoveAll(e => string.Equals(NormalizeEntry(e), normalized, StringComparison.OrdinalIgnoreCase));
        result.Insert(0, normalized);
        return result;
    }

    public static List<string> Append(List<string> entries, string newEntry)
    {
        var result = new List<string>(entries);
        string normalized = NormalizeEntry(newEntry);
        if (string.IsNullOrWhiteSpace(normalized)) return result;

        if (!ContainsEntry(result, normalized))
        {
            result.Add(normalized);
        }
        return result;
    }

    public static List<string> Remove(List<string> entries, string entryToRemove)
    {
        var result = new List<string>(entries);
        string normalized = NormalizeEntry(entryToRemove);
        if (string.IsNullOrWhiteSpace(normalized)) return result;

        result.RemoveAll(e => string.Equals(NormalizeEntry(e), normalized, StringComparison.OrdinalIgnoreCase));
        return result;
    }

    public static string Join(IEnumerable<string> entries)
    {
        return string.Join(";", entries);
    }
}
