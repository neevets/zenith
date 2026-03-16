<?php
function sieve($n) {
    $count = 0;
    $primes = array_fill(0, $n + 1, true);

    for ($p = 2; $p * $p <= $n; $p++) {
        if ($primes[$p] === true) {
            for ($i = $p * $p; $i <= $n; $i += $p) {
                $primes[$i] = false;
            }
        }
    }

    for ($p = 2; $p <= $n; $p++) {
        if ($primes[$p] === true) {
            $count++;
        }
    }
    return $count;
}

$start = microtime(true);
$result = sieve(10000);
$end = microtime(true);
$elapsed = $elapsed = microtime(true) - $start;

echo "PHP Prime Sieve(10000) Result: " . $result . PHP_EOL;
echo "PHP Prime Sieve(10000) Time: " . $elapsed . " seconds" . PHP_EOL;
