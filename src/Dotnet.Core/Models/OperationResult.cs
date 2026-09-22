namespace dotnet.Models;

public sealed record OperationResult(bool Success, string? Message = null, Exception? Error = null)
{
    public static OperationResult Ok(string? message = null) => new(true, message);
    public static OperationResult Fail(string message, Exception? error = null) => new(false, message, error);
}
