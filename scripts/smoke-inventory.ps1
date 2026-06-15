$ErrorActionPreference = "Stop"

$baseUrl = $env:BACKEND_BASE_URL
if (-not $baseUrl) {
    $baseUrl = "http://127.0.0.1:8080"
}

$email = $env:BOOTSTRAP_ADMIN_EMAIL
if (-not $email) {
    $email = "admin@tuluklar.local"
}

$password = $env:BOOTSTRAP_ADMIN_PASSWORD
if (-not $password) {
    $password = "Admin12345!"
}

Write-Host "Smoke test inventory: $baseUrl"

$login = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/auth/login" `
    -ContentType "application/json" `
    -Body (@{
        email = $email
        password = $password
    } | ConvertTo-Json)

$headers = @{ Authorization = "Bearer $($login.access_token)" }
$stamp = Get-Date -Format "yyyyMMddHHmmss"

$product = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/products" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_name = "Smoke Plastik Bardak $stamp"
        category = "Sarf"
        sub_category = "Mutfak"
        brand = "Smoke"
        description = "Inventory smoke product"
        main_unit = "adet"
        package_unit = "koli"
        package_multiplier = 1000
        minimum_stock = 500
        maximum_stock = 10000
        critical_stock = 750
        safety_stock = 1000
        unit_cost = 1.25
        is_lot_tracked = $true
        expiry_tracking = $true
    } | ConvertTo-Json)
Write-Host "[OK] product id=$($product.id)"

$lot = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/lots" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_id = $product.id
        lot_number = "LOT-$stamp"
        production_date = "2026-06-01"
        expiry_date = "2026-07-01"
        supplier = "Smoke Supplier"
        quantity = 5000
    } | ConvertTo-Json)
Write-Host "[OK] lot id=$($lot.id)"

$inMovement = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/movements" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_id = $product.id
        lot_id = $lot.id
        movement_type = "warehouse_in"
        quantity = 5
        unit = "koli"
        unit_multiplier = 1000
        description = "Smoke 5 koli giris"
    } | ConvertTo-Json)
if ([decimal]$inMovement.next_stock -ne 5000) {
    throw "Warehouse in expected next_stock 5000, got $($inMovement.next_stock)"
}
Write-Host "[OK] warehouse in next_stock=$($inMovement.next_stock)"

$outMovement = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/movements" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_id = $product.id
        movement_type = "warehouse_out"
        quantity = 150
        unit = "adet"
        unit_multiplier = 1
        description = "Smoke 150 adet cikis"
    } | ConvertTo-Json)
if ([decimal]$outMovement.next_stock -ne 4850) {
    throw "Warehouse out expected next_stock 4850, got $($outMovement.next_stock)"
}
Write-Host "[OK] warehouse out next_stock=$($outMovement.next_stock)"

$productAfterOut = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/products/$($product.id)" -Headers $headers
$currentStock = [decimal]$productAfterOut.current_stock
if ($currentStock -ne 4850) {
    throw "Product current_stock expected 4850, got $currentStock"
}
Write-Host "[OK] product current_stock=$currentStock"

$otherProduct = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/products" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_name = "Smoke Lot Guard $stamp"
        category = "Sarf"
        main_unit = "adet"
        minimum_stock = 1
    } | ConvertTo-Json)

try {
    Invoke-RestMethod `
        -Method Post `
        -Uri "$baseUrl/api/v1/inventory/movements" `
        -Headers $headers `
        -ContentType "application/json" `
        -Body (@{
            product_id = $otherProduct.id
            lot_id = $lot.id
            movement_type = "warehouse_in"
            quantity = 1
            unit = "adet"
            unit_multiplier = 1
            description = "Smoke wrong product lot guard"
        } | ConvertTo-Json) | Out-Null
    throw "Wrong-product lot movement should have failed"
} catch {
    if ($_.Exception.Response.StatusCode.value__ -ne 400) {
        throw
    }
}
Write-Host "[OK] lot product guard"

$shipment = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/shipments" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        branch_id = $null
        note = "Smoke sevkiyat detay"
        items = @(
            @{
                product_id = $product.id
                quantity = 2
                unit = "koli"
                unit_multiplier = 1000
            }
        )
    } | ConvertTo-Json -Depth 5)
Write-Host "[OK] shipment id=$($shipment.id)"

$shipmentDetail = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/shipments/$($shipment.id)" -Headers $headers
if ($shipmentDetail.items.Count -ne 1 -or [decimal]$shipmentDetail.items[0].base_quantity -ne 2000) {
    throw "Shipment detail did not return expected item base quantity"
}
Write-Host "[OK] shipment detail items=$($shipmentDetail.items.Count)"

$count = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/counts" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        count_type = "partial"
        count_method = "manual"
        note = "Smoke sayim"
        items = @(
            @{
                product_id = $product.id
                counted_stock = 4800
                reason = "Smoke negative adjustment"
            }
        )
    } | ConvertTo-Json -Depth 5)
