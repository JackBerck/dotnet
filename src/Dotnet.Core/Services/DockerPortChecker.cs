using System.Text.RegularExpressions;

namespace dotnet.Services;

public class DockerPortConflict
{
    public int HostPort { get; set; }
    public int ContainerPort { get; set; }
    public string ServiceName { get; set; } = string.Empty;
    public bool IsConflicting { get; set; }
    public string ConflictOwner { get; set; } = string.Empty;
}

public class DockerPortChecker
{
    public static List<DockerPortConflict> CheckComposeFile(string composeFilePath)
    {
        var results = new List<DockerPortConflict>();

        if (!File.Exists(composeFilePath))
        {
            AppLogger.Log($"Docker compose file not found: {composeFilePath}");
            return results;
        }

        try
        {
            string[] lines = File.ReadAllLines(composeFilePath);
            // Quick regex match for common compose port syntax: "3306:3306", "5432:5432", "8080:80", etc.
            var portRegex = new Regex(@"^\s*-\s*[""']?(\d+):(\d+)[""']?");

            foreach (string line in lines)
            {
                var match = portRegex.Match(line);
                if (match.Success)
                {
                    int hostPort = int.Parse(match.Groups[1].Value);
                    int containerPort = int.Parse(match.Groups[2].Value);

                    var conflictInfo = PortConflictDetector.CheckPort(hostPort);

                    results.Add(new DockerPortConflict
                    {
                        HostPort = hostPort,
                        ContainerPort = containerPort,
                        IsConflicting = conflictInfo.IsInUse,
                        ConflictOwner = conflictInfo.IsInUse ? $"{conflictInfo.ProcessName} (PID {conflictInfo.ProcessId})" : "None"
                    });
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error checking Docker compose file: {ex.Message}");
        }

        return results;
    }
}
