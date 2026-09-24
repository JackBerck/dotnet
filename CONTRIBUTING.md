# Contributing to Dotnet (Standalone DevManager)

Thank you for your interest in contributing to **Dotnet**! This project is a native, lightweight, and modern Windows developer environment manager (inspired by Laragon).

## Development Setup

1. **Prerequisites:**
   - Windows 10/11 x64
   - .NET 10.0 SDK
   - Visual Studio 2022 / VS Code / JetBrains Rider

2. **Clone & Build:**
   ```powershell
   git clone https://github.com/JackBerck/dotnet.git
   cd dotnet
   dotnet restore
   dotnet build
   dotnet test
   ```

3. **Architecture Principles:**
   - `Dotnet.Core` contains zero UI dependencies and is completely testable.
   - All I/O, registry edits, and OS operations must follow atomic writes and backup procedures (`JsonStore`, `AtomicFile`, `IniDocument`).
   - Classic Windows 7/XP aesthetics are preserved in `Dotnet.App` per `STYLE-SPEC.md`.

## Pull Request Guidelines

1. Ensure `dotnet build` produces **0 warnings and 0 errors**.
2. Run `dotnet test` to confirm all unit tests pass.
3. Follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g. `feat:`, `fix:`, `docs:`, `refactor:`).
4. Provide a clear explanation of changes in the PR description.
