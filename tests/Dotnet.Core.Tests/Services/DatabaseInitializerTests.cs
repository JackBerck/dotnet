using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class DatabaseInitializerTests
{
    [Fact]
    public void GetDataDirectory_ReturnsPathUnderLocalData()
    {
        string dirMysql = DatabaseInitializer.GetDataDirectory("mysql");
        string dirPostgres = DatabaseInitializer.GetDataDirectory("postgres");

        Assert.Contains("data", dirMysql);
        Assert.Contains("mysql", dirMysql);
        Assert.Contains("data", dirPostgres);
        Assert.Contains("postgres", dirPostgres);
    }

    [Fact]
    public void BuildRuntimeArguments_ForMysql_ContainsExpectedFlags()
    {
        var args = DatabaseInitializer.BuildRuntimeArguments("mysql", 3306);

        Assert.Contains(args, a => a.StartsWith("--datadir="));
        Assert.Contains("--port=3306", args);
        Assert.Contains("--console", args);
    }

    [Fact]
    public void BuildRuntimeArguments_ForPostgres_ContainsExpectedFlags()
    {
        var args = DatabaseInitializer.BuildRuntimeArguments("postgres", 5432);

        Assert.Contains("-D", args);
        Assert.Contains("-p", args);
        Assert.Contains("5432", args);
    }

    [Fact]
    public void VcRedistChecker_OfficialUrlIsAkaMs()
    {
        Assert.StartsWith("https://aka.ms/", VcRedistChecker.OfficialDownloadUrl);
    }
}
