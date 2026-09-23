namespace dotnet.Models;

public enum ConfigTargetType
{
    Ini,
    CliConfig,
    Json,
    TemplateFile
}

public class ConfigField
{
    public string Key { get; set; } = string.Empty;
    public string Label { get; set; } = string.Empty;
    public string Type { get; set; } = "string"; // size, int, string, boolean, timezone, extensionList
    public string DefaultValue { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public string ValidationPattern { get; set; } = string.Empty;
    public List<string> Options { get; set; } = new();
}

public class ConfigGroup
{
    public string Title { get; set; } = string.Empty;
    public List<ConfigField> Fields { get; set; } = new();
}

public class ConfigValidator
{
    public string Type { get; set; } = "command";
    public string Executable { get; set; } = string.Empty;
    public List<string> Arguments { get; set; } = new();
    public string FailOnStderrMatch { get; set; } = string.Empty;
}

public class ConfigSchema
{
    public string Id { get; set; } = string.Empty;
    public string Title { get; set; } = string.Empty;
    public ConfigTargetType TargetType { get; set; } = ConfigTargetType.Ini;
    public string TargetFile { get; set; } = string.Empty;
    public List<ConfigGroup> Groups { get; set; } = new();
    public List<ConfigValidator> Validators { get; set; } = new();
    public List<string> Presets { get; set; } = new();
}
