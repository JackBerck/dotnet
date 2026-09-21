namespace dotnet.Models;

public enum ServiceStatus
{
    NotInstalled,
    Stopped,
    Starting,
    Running,
    Stopping,
    PortConflict,
    Error,
    Unknown
}
