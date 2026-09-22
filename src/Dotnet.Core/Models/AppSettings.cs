namespace dotnet.Models;

public enum ExitActionPreference
{
    Prompt,
    StopServices,
    KeepRunning
}

public class AppSettings
{
    public ExitActionPreference ExitPreference { get; set; } = ExitActionPreference.Prompt;
    public bool MinimizeToTrayOnClose { get; set; } = true;
    public bool StartMinimized { get; set; } = false;
}
