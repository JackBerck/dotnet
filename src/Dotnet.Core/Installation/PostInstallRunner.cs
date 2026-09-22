using dotnet.Catalog;
using dotnet.Config;
using dotnet.Services;

namespace dotnet.Installation;

public class PostInstallRunner
{
    public static void Run(ToolDefinition tool, string installDir)
    {
        if (tool.PostInstall == null || tool.PostInstall.Count == 0) return;

        foreach (var op in tool.PostInstall)
        {
            try
            {
                switch (op.Op.ToLowerInvariant())
                {
                    case "copyifmissing":
                        if (!string.IsNullOrEmpty(op.From) && !string.IsNullOrEmpty(op.To))
                        {
                            string source = Path.Combine(installDir, op.From);
                            string dest = Path.Combine(installDir, op.To);
                            if (File.Exists(source) && !File.Exists(dest))
                            {
                                File.Copy(source, dest);
                                AppLogger.Log($"Post-install: Copied {op.From} to {op.To}");
                            }
                        }
                        break;

                    case "iniset":
                        if (!string.IsNullOrEmpty(op.File) && !string.IsNullOrEmpty(op.Key) && op.Value != null)
                        {
                            string iniPath = Path.Combine(installDir, op.File);
                            if (File.Exists(iniPath))
                            {
                                var doc = IniDocument.Load(iniPath);
                                doc.Set(op.Key, op.Value);
                                doc.Save(iniPath);
                                AppLogger.Log($"Post-install: Set INI {op.Key} = {op.Value}");
                            }
                        }
                        break;

                    case "inienableextensions":
                        if (!string.IsNullOrEmpty(op.File) && op.Names != null && op.Names.Count > 0)
                        {
                            string iniPath = Path.Combine(installDir, op.File);
                            if (File.Exists(iniPath))
                            {
                                var doc = IniDocument.Load(iniPath);
                                foreach (var extName in op.Names)
                                {
                                    doc.Set("extension", extName);
                                }
                                doc.Save(iniPath);
                                AppLogger.Log($"Post-install: Enabled {op.Names.Count} extensions in {op.File}");
                            }
                        }
                        break;

                    case "writeshim":
                        if (!string.IsNullOrEmpty(op.Name) && !string.IsNullOrEmpty(op.Content))
                        {
                            string shimPath = Path.Combine(installDir, op.Name);
                            File.WriteAllText(shimPath, op.Content);
                            AppLogger.Log($"Post-install: Written shim {op.Name}");
                        }
                        break;
                }
            }
            catch (Exception ex)
            {
                AppLogger.Log($"Error executing post-install step '{op.Op}': {ex.Message}");
            }
        }
    }
}
