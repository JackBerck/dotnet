using System.Diagnostics;
using dotnet.Models;
using dotnet.Persistence;

namespace dotnet.Services;

public class ProjectManager
{
    private static readonly string ProjectsFilePath = AppPaths.GetPath(Path.Combine("state", "projects.json"));
    private static readonly string LegacyProjectsFilePath = AppPaths.GetPath("projects.json");
    private static List<ProjectInfo> _projects = new();

    static ProjectManager()
    {
        LoadProjects();
    }

    public static List<ProjectInfo> GetProjects() => _projects;

    public static void LoadProjects()
    {
        if (!File.Exists(ProjectsFilePath) && File.Exists(LegacyProjectsFilePath))
        {
            try
            {
                var legacyDoc = JsonStore.Load<List<ProjectInfo>>(LegacyProjectsFilePath);
                if (legacyDoc?.Data != null)
                {
                    _projects = legacyDoc.Data;
                    SaveProjects();
                    AppLogger.Log("Migrated legacy projects.json to state/ directory.");
                    return;
                }
            }
            catch { }
        }

        var doc = JsonStore.Load<List<ProjectInfo>>(ProjectsFilePath);
        _projects = doc?.Data ?? new List<ProjectInfo>();
    }

    public static void SaveProjects()
    {
        JsonStore.Save(ProjectsFilePath, _projects, schemaVersion: 2);
    }

    public static void AddOrUpdateProject(ProjectInfo project)
    {
        var existing = _projects.FirstOrDefault(p => p.Id == project.Id);
        if (existing != null)
        {
            int index = _projects.IndexOf(existing);
            _projects[index] = project;
        }
        else
        {
            _projects.Add(project);
        }

        SaveProjects();
        AppLogger.Log($"Saved project: {project.Name} ({project.Path})");
    }

    public static void DeleteProject(string id)
    {
        _projects.RemoveAll(p => p.Id == id);
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

            if (string.IsNullOrWhiteSpace(project.DevCommand))
            {
                AppLogger.Log($"No dev command configured for {project.Name}.");
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
            if (!Directory.Exists(path)) return;

            string wtPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), @"Microsoft\WindowsApps\wt.exe");
            if (File.Exists(wtPath))
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = wtPath,
                    Arguments = $"-d \"{path}\"",
                    UseShellExecute = true
                });
                return;
            }

            var psi = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-NoExit -Command \"Set-Location '{path}'\"",
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

    public static void OpenInVSCode(string path)
    {
        try
        {
            if (!Directory.Exists(path)) return;

            var psi = new ProcessStartInfo
            {
                FileName = "code.cmd",
                Arguments = $".",
                WorkingDirectory = path,
                UseShellExecute = true,
                CreateNoWindow = true
            };
            Process.Start(psi);
            AppLogger.Log($"Opened VS Code at {path}");
        }
        catch
        {
            // Fallback to code.exe
            try
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = "code.exe",
                    Arguments = $".",
                    WorkingDirectory = path,
                    UseShellExecute = true
                });
            }
            catch (Exception ex)
            {
                AppLogger.Log($"VS Code not found in PATH: {ex.Message}");
            }
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
                string url = host.StartsWith("http", StringComparison.OrdinalIgnoreCase) ? host : $"http://{host}";
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