Write-Host "[OK] count id=$($count.id)"

$countDetail = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/counts/$($count.id)" -Headers $headers
if ($countDetail.items.Count -ne 1 -or [decimal]$countDetail.items[0].difference -ne -50) {
    throw "Count detail did not return expected difference"
}
Write-Host "[OK] count detail items=$($countDetail.items.Count)"

$completedCount = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/counts/$($count.id)/complete" `
    -Headers $headers
if ($completedCount.count_status -ne "completed") {
    throw "Count completion failed"
}
Write-Host "[OK] count completed"

$productAfterCount = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/products/$($product.id)" -Headers $headers
$stockAfterCount = [decimal]$productAfterCount.current_stock
if ($stockAfterCount -ne 4800) {
    throw "Count adjustment expected current_stock 4800, got $stockAfterCount"
}
Write-Host "[OK] count adjustment current_stock=$stockAfterCount"

$dashboard = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/dashboard" -Headers $headers
if ($dashboard.total_products -lt 1) {
    throw "Inventory dashboard did not return product count"
}
Write-Host "[OK] inventory dashboard"

$alerts = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/alerts" -Headers $headers
if ($null -eq $alerts) {
    throw "Inventory alerts did not return a response"
}
Write-Host "[OK] inventory alerts"

$suggestions = Invoke-RestMethod -Uri "$baseUrl/api/v1/inventory/purchase-suggestions" -Headers $headers
if ($null -eq $suggestions) {
    throw "Inventory purchase suggestions did not return a response"
}
Write-Host "[OK] purchase suggestions"

$mobileUserEmail = "mobile-user-$stamp@tuluklar.local"
$mobileUserPassword = "Mobile12345!"
$mobileUser = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/users" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        email = $mobileUserEmail
        full_name = "Smoke Mobil Kullanici $stamp"
        password = $mobileUserPassword
        role = "user"
    } | ConvertTo-Json)
if (-not $mobileUser.id) {
    throw "Mobile user create failed"
}
Write-Host "[OK] mobile user id=$($mobileUser.id)"

$mobilePermission = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/users/$($mobileUser.id)/mobile-permissions" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        permission_key = "inventory_order"
        can_view = $true
        can_create = $true
        can_update = $false
        can_approve = $false
    } | ConvertTo-Json)
if ($mobilePermission.permission_key -ne "inventory_order" -or -not $mobilePermission.can_create) {
    throw "Mobile permission upsert failed"
}
Write-Host "[OK] mobile permission id=$($mobilePermission.id)"

$mobilePermissions = Invoke-RestMethod -Uri "$baseUrl/api/v1/users/$($mobileUser.id)/mobile-permissions" -Headers $headers
if (-not ($mobilePermissions | Where-Object { $_.permission_key -eq "inventory_order" })) {
    throw "Mobile permission list failed"
}
Write-Host "[OK] mobile permission list"

$mobileLogin = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/auth/login" `
    -ContentType "application/json" `
    -Body (@{
        email = $mobileUserEmail
        password = $mobileUserPassword
    } | ConvertTo-Json)
$mobileHeaders = @{ Authorization = "Bearer $($mobileLogin.access_token)" }
$mobilePurchaseRequest = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/purchase-requests" `
    -Headers $mobileHeaders `
    -ContentType "application/json" `
    -Body (@{
        product_id = $product.id
        requested_quantity = 125
        reason = "Smoke mobile purchase request"
        note = "Mobil kullanici siparis talebi"
    } | ConvertTo-Json)
if (-not $mobilePurchaseRequest.id -or $mobilePurchaseRequest.created_by -ne $mobileUser.id) {
    throw "Mobile user purchase request failed"
}
Write-Host "[OK] mobile user purchase request id=$($mobilePurchaseRequest.id)"

$purchaseRequest = Invoke-RestMethod `
    -Method Post `
    -Uri "$baseUrl/api/v1/inventory/purchase-requests" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        product_id = $product.id
        requested_quantity = 250
        reason = "Smoke purchase request"
        note = "Smoke purchase request note"
    } | ConvertTo-Json)
if (-not $purchaseRequest.id) {
    throw "Inventory purchase request did not return id"
}
Write-Host "[OK] purchase request id=$($purchaseRequest.id)"

$purchaseRequestUpdated = Invoke-RestMethod `
    -Method Patch `
    -Uri "$baseUrl/api/v1/inventory/purchase-requests/$($purchaseRequest.id)" `
    -Headers $headers `
    -ContentType "application/json" `
    -Body (@{
        request_status = "approved"
        approved_quantity = 250
    } | ConvertTo-Json)
if ($purchaseRequestUpdated.request_status -ne "approved") {
    throw "Inventory purchase request status update failed"
}
Write-Host "[OK] purchase request update"

Invoke-RestMethod -Method Delete -Uri "$baseUrl/api/v1/inventory/products/$($product.id)" -Headers $headers | Out-Null
Write-Host "[OK] product archive"

Write-Host "Inventory smoke test passed"
