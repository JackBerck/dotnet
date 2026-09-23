using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class FrameworkDetectorTests : IDisposable
{
    private readonly string _tempDir;

    public FrameworkDetectorTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "dotnet_fw_test_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_tempDir))
            {
                Directory.Delete(_tempDir, true);
            }
        }
        catch { }
    }

    [Fact]
    public void Detect_LaravelProject_ReturnsLaravelFramework()
    {
        string composerJson = @"{
            ""name"": ""laravel/laravel"",
            ""require"": {
                ""php"": ""^8.2"",
                ""laravel/framework"": ""^11.0""
            }
        }";
        File.WriteAllText(Path.Combine(_tempDir, "composer.json"), composerJson);

        var result = FrameworkDetector.Detect(_tempDir);

        Assert.Equal("laravel", result.Framework);
        Assert.Equal("php artisan serve", result.DevCommand);
        Assert.Equal("public", result.PublicDirectory);
        Assert.EndsWith(".test", result.SuggestedHost);
    }

    [Fact]
    public void Detect_NextJsProject_ReturnsNextJsFramework()
    {
        string packageJson = @"{
            ""name"": ""my-next-app"",
            ""dependencies"": {
                ""next"": ""14.2.0"",
                ""react"": ""^18""
            }
        }";
        File.WriteAllText(Path.Combine(_tempDir, "package.json"), packageJson);

        var result = FrameworkDetector.Detect(_tempDir);

        Assert.Equal("nextjs", result.Framework);
        Assert.Equal("npm run dev", result.DevCommand);
        Assert.Equal(3000, result.SuggestedPort);
    }

    [Fact]
    public void Detect_ViteProject_ReturnsViteFramework()
    {
        string packageJson = @"{
            ""name"": ""vite-project"",
            ""devDependencies"": {
                ""vite"": ""^5.0.0""
            }
        }";
        File.WriteAllText(Path.Combine(_tempDir, "package.json"), packageJson);

        var result = FrameworkDetector.Detect(_tempDir);

        Assert.Equal("vite", result.Framework);
        Assert.Equal("npm run dev", result.DevCommand);
        Assert.Equal(5173, result.SuggestedPort);
    }

    [Fact]
    public void Detect_StaticProject_ReturnsStaticFramework()
    {
        File.WriteAllText(Path.Combine(_tempDir, "index.html"), "<h1>Hello</h1>");

        var result = FrameworkDetector.Detect(_tempDir);

        Assert.Equal("static", result.Framework);
        Assert.Equal(".", result.PublicDirectory);
    }
}
