namespace dotnet.Models;

public enum DevServiceType
{
    WindowsService,
    ManagedProcess
}

public class DevServiceInfo
{
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public DevServiceType Type { get; set; }
    public string WindowsServiceName { get; set; } = string.Empty;
    public string ExecutablePath { get; set; } = string.Empty;
    public string Arguments { get; set; } = string.Empty;
    public string WorkingDirectory { get; set; } = string.Empty;
    public int Port { get; set; }
    public ServiceStatus Status { get; set; } = ServiceStatus.Unknown;
    public int? ProcessId { get; set; }
    public string StatusMessage { get; set; } = string.Empty;
    public bool AutoStartWithGroup { get; set; } = true;
    public bool AutoStartOnBoot { get; set; } = true;
}
