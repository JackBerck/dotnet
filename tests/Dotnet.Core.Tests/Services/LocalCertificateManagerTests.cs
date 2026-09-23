using System.Security.Cryptography.X509Certificates;
using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class LocalCertificateManagerTests
{
    [Fact]
    public void GetOrCreateRootCa_GeneratesValidRootCa()
    {
        using var ca = LocalCertificateManager.GetOrCreateRootCa();
        Assert.NotNull(ca);
        Assert.Contains("Dotnet Development CA", ca.Subject);
        Assert.True(ca.HasPrivateKey);

        // Check Basic Constraints
        var basicConstraint = ca.Extensions.OfType<X509BasicConstraintsExtension>().FirstOrDefault();
        Assert.NotNull(basicConstraint);
        Assert.True(basicConstraint.CertificateAuthority);
    }

    [Fact]
    public void CreateOrGetCertificate_GeneratesValidLeafCertificate()
    {
        string domain = $"testsite-{Guid.NewGuid():N}.test";
        var info = LocalCertificateManager.CreateOrGetCertificate(domain);

        Assert.Equal(domain, info.Domain);
        Assert.True(File.Exists(info.CertPath));
        Assert.True(File.Exists(info.KeyPath));

        using var cert = X509Certificate2.CreateFromPem(File.ReadAllText(info.CertPath));
        Assert.Contains(domain, cert.Subject);

        // Check SAN extension
        var sanExt = cert.Extensions.OfType<X509SubjectAlternativeNameExtension>().FirstOrDefault();
        Assert.NotNull(sanExt);
        Assert.True(cert.NotAfter > DateTime.UtcNow);
    }

    [Fact]
    public void BuildSiteConfig_WithSslEnabled_GeneratesSslDirectives()
    {
        var proj = new dotnet.Models.ProjectInfo
        {
            Name = "laravel-app",
            Path = @"C:\projects\laravel-app",
            NginxHost = "laravel-app.test",
            Framework = "laravel",
            PublicDirectory = "public"
        };

        string certPath = @"C:\ssl\laravel-app.test.crt";
        string keyPath = @"C:\ssl\laravel-app.test.key";

        string config = NginxSiteGenerator.BuildSiteConfig(proj, enableSsl: true, certPath: certPath, keyPath: keyPath);

        Assert.Contains("listen 80;", config);
        Assert.Contains("return 301 https://$host$request_uri;", config);
        Assert.Contains("listen 443 ssl;", config);
        Assert.Contains("ssl_certificate \"C:/ssl/laravel-app.test.crt\";", config);
        Assert.Contains("ssl_certificate_key \"C:/ssl/laravel-app.test.key\";", config);
        Assert.Contains("ssl_protocols TLSv1.2 TLSv1.3;", config);
    }
}
