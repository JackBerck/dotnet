using dotnet.Config;

namespace Dotnet.Core.Tests.Config;

public class IniDocumentTests
{
    [Fact]
    public void Parse_ReadsKeyValuePairsCorrectly()
    {
        string ini = @"
[PHP]
memory_limit = 512M
upload_max_filesize = 64M
;max_execution_time = 30
";
        var doc = IniDocument.Parse(ini);

        Assert.Equal("512M", doc.Get("memory_limit", "PHP"));
        Assert.Equal("64M", doc.Get("upload_max_filesize"));
        Assert.Null(doc.Get("max_execution_time")); // commented out
        Assert.Null(doc.Get("non_existent"));
    }

    [Fact]
    public void Set_UpdatesExistingValue_PreservesComments()
    {
        string ini = @"
[PHP]
; Default memory limit
memory_limit = 128M ; inline comment
upload_max_filesize = 10M
";
        var doc = IniDocument.Parse(ini);
        doc.Set("memory_limit", "1024M", "PHP");

        Assert.Equal("1024M", doc.Get("memory_limit", "PHP"));
        string content = doc.ToContent();
        Assert.Contains("1024M ; inline comment", content);
        Assert.Contains("; Default memory limit", content);
    }

    [Fact]
    public void Set_UncommentsCommentedOutKey()
    {
        string ini = @"
[PHP]
;extension=curl
;extension=mbstring
";
        var doc = IniDocument.Parse(ini);
        Assert.Null(doc.Get("extension"));

        doc.Set("extension", "curl", "PHP");
        Assert.Equal("curl", doc.Get("extension", "PHP"));
        Assert.Contains("extension = curl", doc.ToContent());
    }

    [Fact]
    public void Set_AppendsNewKeyIfNotFound()
    {
        string ini = @"
[PHP]
memory_limit = 256M
";
        var doc = IniDocument.Parse(ini);
        doc.Set("max_execution_time", "300", "PHP");

        Assert.Equal("300", doc.Get("max_execution_time", "PHP"));
        string content = doc.ToContent();
        Assert.Contains("max_execution_time = 300", content);
    }
}
