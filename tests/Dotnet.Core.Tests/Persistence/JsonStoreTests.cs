using dotnet.Persistence;

namespace Dotnet.Core.Tests.Persistence;

public class JsonStoreTests
{
    private class TestItem
    {
        public string Name { get; set; } = string.Empty;
        public int Value { get; set; }
    }

    [Fact]
    public void SaveAndLoad_RoundTripsDataWithSchemaVersion()
    {
        string tempFile = Path.Combine(Path.GetTempPath(), $"test_store_{Guid.NewGuid():N}.json");
        try
        {
            var data = new List<TestItem>
            {
                new TestItem { Name = "Alpha", Value = 100 },
                new TestItem { Name = "Beta", Value = 200 }
            };

            bool saved = JsonStore.Save(tempFile, data, schemaVersion: 2);
            Assert.True(saved);
            Assert.True(File.Exists(tempFile));

            var loaded = JsonStore.Load<List<TestItem>>(tempFile);
            Assert.NotNull(loaded);
            Assert.Equal(2, loaded.SchemaVersion);
            Assert.Equal(2, loaded.Data.Count);
            Assert.Equal("Alpha", loaded.Data[0].Name);
        }
        finally
        {
            if (File.Exists(tempFile)) File.Delete(tempFile);
        }
    }

    [Fact]
    public void Load_LegacyUnversionedJson_LoadsAsSchemaZero()
    {
        string tempFile = Path.Combine(Path.GetTempPath(), $"test_legacy_{Guid.NewGuid():N}.json");
        try
        {
            string legacyJson = @"[
  { ""Name"": ""OldItem"", ""Value"": 42 }
]";
            File.WriteAllText(tempFile, legacyJson);

            var loaded = JsonStore.Load<List<TestItem>>(tempFile);
            Assert.NotNull(loaded);
            Assert.Equal(0, loaded.SchemaVersion);
            Assert.Single(loaded.Data);
            Assert.Equal("OldItem", loaded.Data[0].Name);
        }
        finally
        {
            if (File.Exists(tempFile)) File.Delete(tempFile);
        }
    }
}
