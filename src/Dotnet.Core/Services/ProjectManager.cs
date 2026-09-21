using System.Diagnostics;
using System.Text.Json;
using dotnet.Models;

namespace dotnet.Services;

public class ProjectManager
{
    private static readonly string ProjectsFilePath = dotnet.Persistence.AppPaths.GetPath("projects.json");
    private static List<ProjectInfo> Projects = new();

    static ProjectManager()
    {
        LoadProjects();
    }

    public static List<ProjectInfo> GetProjects() => Projects;

    public static void LoadProjects()
    {
        try
        {
            if (File.Exists(ProjectsFilePath))
            {
                string json = File.ReadAllText(ProjectsFilePath);
                Projects = JsonSerializer.Deserialize<List<ProjectInfo>>(json) ?? new List<ProjectInfo>();
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error loading projects file: {ex.Message}");
            Projects = new List<ProjectInfo>();
        }
    }

    public static void SaveProjects()
    {
        try
        {
            string json = JsonSerializer.Serialize(Projects, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(ProjectsFilePath, json);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Error saving projects file: {ex.Message}");
        }
    }

    public static void AddOrUpdateProject(ProjectInfo project)
    {
        var existing = Projects.FirstOrDefault(p => p.Id == project.Id);
        if (existing != null)
        {
            int index = Projects.IndexOf(existing);
            Projects[index] = project;
        }
        else
        {
            Projects.Add(project);
        }

        SaveProjects();
        AppLogger.Log($"Saved project: {project.Name} ({project.Path})");
    }

    public static void DeleteProject(string id)
    {
        Projects.RemoveAll(p => p.Id == id);
        SaveProjects();
        AppLogger.Log($"Deleted project ID: {id}");
    }

    public static void LaunchDevCommand(ProjectInfo project)
    {
        try
        {
            if (!Directory.Exists(project.Path))
            {
                AppLogger.Log($"Directory not found: {project.Path}");
                return;
            }

            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = $"/k cd /d \"{project.Path}\" && {project.DevCommand}",
                UseShellExecute = true,
                WorkingDirectory = project.Path
            };

            Process.Start(psi);
            AppLogger.Log($"Launched command '{project.DevCommand}' for project {project.Name}");
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to launch dev command for {project.Name}: {ex.Message}");
        }
    }

    public static void OpenInExplorer(string path)
    {
        try
        {
            if (Directory.Exists(path))
            {
                Process.Start("explorer.exe", path);
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to open directory: {ex.Message}");
        }
    }

    public static void OpenTerminal(string path)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                WorkingDirectory = path,
                UseShellExecute = true
            };
            Process.Start(psi);
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to open terminal: {ex.Message}");
        }
    }

    public static void OpenEnvFile(string path)
    {
        try
        {
            string envPath = Path.Combine(path, ".env");
            if (File.Exists(envPath))
            {
                Process.Start("notepad.exe", envPath);
            }
            else
            {
                AppLogger.Log($".env file not found at {envPath}");
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to open .env file: {ex.Message}");
        }
    }

    public static void OpenBrowser(string host)
    {
        try
        {
            if (!string.IsNullOrWhiteSpace(host))
            {
                string url = host.StartsWith("http") ? host : $"http://{host}";
                Process.Start(new ProcessStartInfo
                {
                    FileName = url,
                    UseShellExecute = true
                });
            }
        }
        catch (Exception ex)
        {
            AppLogger.Log($"Failed to open browser for {host}: {ex.Message}");
        }
    }
}
