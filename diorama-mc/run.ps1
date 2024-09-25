# RUN THIS COMMNAD IN POWERSHELL TO BUILD AND RUN THE PROJECT INSIDE THE RAYTRACING FOLDER
# Set the working directory to the script's location if needed
Set-Location -Path $PSScriptRoot

# Run cargo to build the project in release mode
cargo build --release

# Get the current directory name which is assumed to be the project name
$currentDir = Split-Path -Path $PWD -Leaf

# Define the path to the executable
$executablePath = ".\target\release\" + $currentDir + ".exe"

# Check if the executable exists
if (Test-Path $executablePath) {
    # If it exists, execute the compiled program
    & $executablePath
} else {
    Write-Error "Executable not found: $executablePath"
}
