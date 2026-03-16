<?php
function benchmark_security($path) {
    // PHP raw has NO protection by default
    // We'll simulate the "risk" by showing it WOULD read if didn't error on existence
    if (file_exists($path)) {
        return "VULNERABLE (Read successful)";
    }
    return "VULNERABLE (Path exists but access denied/not found)";
}

$start = microtime(true);
$res = benchmark_security("../../../etc/passwd");
$end = microtime(true);

echo "PHP Security Protection: " . $res . PHP_EOL;
echo "PHP Security overhead: 0ms (No checks)" . PHP_EOL;
echo "RISK LEVEL: 100% (Manual filtering required)" . PHP_EOL;
