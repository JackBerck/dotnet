namespace dotnet.Models;

public class ProjectInfo
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string Name { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    public string Framework { get; set; } = "custom"; // laravel, nextjs, vite, node, php, static, custom
    public string PhpVersion { get; set; } = "8.5";
    public string? NodeVersion { get; set; }
    public string DatabaseType { get; set; } = "MySQL";
    public string DatabaseName { get; set; } = string.Empty;
    public bool RequiresRedis { get; set; } = false;
    public string DevCommand { get; set; } = "composer run dev";
    public int? DevPort { get; set; }
    public string NginxHost { get; set; } = string.Empty;
    public string NginxSiteMode { get; set; } = "auto"; // auto, manual, disabled
    public string PublicDirectory { get; set; } = "public";
    public string Notes { get; set; } = string.Empty;
}
