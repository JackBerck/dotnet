using dotnet.Models;
using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class NginxSiteGeneratorTests
{
    [Fact]
    public void BuildSiteConfig_LaravelProject_GeneratesFastCgiConfig()
    {
        var project = new ProjectInfo
        {
            Name = "laravel-app",
            Path = @"C:\dev\laravel-app",
            Framework = "laravel",
            NginxHost = "laravel-app.test",
            PublicDirectory = "public"
        };

        string config = NginxSiteGenerator.BuildSiteConfig(project, 9000);

        Assert.Contains("server_name laravel-app.test;", config);
        Assert.Contains("root \"C:/dev/laravel-app/public\";", config);
        Assert.Contains("try_files $uri $uri/ /index.php?$query_string;", config);
        Assert.Contains("fastcgi_pass 127.0.0.1:9000;", config);
    }

    [Fact]
    public void BuildSiteConfig_ViteProject_GeneratesReverseProxyConfig()
    {
        var project = new ProjectInfo
        {
            Name = "vite-app",
            Path = @"C:\dev\vite-app",
            Framework = "vite",
            NginxHost = "vite-app.test",
            DevPort = 5173
        };

        string config = NginxSiteGenerator.BuildSiteConfig(project);

        Assert.Contains("server_name vite-app.test;", config);
        Assert.Contains("proxy_pass http://127.0.0.1:5173;", config);
        Assert.Contains("proxy_set_header Upgrade $http_upgrade;", config);
    }

    [Fact]
    public void BuildSiteConfig_StaticProject_GeneratesStaticHtmlConfig()
    {
        var project = new ProjectInfo
        {
            Name = "static-site",
            Path = @"C:\dev\static-site",
            Framework = "static",
            NginxHost = "static-site.test",
            PublicDirectory = "."
        };

        string config = NginxSiteGenerator.BuildSiteConfig(project);

        Assert.Contains("server_name static-site.test;", config);
        Assert.Contains("try_files $uri $uri/ =404;", config);
    }
}
