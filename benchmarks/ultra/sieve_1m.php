<?php
function sieve($n) {
    $count = 0;
    $primes = array_fill(0, $n + 1, true);
    $p = 2;
    while ($p * $p <= $n) {
        if ($primes[$p] === true) {
            for ($i = $p * $p; $i <= $n; $i += $p) {
                $primes[$i] = false;
            }
        }
        $p++;
    }
    for ($k = 2; $k <= $n; $k++) {
        if ($primes[$k] === true) $count++;
    }
    return $count;
}
$start = microtime(true);
$res = sieve(1000000);
$end = microtime(true);
echo "PHP Sieve(1M) Result: $res\n";
echo "PHP Sieve(1M) Time: " . ($end - $start) . "s\n";
