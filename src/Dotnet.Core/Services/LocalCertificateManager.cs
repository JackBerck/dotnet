using System.Net;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using dotnet.Config;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class CertificateInfo
{
    public string Domain { get; set; } = string.Empty;
    public string CertPath { get; set; } = string.Empty;
    public string KeyPath { get; set; } = string.Empty;
    public DateTimeOffset NotAfter { get; set; }
}

public static class LocalCertificateManager
{
    private static readonly string SslDir = AppPaths.GetPath("ssl");
    private static readonly string RootCaCertPath = Path.Combine(SslDir, "rootCA.crt");
    private static readonly string RootCaKeyPath = Path.Combine(SslDir, "rootCA.key");

    public static string GetSslDirectory() => SslDir;
    public static string GetRootCaCertPath() => RootCaCertPath;

    public static X509Certificate2 GetOrCreateRootCa()
    {
        Directory.CreateDirectory(SslDir);

        if (File.Exists(RootCaCertPath) && File.Exists(RootCaKeyPath))
        {
            try
            {
                string certPem = File.ReadAllText(RootCaCertPath);
                string keyPem = File.ReadAllText(RootCaKeyPath);
                return X509Certificate2.CreateFromPem(certPem, keyPem);
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Error loading existing Root CA: {ex.Message}. Regenerating.");
            }
        }

        using var rsa = RSA.Create(2048);
        var req = new CertificateRequest(
            "CN=Dotnet Development CA, O=Dotnet Local Dev, OU=Development",
            rsa,
            HashAlgorithmName.SHA256,
            RSASignaturePadding.Pkcs1);

        req.CertificateExtensions.Add(
            new X509BasicConstraintsExtension(certificateAuthority: true, hasPathLengthConstraint: false, pathLengthConstraint: 0, critical: true));
        req.CertificateExtensions.Add(
            new X509KeyUsageExtension(X509KeyUsageFlags.KeyCertSign | X509KeyUsageFlags.CrlSign, critical: true));

        var cert = req.CreateSelfSigned(
            DateTimeOffset.UtcNow.AddDays(-1),
            DateTimeOffset.UtcNow.AddYears(10));

        File.WriteAllText(RootCaCertPath, cert.ExportCertificatePem());
        File.WriteAllText(RootCaKeyPath, rsa.ExportPkcs8PrivateKeyPem());

        AppLogger.Log($"Generated new Dotnet Root CA: {RootCaCertPath}");
        return X509Certificate2.CreateFromPem(File.ReadAllText(RootCaCertPath), File.ReadAllText(RootCaKeyPath));
    }

    public static CertificateInfo CreateOrGetCertificate(string domain)
    {
        Directory.CreateDirectory(SslDir);
        string certPath = Path.Combine(SslDir, $"{domain}.crt");
        string keyPath = Path.Combine(SslDir, $"{domain}.key");

        if (File.Exists(certPath) && File.Exists(keyPath))
        {
            try
            {
                var existing = X509Certificate2.CreateFromPem(File.ReadAllText(certPath));
                if (existing.NotAfter > DateTime.UtcNow.AddDays(7))
                {
                    return new CertificateInfo
                    {
                        Domain = domain,
                        CertPath = certPath,
                        KeyPath = keyPath,
                        NotAfter = existing.NotAfter
                    };
                }
            }
            catch { }
        }

        using var caCert = GetOrCreateRootCa();
        using var rsa = RSA.Create(2048);

        var req = new CertificateRequest(
            $"CN={domain}",
            rsa,
            HashAlgorithmName.SHA256,
            RSASignaturePadding.Pkcs1);

        req.CertificateExtensions.Add(
            new X509BasicConstraintsExtension(certificateAuthority: false, hasPathLengthConstraint: false, pathLengthConstraint: 0, critical: false));
        req.CertificateExtensions.Add(
            new X509KeyUsageExtension(X509KeyUsageFlags.DigitalSignature | X509KeyUsageFlags.KeyEncipherment, critical: true));
        req.CertificateExtensions.Add(
            new X509EnhancedKeyUsageExtension(new OidCollection { new Oid("1.3.6.1.5.5.7.3.1") }, critical: false));

        var sanBuilder = new SubjectAlternativeNameBuilder();
        sanBuilder.AddDnsName(domain);
        if (!domain.StartsWith("*."))
        {
            sanBuilder.AddDnsName($"*.{domain}");
        }
        sanBuilder.AddIpAddress(IPAddress.Loopback);
        req.CertificateExtensions.Add(sanBuilder.Build());

        byte[] serialNumber = new byte[16];
        RandomNumberGenerator.Fill(serialNumber);

        var notBefore = DateTimeOffset.UtcNow.AddDays(-1);
        var notAfter = DateTimeOffset.UtcNow.AddYears(2);

        using var leafCert = req.Create(caCert, notBefore, notAfter, serialNumber);
        var certWithKey = leafCert.CopyWithPrivateKey(rsa);

        File.WriteAllText(certPath, certWithKey.ExportCertificatePem());
        File.WriteAllText(keyPath, rsa.ExportPkcs8PrivateKeyPem());

        AppLogger.Log($"Generated SSL certificate for {domain} at {certPath}");

        return new CertificateInfo
        {
            Domain = domain,
            CertPath = certPath,
            KeyPath = keyPath,
            NotAfter = notAfter
        };
    }

    public static bool IsRootCaTrusted()
    {
        if (!File.Exists(RootCaCertPath)) return false;

        try
        {
            using var ca = X509Certificate2.CreateFromPem(File.ReadAllText(RootCaCertPath));
            string thumbprint = ca.Thumbprint;

            using var store = new X509Store(StoreName.Root, StoreLocation.CurrentUser);
            store.Open(OpenFlags.ReadOnly);
            var matches = store.Certificates.Find(X509FindType.FindByThumbprint, thumbprint, false);
            if (matches.Count > 0) return true;

            using var lmStore = new X509Store(StoreName.Root, StoreLocation.LocalMachine);
            lmStore.Open(OpenFlags.ReadOnly);
            var lmMatches = lmStore.Certificates.Find(X509FindType.FindByThumbprint, thumbprint, false);
            return lmMatches.Count > 0;
        }
        catch
        {
            return false;
        }
    }

    public static OperationResult TrustRootCertificate()
    {
        try
        {
            var ca = GetOrCreateRootCa();
            if (IsRootCaTrusted())
            {
                return new OperationResult(true, "Dotnet Root CA is already trusted.");
            }

            using var store = new X509Store(StoreName.Root, StoreLocation.CurrentUser);
            store.Open(OpenFlags.ReadWrite);
            store.Add(ca);
            store.Close();

            AppLogger.Log("Installed Dotnet Root CA into CurrentUser Root store.");
            return new OperationResult(true, "Root CA successfully installed to CurrentUser Certificate Store.");
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to trust Root CA: {ex.Message}");
            return new OperationResult(false, $"Failed to install Root CA: {ex.Message}", ex);
        }
    }
}
