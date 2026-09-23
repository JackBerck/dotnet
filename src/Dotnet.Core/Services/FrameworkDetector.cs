using System.Text.RegularExpressions;

namespace dotnet.Services;

public class FrameworkDetectionResult
{
    public string Framework { get; set; } = "custom";
    public string SuggestedName { get; set; } = string.Empty;
    public string DevCommand { get; set; } = string.Empty;
    public int? SuggestedPort { get; set; }
    public string PublicDirectory { get; set; } = "public";
    public string SuggestedHost { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
}

public static class FrameworkDetector
{
    public static FrameworkDetectionResult Detect(string directoryPath)
    {
        string dirName = Path.GetFileName(directoryPath.TrimEnd('\\', '/'));
        if (string.IsNullOrEmpty(dirName))
        {
            dirName = "project";
        }

        string safeHost = Regex.Replace(dirName.ToLowerInvariant(), @"[^a-z0-9\-]", "-").Trim('-') + ".test";
        if (safeHost == ".test") safeHost = "myproject.test";

        var result = new FrameworkDetectionResult
        {
            SuggestedName = dirName,
            SuggestedHost = safeHost,
            Framework = "custom",
            PublicDirectory = ".",
            Description = "Custom Project"
        };

        if (!Directory.Exists(directoryPath))
        {
            return result;
        }

        string composerFile = Path.Combine(directoryPath, "composer.json");
        string packageFile = Path.Combine(directoryPath, "package.json");
        string indexPhp = Path.Combine(directoryPath, "index.php");
        string indexHtml = Path.Combine(directoryPath, "index.html");

        if (File.Exists(composerFile))
        {
            try
            {
                string text = File.ReadAllText(composerFile);
                if (text.Contains("\"laravel/framework\"", StringComparison.OrdinalIgnoreCase))
                {
                    result.Framework = "laravel";
                    result.DevCommand = "php artisan serve";
                    result.SuggestedPort = 8000;
                    result.PublicDirectory = "public";
                    result.Description = "Laravel Web Application";
                    return result;
                }
                else
                {
                    result.Framework = "php";
                    result.DevCommand = "php -S 127.0.0.1:8000";
                    result.SuggestedPort = 8000;
                    result.PublicDirectory = Directory.Exists(Path.Combine(directoryPath, "public")) ? "public" : ".";
                    result.Description = "PHP Project (Composer)";
                    return result;
                }
            }
            catch { }
        }

        if (File.Exists(packageFile))
        {
            try
            {
                string text = File.ReadAllText(packageFile);
                if (text.Contains("\"next\"", StringComparison.OrdinalIgnoreCase))
                {
                    result.Framework = "nextjs";
                    result.DevCommand = "npm run dev";
                    result.SuggestedPort = 3000;
                    result.PublicDirectory = ".";
                    result.Description = "Next.js React Framework";
                    return result;
                }
                else if (text.Contains("\"vite\"", StringComparison.OrdinalIgnoreCase))
                {
                    result.Framework = "vite";
                    result.DevCommand = "npm run dev";
                    result.SuggestedPort = 5173;
                    result.PublicDirectory = ".";
                    result.Description = "Vite Frontend Framework";
                    return result;
                }
                else
                {
                    result.Framework = "node";
                    result.DevCommand = "npm run dev";
                    result.SuggestedPort = 3000;
                    result.PublicDirectory = ".";
                    result.Description = "Node.js Application";
                    return result;
                }
            }
            catch { }
        }

        if (File.Exists(indexPhp) || Directory.Exists(Path.Combine(directoryPath, "public")) && File.Exists(Path.Combine(directoryPath, "public", "index.php")))
        {
            result.Framework = "php";
            result.DevCommand = "php -S 127.0.0.1:8000";
            result.SuggestedPort = 8000;
            result.PublicDirectory = Directory.Exists(Path.Combine(directoryPath, "public")) ? "public" : ".";
            result.Description = "PHP Application";
            return result;
        }

        if (File.Exists(indexHtml))
        {
            result.Framework = "static";
            result.DevCommand = "";
            result.SuggestedPort = null;
            result.PublicDirectory = ".";
            result.Description = "Static Website";
            return result;
        }

        return result;
    }
}
