using System.Text;
using System.Text.RegularExpressions;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class NginxSiteGeneratorResult
{
    public bool Success { get; set; }
    public string ConfigPath { get; set; } = string.Empty;
    public string Message { get; set; } = string.Empty;
}

public static class NginxSiteGenerator
{
    private static readonly string SitesDirectory = AppPaths.GetPath(Path.Combine("nginx", "sites"));

    public static string GetSitesDirectory()
    {
        if (!Directory.Exists(SitesDirectory))
        {
            Directory.CreateDirectory(SitesDirectory);
        }
        return SitesDirectory;
    }

    public static string BuildSiteConfig(ProjectInfo project, int fastCgiPort = 9000)
    {
        string host = string.IsNullOrWhiteSpace(project.NginxHost) ? "localhost" : project.NginxHost.Trim();
        string publicDir = string.IsNullOrWhiteSpace(project.PublicDirectory) ? "." : project.PublicDirectory.Trim();
        string fullRoot = Path.Combine(project.Path, publicDir).Replace('\\', '/');

        var sb = new StringBuilder();

        if (project.Framework.Equals("nextjs", StringComparison.OrdinalIgnoreCase) ||
            project.Framework.Equals("vite", StringComparison.OrdinalIgnoreCase) ||
            project.Framework.Equals("node", StringComparison.OrdinalIgnoreCase))
        {
            int port = project.DevPort ?? (project.Framework.Equals("vite", StringComparison.OrdinalIgnoreCase) ? 5173 : 3000);
            sb.AppendLine("server {");
            sb.AppendLine("    listen 80;");
            sb.AppendLine($"    server_name {host};");
            sb.AppendLine();
            sb.AppendLine("    location / {");
            sb.AppendLine($"        proxy_pass http://127.0.0.1:{port};");
            sb.AppendLine("        proxy_http_version 1.1;");
            sb.AppendLine("        proxy_set_header Upgrade $http_upgrade;");
            sb.AppendLine("        proxy_set_header Connection \"upgrade\";");
            sb.AppendLine("        proxy_set_header Host $host;");
            sb.AppendLine("        proxy_cache_bypass $http_upgrade;");
            sb.AppendLine("    }");
            sb.AppendLine("}");
        }
        else if (project.Framework.Equals("static", StringComparison.OrdinalIgnoreCase))
        {
            sb.AppendLine("server {");
            sb.AppendLine("    listen 80;");
            sb.AppendLine($"    server_name {host};");
            sb.AppendLine($"    root \"{fullRoot}\";");
            sb.AppendLine("    index index.html index.htm;");
            sb.AppendLine();
            sb.AppendLine("    location / {");
            sb.AppendLine("        try_files $uri $uri/ =404;");
            sb.AppendLine("    }");
            sb.AppendLine("}");
        }
        else
        {
            // Default: PHP / Laravel
            sb.AppendLine("server {");
            sb.AppendLine("    listen 80;");
            sb.AppendLine($"    server_name {host};");
            sb.AppendLine($"    root \"{fullRoot}\";");
            sb.AppendLine("    index index.php index.html index.htm;");
            sb.AppendLine();
            sb.AppendLine("    location / {");
            sb.AppendLine("        try_files $uri $uri/ /index.php?$query_string;");
            sb.AppendLine("    }");
            sb.AppendLine();
            sb.AppendLine("    location ~ \\.php$ {");
            sb.AppendLine($"        fastcgi_pass 127.0.0.1:{fastCgiPort};");
            sb.AppendLine("        fastcgi_index index.php;");
            sb.AppendLine("        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;");
            sb.AppendLine("        include fastcgi_params;");
            sb.AppendLine("    }");
            sb.AppendLine("}");
        }

        return sb.ToString();
    }

