using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class ComposePortCheckerTests
{
    [Fact]
    public void ExpandPortRange_SinglePort_ReturnsSingleElement()
    {
        var result = DockerPortChecker.ExpandPortRange("3306");
        Assert.Single(result);
        Assert.Equal(3306, result[0]);
    }

    [Fact]
    public void ExpandPortRange_RangePort_ReturnsAllNumbersInRange()
    {
        var result = DockerPortChecker.ExpandPortRange("8080-8083");
        Assert.Equal(4, result.Count);
        Assert.Equal(new[] { 8080, 8081, 8082, 8083 }, result);
    }

    [Fact]
    public void ParsePortString_ComplexSpecifications_ParsesCorrectly()
    {
        var p1 = DockerPortChecker.ParsePortString("127.0.0.1:5432:5432/tcp");
        Assert.Single(p1);
        Assert.Equal(5432, p1[0].hostPort);
        Assert.Equal(5432, p1[0].contPort);
        Assert.Equal("127.0.0.1", p1[0].ip);
        Assert.Equal("tcp", p1[0].proto);

        var p2 = DockerPortChecker.ParsePortString("8080:80");
        Assert.Single(p2);
        Assert.Equal(8080, p2[0].hostPort);
        Assert.Equal(80, p2[0].contPort);

        var p3 = DockerPortChecker.ParsePortString("9000-9002:9000-9002/udp");
        Assert.Equal(3, p3.Count);
        Assert.Equal(9000, p3[0].hostPort);
        Assert.Equal("udp", p3[0].proto);
    }

    [Fact]
    public void ParseComposeJson_ValidServices_ExtractsAllPorts()
    {
        string sampleJson = """
        {
          "services": {
            "web": {
              "ports": [
                {
                  "mode": "ingress",
                  "target": 80,
                  "published": "8080",
                  "protocol": "tcp"
                }
              ]
            },
            "db": {
              "ports": [
                "3306:3306"
              ]
            }
          }
        }
        """;

        var conflicts = DockerPortChecker.ParseComposeJson(sampleJson, new HashSet<int>());
        Assert.Equal(2, conflicts.Count);
        Assert.Contains(conflicts, c => c.ServiceName == "web" && c.HostPort == 8080 && c.ContainerPort == 80);
        Assert.Contains(conflicts, c => c.ServiceName == "db" && c.HostPort == 3306 && c.ContainerPort == 3306);
    }

    [Fact]
    public void ParseComposeFallback_ValidYamlFile_ParsesPortsCorrectly()
    {
        string tempFile = Path.Combine(Path.GetTempPath(), $"docker-compose-{Guid.NewGuid():N}.yml");
        try
        {
            string yaml = """
            services:
              api:
                ports:
                  - "5000:5000"
                  - "5001:5001/udp"
              cache:
                ports:
                  - 6379:6379
            """;
            File.WriteAllText(tempFile, yaml);

            var conflicts = DockerPortChecker.ParseComposeFallback(tempFile);
            Assert.Equal(3, conflicts.Count);
            Assert.Contains(conflicts, c => c.ServiceName == "api" && c.HostPort == 5000);
            Assert.Contains(conflicts, c => c.ServiceName == "api" && c.HostPort == 5001 && c.Protocol == "udp");
            Assert.Contains(conflicts, c => c.ServiceName == "cache" && c.HostPort == 6379);
        }
        finally
        {
            if (File.Exists(tempFile)) File.Delete(tempFile);
        }
    }
}
