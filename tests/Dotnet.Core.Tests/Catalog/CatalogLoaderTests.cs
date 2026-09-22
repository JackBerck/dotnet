using dotnet.Catalog;

namespace Dotnet.Core.Tests.Catalog;

public class CatalogLoaderTests
{
    [Fact]
    public void LoadCatalog_LoadsAllEmbeddedTools()
    {
        var loader = new CatalogLoader();
        loader.LoadCatalog();

        Assert.NotEmpty(loader.Tools);
        Assert.NotNull(loader.GetTool("php"));
        Assert.NotNull(loader.GetTool("node"));
        Assert.NotNull(loader.GetTool("composer"));
        Assert.NotNull(loader.GetTool("git"));
        Assert.NotNull(loader.GetTool("nginx"));
        Assert.NotNull(loader.GetTool("bun"));
        Assert.NotNull(loader.GetTool("go"));
    }

    [Fact]
    public void PhpManifest_HasValidMetadataAndVersions()
    {
        var loader = new CatalogLoader();
        loader.LoadCatalog();

        var php = loader.GetTool("php");
        Assert.NotNull(php);
        Assert.Equal("language", php.Category);
        Assert.Contains("windows.php.net", php.AllowedHosts);
        Assert.NotEmpty(php.Versions);
        Assert.NotEmpty(php.Versions[0].Source.Sha256);
    }
}