    public static async Task<NginxSiteGeneratorResult> CreateOrUpdateSiteAsync(ProjectInfo project, bool registerHost = true)
    {
        if (string.IsNullOrWhiteSpace(project.NginxHost))
        {
            return new NginxSiteGeneratorResult { Success = false, Message = "Host domain is empty." };
        }

        if (!HostsFileManager.IsValidHostname(project.NginxHost, out string hostError))
        {
            return new NginxSiteGeneratorResult { Success = false, Message = $"Invalid hostname: {hostError}" };
        }

        // Prevent hijack of known public domains
        var blockedDomains = new[] { "google.com", "github.com", "facebook.com", "microsoft.com", "apple.com" };
        if (blockedDomains.Any(d => project.NginxHost.Equals(d, StringComparison.OrdinalIgnoreCase) || project.NginxHost.EndsWith("." + d, StringComparison.OrdinalIgnoreCase)))
        {
            return new NginxSiteGeneratorResult { Success = false, Message = "Target domain is a restricted public website." };
        }

        string sitesDir = GetSitesDirectory();
        string safeFileName = Regex.Replace(project.Name.ToLowerInvariant(), @"[^a-z0-9\-]", "-").Trim('-') + ".conf";
        string targetConfPath = Path.Combine(sitesDir, safeFileName);

        // Ensure main nginx.conf includes this directory
        EnsureNginxIncludesSitesDir();

        string content = BuildSiteConfig(project);
        string? oldBackup = File.Exists(targetConfPath) ? File.ReadAllText(targetConfPath) : null;

        try
        {
            await File.WriteAllTextAsync(targetConfPath, content, Encoding.UTF8);

            // Validate configuration before keeping
            var (valid, errorOutput) = await NginxManager.ValidateConfigAsync();
            if (!valid)
            {
                // Rollback
                if (oldBackup != null)
                {
                    await File.WriteAllTextAsync(targetConfPath, oldBackup, Encoding.UTF8);
                }
                else
                {
                    File.Delete(targetConfPath);
                }
                AppLogger.Log($"Nginx validation failed for site '{project.NginxHost}': {errorOutput}");
                return new NginxSiteGeneratorResult
                {
                    Success = false,
                    Message = $"Nginx configuration test failed:\n{errorOutput}"
                };
            }

            // Register in Windows hosts file if requested
            if (registerHost)
            {
                HostsFileManager.AddOrUpdateEntry("127.0.0.1", project.NginxHost);
            }

            // Reload Nginx
            await NginxManager.ReloadNginxAsync();

            AppLogger.Log($"Successfully generated and activated Nginx site for {project.NginxHost} at {targetConfPath}");
            return new NginxSiteGeneratorResult
            {
                Success = true,
                ConfigPath = targetConfPath,
                Message = $"Site {project.NginxHost} is active and routed."
            };
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error creating Nginx site: {ex.Message}");
            return new NginxSiteGeneratorResult { Success = false, Message = ex.Message };
        }
    }

    public static async Task<bool> RemoveSiteAsync(ProjectInfo project)
    {
        try
        {
            string sitesDir = GetSitesDirectory();
            string safeFileName = Regex.Replace(project.Name.ToLowerInvariant(), @"[^a-z0-9\-]", "-").Trim('-') + ".conf";
            string targetConfPath = Path.Combine(sitesDir, safeFileName);

            if (File.Exists(targetConfPath))
            {
                File.Delete(targetConfPath);
            }

            if (!string.IsNullOrWhiteSpace(project.NginxHost))
            {
                HostsFileManager.RemoveEntry(project.NginxHost);
            }

            await NginxManager.ReloadNginxAsync();
            AppLogger.Log($"Removed Nginx site and hosts entry for {project.Name}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error removing Nginx site: {ex.Message}");
            return false;
        }
    }

    public static void EnsureNginxIncludesSitesDir()
    {
        string nginxDir = NginxManager.GetNginxDirectory();
        string nginxConf = Path.Combine(nginxDir, "conf", "nginx.conf");

        if (!File.Exists(nginxConf))
        {
            return;
        }

        try
        {
            string content = File.ReadAllText(nginxConf);
            string forwardSitesPattern = GetSitesDirectory().Replace('\\', '/') + "/*.conf";
            string includeDirective = $"include \"{forwardSitesPattern}\";";

            if (content.Contains(includeDirective, StringComparison.OrdinalIgnoreCase) ||
                content.Contains("nginx/sites/*.conf", StringComparison.OrdinalIgnoreCase) ||
                content.Contains("conf/sites/*.conf", StringComparison.OrdinalIgnoreCase))
            {
                return; // Already included
            }

            // Insert before last closing bracket of http block
            int lastBrace = content.LastIndexOf('}');
            if (lastBrace > 0)
            {
                string updated = content.Substring(0, lastBrace) +
                                 $"\n    # >>> Dotnet Managed Sites >>>\n    {includeDirective}\n    # <<< Dotnet Managed Sites <<<\n" +
                                 content.Substring(lastBrace);
                File.WriteAllText(nginxConf, updated, Encoding.UTF8);
                AppLogger.Log($"Added managed sites include directive to {nginxConf}");
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to inject sites include into nginx.conf: {ex.Message}");
        }
    }
}
