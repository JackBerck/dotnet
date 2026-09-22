using dotnet.Services;
using Microsoft.Win32;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class PathShadowAnalyzerTests
{
    [Fact]
    public void Analyze_ShouldExecuteWithoutExceptions()
    {
        var results = PathShadowAnalyzer.Analyze();
        Assert.NotNull(results);
        // May be empty or non-empty depending on system, but should be valid objects
        foreach (var res in results)
        {
            Assert.False(string.IsNullOrWhiteSpace(res.BinaryName));
            Assert.NotNull(res.Recommendation);
        }
    }

    [Fact]
    public void GetPathDirectories_WithValidRegistryKey_ReturnsDistinctList()
    {
        var dirs = PathShadowAnalyzer.GetPathDirectories(Registry.CurrentUser, "Environment");
        Assert.NotNull(dirs);
        // Should not contain duplicate entries
        Assert.Equal(dirs.Count, dirs.Distinct(StringComparer.OrdinalIgnoreCase).Count());
    }
}
