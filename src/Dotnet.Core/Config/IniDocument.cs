using System.Text;
using System.Text.RegularExpressions;

namespace dotnet.Config;

public class IniDocument
{
    private class IniLine
    {
        public enum LineKind { Blank, Comment, Section, KeyValue }
        public LineKind Kind { get; set; }
        public string? Section { get; set; }
        public string? Key { get; set; }
        public string? Value { get; set; }
        public string? InlineComment { get; set; }
        public string Raw { get; set; } = string.Empty;
        public bool IsCommentedOutKeyValue { get; set; }
        public string? CommentedKey { get; set; }
    }

    private readonly List<IniLine> _lines = new();

    public static IniDocument Load(string filePath)
    {
        if (!File.Exists(filePath))
        {
            return new IniDocument();
        }

        string content = File.ReadAllText(filePath);
        return Parse(content);
    }

    public static IniDocument Parse(string content)
    {
        var doc = new IniDocument();
        using var reader = new StringReader(content);
        string? currentSection = null;
        string? line;

        var sectionRegex = new Regex(@"^\s*\[([^\]]+)\]\s*(?:[;#].*)?$");
        var keyValueRegex = new Regex(@"^\s*([^=;#\s][^=]*?)\s*=\s*(.*?)\s*(?:([;#].*))?$");
        var commentedKeyValueRegex = new Regex(@"^\s*[;#]\s*([^=;#\s][^=]*?)\s*=\s*(.*?)\s*(?:([;#].*))?$");

        while ((line = reader.ReadLine()) != null)
        {
            if (string.IsNullOrWhiteSpace(line))
            {
                doc._lines.Add(new IniLine { Kind = IniLine.LineKind.Blank, Raw = line, Section = currentSection });
                continue;
            }

            var secMatch = sectionRegex.Match(line);
            if (secMatch.Success)
            {
                currentSection = secMatch.Groups[1].Value.Trim();
                doc._lines.Add(new IniLine { Kind = IniLine.LineKind.Section, Raw = line, Section = currentSection });
                continue;
            }

            var kvMatch = keyValueRegex.Match(line);
            if (kvMatch.Success)
            {
                string key = kvMatch.Groups[1].Value.Trim();
                string val = kvMatch.Groups[2].Value.Trim();
                string comment = kvMatch.Groups[3].Success ? kvMatch.Groups[3].Value : "";

                doc._lines.Add(new IniLine
                {
                    Kind = IniLine.LineKind.KeyValue,
                    Key = key,
                    Value = StripQuotes(val),
                    InlineComment = comment,
                    Raw = line,
                    Section = currentSection
                });
                continue;
            }

            var ckvMatch = commentedKeyValueRegex.Match(line);
            if (ckvMatch.Success)
            {
                string key = ckvMatch.Groups[1].Value.Trim();
                string val = ckvMatch.Groups[2].Value.Trim();
                string comment = ckvMatch.Groups[3].Success ? ckvMatch.Groups[3].Value : "";

                doc._lines.Add(new IniLine
                {
                    Kind = IniLine.LineKind.Comment,
                    IsCommentedOutKeyValue = true,
                    CommentedKey = key,
                    Value = StripQuotes(val),
                    InlineComment = comment,
                    Raw = line,
                    Section = currentSection
                });
                continue;
            }

            // Normal comment or arbitrary line
            doc._lines.Add(new IniLine { Kind = IniLine.LineKind.Comment, Raw = line, Section = currentSection });
        }

        return doc;
    }

    private static string StripQuotes(string value)
    {
        if (value.Length >= 2 &&
            ((value.StartsWith('"') && value.EndsWith('"')) ||
             (value.StartsWith('\'') && value.EndsWith('\''))))
        {
            return value.Substring(1, value.Length - 2);
        }
        return value;
    }

    public string? Get(string key, string? section = null)
    {
        var match = _lines
            .Where(l => l.Kind == IniLine.LineKind.KeyValue &&
                        IsSectionMatch(l.Section, section) &&
                        string.Equals(l.Key, key, StringComparison.OrdinalIgnoreCase))
            .LastOrDefault();

        return match?.Value;
    }

