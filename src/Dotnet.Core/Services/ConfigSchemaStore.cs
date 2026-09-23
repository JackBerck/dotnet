using dotnet.Models;

namespace dotnet.Services;

public static class ConfigSchemaStore
{
    public static ConfigSchema GetPhpSchema(string phpDir = @"C:\tools\php85")
    {
        return new ConfigSchema
        {
            Id = "php.ini",
            Title = "PHP Configuration (php.ini)",
            TargetType = ConfigTargetType.Ini,
            TargetFile = Path.Combine(phpDir, "php.ini"),
            Presets = new() { "development", "production" },
            Groups = new()
            {
                new()
                {
                    Title = "Resource Limits",
                    Fields = new()
                    {
                        new() { Key = "memory_limit", Label = "Memory Limit", Type = "size", DefaultValue = "512M", Description = "Maximum amount of memory a script may consume." },
                        new() { Key = "upload_max_filesize", Label = "Max Upload Size", Type = "size", DefaultValue = "64M", Description = "Maximum allowed size for uploaded files." },
                        new() { Key = "post_max_size", Label = "Max POST Size", Type = "size", DefaultValue = "64M", Description = "Maximum size of POST data that PHP will accept." },
                        new() { Key = "max_execution_time", Label = "Max Execution Time (s)", Type = "int", DefaultValue = "300", Description = "Maximum execution time of each script, in seconds." }
                    }
                },
                new()
                {
                    Title = "Date & Regional Settings",
                    Fields = new()
                    {
                        new() { Key = "date.timezone", Label = "Timezone", Type = "timezone", DefaultValue = "UTC", Description = "Default timezone used by date/time functions." },
                        new() { Key = "default_charset", Label = "Default Charset", Type = "string", DefaultValue = "UTF-8", Description = "Default character encoding." }
                    }
                },
                new()
                {
                    Title = "Error Reporting & Debugging",
                    Fields = new()
                    {
                        new() { Key = "display_errors", Label = "Display Errors", Type = "boolean", DefaultValue = "On", Description = "Whether to display errors to browser." },
                        new() { Key = "display_startup_errors", Label = "Display Startup Errors", Type = "boolean", DefaultValue = "On", Description = "Whether to display startup errors." },
                        new() { Key = "error_reporting", Label = "Error Reporting", Type = "string", DefaultValue = "E_ALL", Description = "Level of error reporting." }
                    }
                }
            }
        };
    }

    public static ConfigSchema GetGitSchema()
    {
        return new ConfigSchema
        {
            Id = "git",
            Title = "Git Global Configuration",
            TargetType = ConfigTargetType.CliConfig,
            Groups = new()
            {
                new()
                {
                    Title = "User Identity",
                    Fields = new()
                    {
                        new() { Key = "user.name", Label = "Author Name", Type = "string", Description = "Name to show on commits." },
                        new() { Key = "user.email", Label = "Author Email", Type = "string", Description = "Email to show on commits." }
                    }
                },
                new()
                {
                    Title = "Repository Preferences",
                    Fields = new()
                    {
                        new() { Key = "init.defaultBranch", Label = "Default Branch", Type = "string", DefaultValue = "main", Description = "Default branch name for new repos." },
                        new() { Key = "core.autocrlf", Label = "Auto CRLF", Type = "string", DefaultValue = "true", Description = "Newline conversion (true/false/input)." },
                        new() { Key = "pull.rebase", Label = "Pull Rebase", Type = "string", DefaultValue = "false", Description = "Whether git pull rebases by default." }
                    }
                }
            }
        };
    }
}
