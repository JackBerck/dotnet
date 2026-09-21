using System;
using System.IO;

namespace dotnet.Persistence;

public static class AppPaths
{
    public static string DataDirectory { get; }

    static AppPaths()
    {
        DataDirectory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Dotnet");
        if (!Directory.Exists(DataDirectory))
        {
            Directory.CreateDirectory(DataDirectory);
        }
    }

    public static string GetPath(string fileName)
    {
        return Path.Combine(DataDirectory, fileName);
    }
}
