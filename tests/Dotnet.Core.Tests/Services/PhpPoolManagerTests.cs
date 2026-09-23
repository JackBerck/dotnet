using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class PhpPoolManagerTests
{
    [Fact]
    public void GenerateUpstreamBlock_WithTwoWorkers_GeneratesBothPorts()
    {
        string block = PhpPoolManager.GenerateUpstreamBlock(2, 9000);

        Assert.Contains("upstream php_pool {", block);
        Assert.Contains("server 127.0.0.1:9000;", block);
        Assert.Contains("server 127.0.0.1:9001;", block);
        Assert.DoesNotContain("server 127.0.0.1:9002;", block);
    }

    [Fact]
    public void GenerateUpstreamBlock_WithFourWorkers_GeneratesAllFourPorts()
    {
        string block = PhpPoolManager.GenerateUpstreamBlock(4, 9000);

        Assert.Contains("server 127.0.0.1:9000;", block);
        Assert.Contains("server 127.0.0.1:9001;", block);
        Assert.Contains("server 127.0.0.1:9002;", block);
        Assert.Contains("server 127.0.0.1:9003;", block);
    }
}
