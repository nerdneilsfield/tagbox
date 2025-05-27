# TagBox Database Initialization Script (PowerShell)
# This script initializes a TagBox database using the SQL schema

param(
    [Parameter(Position=0, Mandatory=$true)]
    [string]$DatabasePath,
    
    [Parameter()]
    [switch]$Force,
    
    [Parameter()]
    [switch]$Verbose,
    
    [Parameter()]
    [switch]$Help
)

# Colors for output
$Colors = @{
    Red = 'Red'
    Green = 'Green'
    Yellow = 'Yellow'
    Blue = 'Blue'
    White = 'White'
}

# Script paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$SqlFile = Join-Path $ScriptDir "init-database.sql"

function Show-Help {
    Write-Host @"
TagBox Database Initialization Script (PowerShell)

USAGE:
    .\init-db.ps1 [OPTIONS] <database_path>

ARGUMENTS:
    <database_path>    Path to the SQLite database file (will be created if doesn't exist)

OPTIONS:
    -Force            Force recreate database (will delete existing data!)
    -Verbose          Enable verbose output
    -Help             Show this help message

EXAMPLES:
    # Initialize a new database
    .\init-db.ps1 .\tagbox.db
    
    # Initialize with verbose output
    .\init-db.ps1 -Verbose .\data\tagbox.db
    
    # Force recreate existing database
    .\init-db.ps1 -Force .\tagbox.db
    
    # Using DATABASE_URL environment variable
    `$env:DATABASE_URL = "sqlite:.\tagbox.db"
    .\init-db.ps1 (`$env:DATABASE_URL -replace "^sqlite:", "")

NOTES:
    - The script requires sqlite3.exe to be available in PATH
    - Database file and parent directories will be created if they don't exist
    - Use -Force carefully as it will delete all existing data
"@
}

function Write-LogInfo {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Colors.Blue
}

function Write-LogSuccess {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Green
}

function Write-LogWarning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Yellow
}

function Write-LogError {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Red
}

function Test-Sqlite3 {
    try {
        $version = & sqlite3 -version 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "sqlite3 execution failed"
        }
        
        if ($Verbose) {
            Write-LogInfo "Using SQLite version: $($version.Split()[0])"
        }
        return $true
    }
    catch {
        Write-LogError "sqlite3 command not found or failed to execute!"
        Write-LogError "Please install SQLite3:"
        Write-LogError "  - Download from: https://www.sqlite.org/download.html"
        Write-LogError "  - Or use chocolatey: choco install sqlite"
        Write-LogError "  - Or use scoop: scoop install sqlite"
        return $false
    }
}

function Initialize-Database {
    param([string]$DbPath)
    
    # Get absolute path
    $DbPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($DbPath)
    $DbDir = Split-Path -Parent $DbPath
    
    # Create parent directories if they don't exist
    if (-not (Test-Path $DbDir)) {
        Write-LogInfo "Creating directory: $DbDir"
        New-Item -ItemType Directory -Path $DbDir -Force | Out-Null
    }
    
    # Check if database exists
    if (Test-Path $DbPath) {
        if ($Force) {
            Write-LogWarning "Removing existing database: $DbPath"
            Remove-Item $DbPath -Force
        }
        else {
            Write-LogError "Database already exists: $DbPath"
            Write-LogError "Use -Force to recreate the database (WARNING: this will delete all data)"
            exit 1
        }
    }
    
    # Validate SQL file exists
    if (-not (Test-Path $SqlFile)) {
        Write-LogError "SQL initialization file not found: $SqlFile"
        exit 1
    }
    
    Write-LogInfo "Initializing database: $DbPath"
    Write-LogInfo "Using SQL file: $SqlFile"
    
    # Initialize database with SQL script
    try {
        if ($Verbose) {
            Write-LogInfo "Executing SQL initialization script..."
            Get-Content $SqlFile | & sqlite3 $DbPath
        }
        else {
            Get-Content $SqlFile | & sqlite3 $DbPath 2>$null
        }
        
        if ($LASTEXITCODE -ne 0) {
            throw "sqlite3 execution failed with exit code: $LASTEXITCODE"
        }
    }
    catch {
        Write-LogError "Failed to initialize database: $_"
        exit 1
    }
    
    # Verify database was created successfully
    if (-not (Test-Path $DbPath)) {
        Write-LogError "Failed to create database!"
        exit 1
    }
    
    # Check if tables were created
    try {
        $tableCount = & sqlite3 $DbPath "SELECT COUNT(*) FROM sqlite_master WHERE type='table';"
        
        if ($tableCount -eq 0) {
            Write-LogError "No tables were created in the database!"
            exit 1
        }
        
        Write-LogSuccess "Database initialized successfully!"
        Write-LogInfo "Database path: $DbPath"
        Write-LogInfo "Tables created: $tableCount"
        
        if ($Verbose) {
            Write-LogInfo "Tables in database:"
            $tables = & sqlite3 $DbPath "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
            $tables | ForEach-Object { Write-Host "  - $_" }
        }
        
        # Show database info
        $dbSize = (Get-Item $DbPath).Length
        $dbSizeFormatted = if ($dbSize -lt 1KB) { "$dbSize bytes" }
                          elseif ($dbSize -lt 1MB) { "{0:N1} KB" -f ($dbSize / 1KB) }
                          else { "{0:N1} MB" -f ($dbSize / 1MB) }
        
        Write-LogInfo "Database size: $dbSizeFormatted"
        
        # Test database connectivity
        try {
            $schemaVersion = & sqlite3 $DbPath "SELECT value FROM system_config WHERE key='schema_version';" 2>$null
            if (-not $schemaVersion) { $schemaVersion = "unknown" }
            Write-LogInfo "Schema version: $schemaVersion"
        }
        catch {
            Write-LogInfo "Schema version: unknown"
        }
    }
    catch {
        Write-LogError "Failed to verify database: $_"
        exit 1
    }
}

function Main {
    if ($Help) {
        Show-Help
        exit 0
    }
    
    if ([string]::IsNullOrEmpty($DatabasePath)) {
        Write-LogError "Database path is required!"
        Write-Host ""
        Show-Help
        exit 1
    }
    
    Write-LogInfo "TagBox Database Initialization Script"
    Write-LogInfo "======================================"
    
    if (-not (Test-Sqlite3)) {
        exit 1
    }
    
    Initialize-Database $DatabasePath
    
    Write-Host ""
    Write-LogSuccess "Database initialization completed!"
    Write-LogInfo "You can now use TagBox with this database:"
    Write-LogInfo "  `$env:DATABASE_URL = `"sqlite:$DatabasePath`""
    Write-LogInfo "  cargo run --bin tagbox-cli -- --help"
}

# Set error action preference
$ErrorActionPreference = "Stop"

# Run main function
Main