    public void Set(string key, string value, string? section = null)
    {
        // 1. Try to find active key
        var activeLine = _lines
            .Where(l => l.Kind == IniLine.LineKind.KeyValue &&
                        IsSectionMatch(l.Section, section) &&
                        string.Equals(l.Key, key, StringComparison.OrdinalIgnoreCase))
            .LastOrDefault();

        if (activeLine != null)
        {
            activeLine.Value = value;
            activeLine.Raw = FormatLine(key, value, activeLine.InlineComment);
            return;
        }

        // 2. Try to find commented-out key in same section
        var commentedLine = _lines
            .Where(l => l.Kind == IniLine.LineKind.Comment &&
                        l.IsCommentedOutKeyValue &&
                        IsSectionMatch(l.Section, section) &&
                        string.Equals(l.CommentedKey, key, StringComparison.OrdinalIgnoreCase))
            .LastOrDefault();

        if (commentedLine != null)
        {
            commentedLine.Kind = IniLine.LineKind.KeyValue;
            commentedLine.Key = key;
            commentedLine.Value = value;
            commentedLine.IsCommentedOutKeyValue = false;
            commentedLine.Raw = FormatLine(key, value, commentedLine.InlineComment);
            return;
        }

        // 3. Append to target section or end of file
        var newLine = new IniLine
        {
            Kind = IniLine.LineKind.KeyValue,
            Key = key,
            Value = value,
            Section = section,
            Raw = FormatLine(key, value, null)
        };

        if (string.IsNullOrEmpty(section))
        {
            // Add before first section header or at end
            int firstSecIdx = _lines.FindIndex(l => l.Kind == IniLine.LineKind.Section);
            if (firstSecIdx >= 0)
            {
                _lines.Insert(firstSecIdx, newLine);
            }
            else
            {
                _lines.Add(newLine);
            }
        }
        else
        {
            int secIdx = _lines.FindIndex(l => l.Kind == IniLine.LineKind.Section &&
                                               string.Equals(l.Section, section, StringComparison.OrdinalIgnoreCase));
            if (secIdx >= 0)
            {
                // Find end of section (next section or EOF)
                int nextSecIdx = _lines.FindIndex(secIdx + 1, l => l.Kind == IniLine.LineKind.Section);
                if (nextSecIdx >= 0)
                {
                    _lines.Insert(nextSecIdx, newLine);
                }
                else
                {
                    _lines.Add(newLine);
                }
            }
            else
            {
                // Create section header
                _lines.Add(new IniLine { Kind = IniLine.LineKind.Blank, Raw = "" });
                _lines.Add(new IniLine { Kind = IniLine.LineKind.Section, Section = section, Raw = $"[{section}]" });
                _lines.Add(newLine);
            }
        }
    }

    private static bool IsSectionMatch(string? currentSection, string? targetSection)
    {
        if (string.IsNullOrEmpty(targetSection)) return true;
        return string.Equals(currentSection, targetSection, StringComparison.OrdinalIgnoreCase);
    }

    private static string FormatLine(string key, string value, string? inlineComment)
    {
        if (!string.IsNullOrWhiteSpace(inlineComment))
        {
            return $"{key} = {value} {inlineComment.Trim()}";
        }
        return $"{key} = {value}";
    }

    public string ToContent()
    {
        var sb = new StringBuilder();
        foreach (var line in _lines)
        {
            sb.AppendLine(line.Raw);
        }
        return sb.ToString();
    }

    public void Save(string filePath)
    {
        string dir = Path.GetDirectoryName(filePath) ?? "";
        if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
        {
            Directory.CreateDirectory(dir);
        }

        string tempPath = filePath + ".tmp." + Guid.NewGuid().ToString("N");
        File.WriteAllText(tempPath, ToContent(), Encoding.UTF8);

        if (File.Exists(filePath))
        {
            File.Move(tempPath, filePath, overwrite: true);
        }
        else
        {
            File.Move(tempPath, filePath);
        }
    }
}
