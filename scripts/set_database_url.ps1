param (
    [string]$databaseUrl
)

# Définit dans l'environnement utilisateur (persistant)
[System.Environment]::SetEnvironmentVariable("DATABASE_URL", $databaseUrl, "User")

Write-Output "DATABASE_URL set to: $databaseUrl"