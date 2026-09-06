namespace dotnet.Models;

public class HostEntryInfo
{
    public string IpAddress { get; set; } = "127.0.0.1";
    public string Hostname { get; set; } = string.Empty;
    public string Comment { get; set; } = string.Empty;
    public bool IsEnabled { get; set; } = true;
    public int LineNumber { get; set; }
    public string RawLine { get; set; } = string.Empty;
}
