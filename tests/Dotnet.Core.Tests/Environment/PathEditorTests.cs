using dotnet.Env;

namespace Dotnet.Core.Tests.Environment;

public class PathEditorTests
{
    [Fact]
    public void Parse_SplitsAndCleansEntries()
    {
        string path = @"C:\Windows; C:\Windows\System32; ; C:\tools\php85\ ;";
        var entries = PathEditor.Parse(path);

        Assert.Equal(3, entries.Count);
        Assert.Equal(@"C:\Windows", entries[0]);
        Assert.Equal(@"C:\Windows\System32", entries[1]);
        Assert.Equal(@"C:\tools\php85\", entries[2]);
    }

    [Fact]
    public void NormalizeEntry_StripsTrailingSlash()
    {
        Assert.Equal(@"C:\tools\php85", PathEditor.NormalizeEntry(@"C:/tools/php85/"));
        Assert.Equal(@"C:\", PathEditor.NormalizeEntry(@"C:\"));
    }

    [Fact]
    public void Prepend_PlacesEntryAtStart_Deduplicates()
    {
        var list = new List<string> { @"C:\Windows", @"C:\tools\php" };
        var updated = PathEditor.Prepend(list, @"C:\tools\php\");

        Assert.Equal(2, updated.Count);
        Assert.Equal(@"C:\tools\php", updated[0]);
        Assert.Equal(@"C:\Windows", updated[1]);
    }

    [Fact]
    public void Remove_RemovesMatchingEntryCaseInsensitive()
    {
        var list = new List<string> { @"C:\Windows", @"C:\tools\php" };
        var updated = PathEditor.Remove(list, @"c:\TOOLS\PHP\");

        Assert.Single(updated);
        Assert.Equal(@"C:\Windows", updated[0]);
    }
}
