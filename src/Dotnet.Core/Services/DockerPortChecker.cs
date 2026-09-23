using System.Diagnostics;
using System.Net.Sockets;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace dotnet.Services;

public class DockerPortConflict
{
    public string ServiceName { get; set; } = string.Empty;
    public int HostPort { get; set; }
    public int ContainerPort { get; set; }
    public string Protocol { get; set; } = "tcp";
    public string HostIp { get; set; } = "0.0.0.0";
    public bool IsConflicting { get; set; }
    public string ConflictOwner { get; set; } = string.Empty;
    public bool IsSameProjectContainer { get; set; } = false;
    public int SuggestedOverridePort { get; set; }
}

public class DockerComposeReport
{
    public string ComposeFile { get; set; } = string.Empty;
    public bool IsDockerAvailable { get; set; } = true;
    public string Message { get; set; } = string.Empty;
    public List<DockerPortConflict> Conflicts { get; set; } = new();
    public int TotalPortsChecked => Conflicts.Count;
    public bool HasConflicts => Conflicts.Any(c => c.IsConflicting && !c.IsSameProjectContainer);
}

public class DockerPortChecker
{
    public static bool IsDockerInstalled()
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "where.exe",
                Arguments = "docker",
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };
            using var p = Process.Start(psi);
            if (p == null) return false;
            p.WaitForExit(2000);
            return p.ExitCode == 0;
        }
        catch
        {
            return false;
        }
    }

    public static async Task<DockerComposeReport> CheckComposeFileAsync(string composeFilePath, CancellationToken ct = default)
    {
        var report = new DockerComposeReport { ComposeFile = composeFilePath };

        if (!File.Exists(composeFilePath))
        {
            report.Message = $"Compose file not found: {composeFilePath}";
            return report;
        }

        bool dockerAvailable = IsDockerInstalled();
        report.IsDockerAvailable = dockerAvailable;

        if (!dockerAvailable)
        {
            report.Message = "Docker CLI not detected in PATH. Using YAML fallback parser.";
            report.Conflicts = ParseComposeFallback(composeFilePath);
            return report;
        }

        try
        {
            string workDir = Path.GetDirectoryName(composeFilePath) ?? "";
            string configJson = await RunCommandAsync("docker", $"compose -f \"{composeFilePath}\" config --format json", workDir, ct);

            if (string.IsNullOrWhiteSpace(configJson))
            {
                report.Message = "docker compose config returned empty. Using fallback parser.";
                report.Conflicts = ParseComposeFallback(composeFilePath);
                return report;
            }

            // Get running containers for this project to exclude self
            string psJson = await RunCommandAsync("docker", $"compose -f \"{composeFilePath}\" ps --format json", workDir, ct);
            var ownContainerPorts = ParseRunningProjectPorts(psJson);

            report.Conflicts = ParseComposeJson(configJson, ownContainerPorts);
            report.Message = report.HasConflicts ? "Port conflicts detected." : "All ports available.";
            return report;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error running docker compose config: {ex.Message}. Using fallback.");
            report.Message = $"Docker CLI failed: {ex.Message}. Evaluated with fallback parser.";
            report.Conflicts = ParseComposeFallback(composeFilePath);
            return report;
        }
    }

    public static List<DockerPortConflict> CheckComposeFile(string composeFilePath)
    {
        return CheckComposeFileAsync(composeFilePath).GetAwaiter().GetResult().Conflicts;
    }

    private static async Task<string> RunCommandAsync(string exe, string args, string workingDir, CancellationToken ct)
    {
        var psi = new ProcessStartInfo
        {
            FileName = exe,
            Arguments = args,
            WorkingDirectory = workingDir,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using var process = Process.Start(psi);
        if (process == null) return string.Empty;

        var stdoutTask = process.StandardOutput.ReadToEndAsync(ct);
        await process.WaitForExitAsync(ct);
        return process.ExitCode == 0 ? await stdoutTask : string.Empty;
    }

    public static List<DockerPortConflict> ParseComposeJson(string json, HashSet<int> ownPorts)
    {
        var results = new List<DockerPortConflict>();

        try
        {
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            if (!root.TryGetProperty("services", out var services) || services.ValueKind != JsonValueKind.Object)
            {
                return results;
            }

            foreach (var serviceProp in services.EnumerateObject())
            {
                string serviceName = serviceProp.Name;
                var serviceObj = serviceProp.Value;

                if (!serviceObj.TryGetProperty("ports", out var ports) || ports.ValueKind != JsonValueKind.Array)
                {
                    continue;
                }

                foreach (var portElem in ports.EnumerateArray())
                {
                    if (portElem.ValueKind == JsonValueKind.Object)
                    {
                        string hostIp = portElem.TryGetProperty("host_ip", out var ipVal) ? ipVal.GetString() ?? "0.0.0.0" : "0.0.0.0";
                        string proto = portElem.TryGetProperty("protocol", out var protoVal) ? protoVal.GetString() ?? "tcp" : "tcp";
                        int containerPort = portElem.TryGetProperty("target", out var targetVal) ? targetVal.GetInt32() : 0;

                        if (portElem.TryGetProperty("published", out var pubVal))
                        {
                            string pubStr = pubVal.ToString();
                            foreach (int port in ExpandPortRange(pubStr))
                            {
                                results.Add(EvaluatePortConflict(serviceName, port, containerPort, proto, hostIp, ownPorts));
                            }
                        }
                    }
                    else if (portElem.ValueKind == JsonValueKind.String)
                    {
                        var parsedPorts = ParsePortString(portElem.GetString() ?? "");
                        foreach (var (hostPort, contPort, proto, ip) in parsedPorts)
                        {
                            results.Add(EvaluatePortConflict(serviceName, hostPort, contPort, proto, ip, ownPorts));
                        }
                    }
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error parsing compose JSON: {ex.Message}");
        }

        return results;
    }

    public static List<DockerPortConflict> ParseComposeFallback(string composeFilePath)
    {
        var results = new List<DockerPortConflict>();
        if (!File.Exists(composeFilePath)) return results;

        try
        {
            string[] lines = File.ReadAllLines(composeFilePath);
            string currentService = "";
            bool inPortsSection = false;
            int portsIndent = 0;

            foreach (var rawLine in lines)
            {
                string line = rawLine;
                string trimmed = line.Trim();
                if (string.IsNullOrWhiteSpace(trimmed) || trimmed.StartsWith("#")) continue;

                int indent = line.TakeWhile(char.IsWhiteSpace).Count();

                // Detect service declaration: e.g. "  web:"
                if (indent == 2 && trimmed.EndsWith(":") && !trimmed.StartsWith("-"))
                {
                    currentService = trimmed.TrimEnd(':');
                    inPortsSection = false;
                    continue;
                }

                // Detect ports header: e.g. "    ports:"
                if (trimmed == "ports:")
                {
                    inPortsSection = true;
                    portsIndent = indent;
                    continue;
                }

                if (inPortsSection)
                {
                    if (indent <= portsIndent && !trimmed.StartsWith("-"))
                    {
                        inPortsSection = false;
                        continue;
                    }

                    if (trimmed.StartsWith("-"))
                    {
                        string portSpec = trimmed.TrimStart('-', ' ', '"', '\'').TrimEnd('"', '\'');
                        var parsedPorts = ParsePortString(portSpec);
                        foreach (var (hostPort, contPort, proto, ip) in parsedPorts)
                        {
                            results.Add(EvaluatePortConflict(currentService, hostPort, contPort, proto, ip, new HashSet<int>()));
                        }
                    }
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error in ParseComposeFallback: {ex.Message}");
        }

        return results;
    }

    private static HashSet<int> ParseRunningProjectPorts(string psJson)
    {
        var ports = new HashSet<int>();
        if (string.IsNullOrWhiteSpace(psJson)) return ports;

        try
        {
            // ps format json can be a single array or newline-delimited json objects
            var portRegex = new Regex(@":(\d+)->");
            foreach (Match match in portRegex.Matches(psJson))
            {
                if (int.TryParse(match.Groups[1].Value, out int p))
                {
                    ports.Add(p);
                }
            }
        }
        catch { }

        return ports;
    }

    public static List<(int hostPort, int contPort, string proto, string ip)> ParsePortString(string spec)
    {
        var list = new List<(int, int, string, string)>();
        if (string.IsNullOrWhiteSpace(spec)) return list;

        string proto = "tcp";
        if (spec.Contains("/"))
        {
            var parts = spec.Split('/');
            spec = parts[0];
            proto = parts[1].ToLowerInvariant();
        }

        var segments = spec.Split(':');
        string ip = "0.0.0.0";
        string hostPart = "";
        string contPart = "";

        if (segments.Length == 3)
        {
            ip = segments[0];
            hostPart = segments[1];
            contPart = segments[2];
        }
        else if (segments.Length == 2)
        {
            hostPart = segments[0];
            contPart = segments[1];
        }
        else if (segments.Length == 1)
        {
            hostPart = segments[0];
            contPart = segments[0];
        }

        var hostRange = ExpandPortRange(hostPart);
        var contRange = ExpandPortRange(contPart);

        for (int i = 0; i < hostRange.Count; i++)
        {
            int h = hostRange[i];
            int c = i < contRange.Count ? contRange[i] : (contRange.Count > 0 ? contRange[0] : h);
            list.Add((h, c, proto, ip));
        }

        return list;
    }

    public static List<int> ExpandPortRange(string rangeStr)
    {
        var result = new List<int>();
        if (string.IsNullOrWhiteSpace(rangeStr)) return result;

        if (rangeStr.Contains("-"))
        {
            var parts = rangeStr.Split('-');
            if (parts.Length == 2 && int.TryParse(parts[0], out int start) && int.TryParse(parts[1], out int end))
            {
                for (int p = start; p <= end; p++)
                {
                    result.Add(p);
                }
                return result;
            }
        }

        if (int.TryParse(rangeStr, out int single))
        {
            result.Add(single);
        }

        return result;
    }

    private static DockerPortConflict EvaluatePortConflict(string serviceName, int hostPort, int contPort, string proto, string hostIp, HashSet<int> ownPorts)
    {
        bool isOwn = ownPorts.Contains(hostPort);
        var conflict = PortConflictDetector.CheckPort(hostPort);
        bool inUse = conflict.IsInUse;

        int suggested = hostPort;
        if (inUse && !isOwn)
        {
            suggested = FindNextFreePort(hostPort);
        }

        return new DockerPortConflict
        {
            ServiceName = serviceName,
            HostPort = hostPort,
            ContainerPort = contPort,
            Protocol = proto,
            HostIp = hostIp,
            IsConflicting = inUse && !isOwn,
            ConflictOwner = inUse ? $"{conflict.ProcessName} (PID {conflict.ProcessId})" : "None",
            IsSameProjectContainer = isOwn,
            SuggestedOverridePort = suggested
        };
    }

    public static int FindNextFreePort(int startPort)
    {
        for (int p = startPort + 1; p < startPort + 1000 && p <= 65535; p++)
        {
            if (!PortConflictDetector.CheckPort(p).IsInUse)
            {
                return p;
            }
        }
        return startPort + 1000;
    }
}
