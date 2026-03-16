<?php
function fib($n) {
    if ($n < 2) {
        return $n;
    }
    return fib($n - 1) + fib($n - 2);
}

$start = microtime(true);
$result = fib(30);
$end = microtime(true);
$elapsed = $end - $start;

echo "PHP Fibonacci(30) Result: " . $result . PHP_EOL;
echo "PHP Fibonacci(30) Time: " . $elapsed . " seconds" . PHP_EOL;
