using dotnet.Models;
using dotnet.Services;

namespace Dotnet.Core.Tests.Services;

public class HostsFileManagerTests
{
    [Fact]
    public void ParseContent_IdentifiesManagedAndUnmanagedEntries()
    {
        string hosts = @"
# Copyright (c) Microsoft Corp.
# 127.0.0.1 localhost
127.0.0.1 systemhost.local

# >>> Dotnet (managed) >>>
127.0.0.1 myapp.test
# 127.0.0.1 disabled.test
# <<< Dotnet (managed) <<<
";
        var entries = HostsFileManager.ParseContent(hosts);

        var systemHost = entries.FirstOrDefault(e => e.Hostname == "systemhost.local");
        Assert.NotNull(systemHost);
        Assert.False(systemHost.IsManaged);
        Assert.True(systemHost.IsEnabled);

        var myApp = entries.FirstOrDefault(e => e.Hostname == "myapp.test");
        Assert.NotNull(myApp);
        Assert.True(myApp.IsManaged);
        Assert.True(myApp.IsEnabled);

        var disabled = entries.FirstOrDefault(e => e.Hostname == "disabled.test");
        Assert.NotNull(disabled);
        Assert.True(disabled.IsManaged);
        Assert.False(disabled.IsEnabled);
    }

    [Fact]
    public void BuildContent_PreservesUnmanagedContent_UpdatesManagedBlock()
    {
        string original = @"# System default
127.0.0.1 localhost

# >>> Dotnet (managed) >>>
127.0.0.1 old.test
# <<< Dotnet (managed) <<<

# End comment";

        var managed = new List<HostEntryInfo>
        {
            new HostEntryInfo { IpAddress = "127.0.0.1", Hostname = "newsite.test", IsEnabled = true, IsManaged = true },
            new HostEntryInfo { IpAddress = "127.0.0.1", Hostname = "stopped.test", IsEnabled = false, IsManaged = true }
        };

        string updated = HostsFileManager.BuildContent(original, managed);

        Assert.Contains("# System default", updated);
        Assert.Contains("127.0.0.1 localhost", updated);
        Assert.Contains("# End comment", updated);
        Assert.DoesNotContain("old.test", updated);
        Assert.Contains("127.0.0.1\tnewsite.test", updated);
        Assert.Contains("# 127.0.0.1\tstopped.test", updated);
    }

    [Theory]
    [InlineData("myproject.test", true)]
    [InlineData("api.v1.local-dev.localhost", true)]
    [InlineData("invalid_hostname!@", false)]
    [InlineData("", false)]
    public void IsValidHostname_ValidatesCorrectly(string hostname, bool expectedValid)
    {
        bool isValid = HostsFileManager.IsValidHostname(hostname, out _);
        Assert.Equal(expectedValid, isValid);
    }
}
