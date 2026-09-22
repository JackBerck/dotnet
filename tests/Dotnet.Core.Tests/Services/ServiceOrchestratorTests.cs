using dotnet.Models;
using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class ServiceOrchestratorTests
{
    [Fact]
    public void FilterProfileServices_ShouldReturnMatchingServicesOnly()
    {
        var allServices = new List<DevServiceInfo>
        {
            new() { Id = "mysql", Name = "MySQL" },
            new() { Id = "postgres", Name = "PostgreSQL" },
            new() { Id = "nginx", Name = "Nginx" },
            new() { Id = "php-cgi", Name = "PHP CGI" }
        };

        var profile = new ServiceProfile
        {
            Id = "docker",
            Name = "Docker Coexistence",
            ServiceIds = new() { "nginx", "php-cgi" }
        };

        var filtered = ServiceOrchestrator.FilterProfileServices(allServices, profile);

        Assert.Equal(2, filtered.Count);
        Assert.Contains(filtered, s => s.Id == "nginx");
        Assert.Contains(filtered, s => s.Id == "php-cgi");
        Assert.DoesNotContain(filtered, s => s.Id == "mysql");
    }

    [Fact]
    public async Task StartProfileServicesAsync_ShouldStartDatabasesBeforeWebServers()
    {
        var allServices = new List<DevServiceInfo>
        {
            new() { Id = "nginx", Name = "Nginx" },
            new() { Id = "mysql", Name = "MySQL" },
            new() { Id = "php-cgi", Name = "PHP CGI" },
            new() { Id = "redis", Name = "Redis" }
        };

        var profile = new ServiceProfile
        {
            Id = "test",
            ServiceIds = new() { "nginx", "mysql", "php-cgi", "redis" }
        };

        var startOrder = new List<string>();

        await ServiceOrchestrator.StartProfileServicesAsync(allServices, profile, svc =>
        {
            startOrder.Add(svc.Id);
            return Task.FromResult(true);
        });

        Assert.Equal(4, startOrder.Count);
        // mysql (1) -> redis (3) -> php-cgi (4) -> nginx (5)
        Assert.Equal("mysql", startOrder[0]);
        Assert.Equal("redis", startOrder[1]);
        Assert.Equal("php-cgi", startOrder[2]);
        Assert.Equal("nginx", startOrder[3]);
    }

    [Fact]
    public async Task StopProfileServicesAsync_ShouldStopWebServersBeforeDatabases()
    {
        var allServices = new List<DevServiceInfo>
        {
            new() { Id = "nginx", Name = "Nginx" },
            new() { Id = "mysql", Name = "MySQL" },
            new() { Id = "php-cgi", Name = "PHP CGI" },
            new() { Id = "redis", Name = "Redis" }
        };

        var profile = new ServiceProfile
        {
            Id = "test",
            ServiceIds = new() { "nginx", "mysql", "php-cgi", "redis" }
        };

        var stopOrder = new List<string>();

        await ServiceOrchestrator.StopProfileServicesAsync(allServices, profile, svc =>
        {
            stopOrder.Add(svc.Id);
            return Task.FromResult(true);
        });

        Assert.Equal(4, stopOrder.Count);
        // Reverse: nginx (5) -> php-cgi (4) -> redis (3) -> mysql (1)
        Assert.Equal("nginx", stopOrder[0]);
        Assert.Equal("php-cgi", stopOrder[1]);
        Assert.Equal("redis", stopOrder[2]);
        Assert.Equal("mysql", stopOrder[3]);
    }
}
