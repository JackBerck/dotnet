namespace dotnet.Models;

public class ProjectInfo
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string Name { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    public string PhpVersion { get; set; } = "8.5";
    public string DatabaseType { get; set; } = "MySQL";
    public string DatabaseName { get; set; } = string.Empty;
    public bool RequiresRedis { get; set; } = false;
    public string DevCommand { get; set; } = "composer run dev";
    public string NginxHost { get; set; } = string.Empty;
}
