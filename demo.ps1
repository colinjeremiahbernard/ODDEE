# ODDEƎ Demo Script
# Demonstrates event ingestion and anomaly detection via the REST API.
# Adjust $baseUrl if your backend runs on a different host/port.

$baseUrl = "http://localhost:3000"

function Invoke-OddeeApi {
    param(
        [string]$Method,
        [string]$Path,
        [object]$Body = $null
    )

    $uri = "$baseUrl$Path"
    $headers = @{
        "Accept" = "application/json"
    }

    $params = @{
        Method      = $Method
        Uri         = $uri
        Headers     = $headers
        ContentType = "application/json"
    }

    if ($Body) {
        $params.Body = ($Body | ConvertTo-Json -Depth 10 -Compress)
    }

    try {
        $result = Invoke-RestMethod @params
        return $result
    }
    catch {
        Write-Host "API error: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}

function New-PhysicalEvent {
    param(
        [string]$EntityId,
        [string]$Kind,
        [string]$Zone,
        [string]$Source = "camera",
        [hashtable]$Metadata = @{}
    )

    $body = @{
        occurred_at = (Get-Date).ToUniversalTime().ToString("o")
        entity_id   = $EntityId
        kind        = $Kind
        zone        = $Zone
        source      = $Source
        metadata    = $Metadata
    }

    return Invoke-OddeeApi -Method Post -Path "/events" -Body $body
}

function Get-Anomalies {
    param(
        [string]$Severity = "",
        [string]$Status   = ""
    )

    $query = @()
    if ($Severity) { $query += "severity=$Severity" }
    if ($Status)   { $query += "status=$Status" }

    $path = "/anomalies"
    if ($query.Count -gt 0) {
        $path += "?" + ($query -join "&")
    }

    return Invoke-OddeeApi -Method Get -Path $path
}

# -------------------------
# Demo scenario
# -------------------------

Write-Host "ODDEƎ Demo - Event ingestion and anomaly detection" -ForegroundColor Cyan
Write-Host "Base URL: $baseUrl" -ForegroundColor Gray
Write-Host ""

# Scenario: package enters the same zone twice in quick succession (no exit)
$entityId = "package-BX-2041"
$zone     = "warehouse-a"

Write-Host "Step 1: Ingest first entry event for $entityId in $zone" -ForegroundColor Yellow
$event1 = New-PhysicalEvent `
    -EntityId $entityId `
    -Kind     "entered_zone" `
    -Zone     $zone `
    -Metadata @{ camera_id = "cam-01"; confidence = 0.96 }

Write-Host "Created event: $($event1.id)" -ForegroundColor Gray
Write-Host ""

Start-Sleep -Seconds 1

Write-Host "Step 2: Ingest second entry event for same $entityId in $zone (no exit)" -ForegroundColor Yellow
$event2 = New-PhysicalEvent `
    -EntityId $entityId `
    -Kind     "entered_zone" `
    -Zone     $zone `
    -Metadata @{ camera_id = "cam-01"; confidence = 0.94 }

Write-Host "Created event: $($event2.id)" -ForegroundColor Gray
Write-Host ""

Start-Sleep -Seconds 1

Write-Host "Step 3: Query anomalies (expect a 'repeated zone entry' anomaly)" -ForegroundColor Yellow
$anomalies = Get-Anomalies

if ($anomalies.items.Count -eq 0) {
    Write-Host "No anomalies detected yet." -ForegroundColor Gray
}
else {
    Write-Host "Detected $($anomalies.items.Count) anomaly/anomalies:" -ForegroundColor Green
    foreach ($a in $anomalies.items) {
        Write-Host ""
        Write-Host "ID:         $($a.id)" -ForegroundColor Cyan
        Write-Host "Severity:   $($a.severity)" -ForegroundColor Cyan
        Write-Host "Score:      $($a.score)" -ForegroundColor Cyan
        Write-Host "Title:      $($a.title)" -ForegroundColor Cyan
        Write-Host "Explanation:$($a.explanation)" -ForegroundColor Cyan
        Write-Host "Status:     $($a.status)" -ForegroundColor Cyan
        if ($a.related_event_ids) {
            Write-Host "Related events: $($a.related_event_ids -join ', ')" -ForegroundColor Gray
        }
    }
}

Write-Host ""
Write-Host "Demo complete." -ForegroundColor Cyan