namespace dotnet.Catalog;

public class ToolDefinition
{
    public int SchemaVersion { get; set; } = 1;
    public string Id { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public string Category { get; set; } = "general";
    public string Homepage { get; set; } = string.Empty;
    public string License { get; set; } = string.Empty;
    public List<string> Platforms { get; set; } = new() { "win-x64" };
    public List<string> Prerequisites { get; set; } = new();
    public List<string> AllowedHosts { get; set; } = new();
    public List<ToolVersion> Versions { get; set; } = new();
    public ToolLayout Layout { get; set; } = new();
    public List<PostInstallOp> PostInstall { get; set; } = new();
    public VerifyStep? Verify { get; set; }
}

public class ToolVersion
{
    public string Version { get; set; } = string.Empty;
    public string Variant { get; set; } = string.Empty;
    public ToolSource Source { get; set; } = new();
}

public class ToolSource
{
    public string Type { get; set; } = "zip"; // zip | single-file | adopt
    public string Url { get; set; } = string.Empty;
    public string Sha256 { get; set; } = string.Empty;
    public bool StripRoot { get; set; } = false;
    public string FileName { get; set; } = string.Empty;
}

public class ToolLayout
{
    public List<string> BinDirs { get; set; } = new() { "." };
    public Dictionary<string, string> EnvVars { get; set; } = new();
}

public class PostInstallOp
{
    public string Op { get; set; } = string.Empty; // copyIfMissing | iniSet | iniEnableExtensions | writeShim | ensureSharedFile
    public string? From { get; set; }
    public string? To { get; set; }
    public string? File { get; set; }
    public string? Key { get; set; }
    public string? Value { get; set; }
    public List<string>? Names { get; set; }
    public string? Name { get; set; }
    public string? Content { get; set; }
    public string? Id { get; set; }
}

public class VerifyStep
{
    public string Command { get; set; } = string.Empty;
    public List<string> Args { get; set; } = new();
    public string? ExpectRegex { get; set; }
}
