using dotnet.Services;
using Xunit;

namespace Dotnet.Core.Tests.Services;

public class ProcessTrackerTests
{
    [Fact]
    public void RecordProcess_And_RemoveProcess_ShouldPersistAndClean()
    {
        string serviceId = "unit-test-svc-" + Guid.NewGuid().ToString("N");
        int fakePid = 999999;
        string exePath = @"C:\dummy\path\app.exe";

        ProcessTracker.RecordProcess(serviceId, fakePid, exePath);

        var list = ProcessTracker.GetTrackedProcesses();
        var record = list.FirstOrDefault(r => r.ServiceId == serviceId);
        Assert.NotNull(record);
        Assert.Equal(fakePid, record.Pid);
        Assert.Equal(exePath, record.ExecutablePath);

        ProcessTracker.RemoveProcess(serviceId);

        var listAfter = ProcessTracker.GetTrackedProcesses();
        Assert.DoesNotContain(listAfter, r => r.ServiceId == serviceId);
    }

    [Fact]
    public void TryAdoptProcess_WithNonExistentPid_ShouldReturnNullAndCleanRecord()
    {
        string serviceId = "dead-svc-" + Guid.NewGuid().ToString("N");
        int fakePid = 999998;
        string exePath = @"C:\dummy\path\app.exe";

        ProcessTracker.RecordProcess(serviceId, fakePid, exePath);

        var proc = ProcessTracker.TryAdoptProcess(serviceId, exePath);
        Assert.Null(proc);

        // Record should have been pruned
        var list = ProcessTracker.GetTrackedProcesses();
        Assert.DoesNotContain(list, r => r.ServiceId == serviceId);
    }
}
